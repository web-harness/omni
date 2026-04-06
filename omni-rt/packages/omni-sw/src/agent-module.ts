import { createDeepAgent } from "deepagents";
import { SqlJsSaver } from "./checkpointer.js";
import type { ProtocolMessage } from "./run-store.js";
import { readSseEvents, type SseFrame } from "./sse.js";
import { BashkitSandboxBackend } from "./agent/sandbox.js";
import { invocationMessages, isObject, resolveAgentModel, type AgentRunRequest } from "./agent/shared.js";
import { buildSystemPrompt } from "./agent/system-prompt.js";

export type AgentModuleEvent = SseFrame;

type RunRequest = AgentRunRequest & {
  thread_id?: string;
  if_not_exists?: "create" | "reject";
  metadata?: Record<string, unknown>;
  stream_mode?: "values" | "messages" | Array<"values" | "messages">;
};

export async function executeRunStream(
  body: Record<string, unknown>,
  baseUrl = "",
): Promise<AsyncGenerator<AgentModuleEvent, void, void>> {
  const request = body as RunRequest;
  const mode = typeof request.metadata?.agent_mode === "string" ? request.metadata.agent_mode : "main";
  if (mode === "direct") {
    const response = await requestDirectRunStream(request);
    if (!response.ok) {
      throw new Error(await response.text().catch(() => "run stream failed"));
    }
    if (!response.body) {
      throw new Error("run stream response body is missing");
    }
    return readSseEvents(response.body);
  }
  return executeDesktopRun(request, normalizeBaseUrl(baseUrl));
}

async function requestDirectRunStream(body: RunRequest): Promise<Response> {
  const metadata = isObject(body.metadata) ? body.metadata : {};
  const agentUrl = typeof metadata.agent_url === "string" ? metadata.agent_url.trim() : "";
  if (!agentUrl) {
    throw new Error("direct agent is missing agent_url");
  }

  const outboundMetadata = { ...metadata };
  delete outboundMetadata.agent_url;
  delete outboundMetadata.agent_bearer_token;
  delete outboundMetadata.agent_name;
  delete outboundMetadata.agent_mode;

  const response = await fetch(`${agentUrl.replace(/\/$/, "")}/runs/stream`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(typeof metadata.agent_bearer_token === "string" && metadata.agent_bearer_token.trim()
        ? { Authorization: `Bearer ${metadata.agent_bearer_token.trim()}` }
        : {}),
    },
    body: JSON.stringify({
      ...body,
      metadata: outboundMetadata,
    }),
  });

  return response;
}

function normalizeBaseUrl(baseUrl: string): string {
  const normalizedBase = baseUrl.trim().replace(/\/$/, "");
  if (!normalizedBase) {
    throw new Error("baseUrl is required for desktop run streaming");
  }
  return normalizedBase;
}

function resolvedStreamModes(value: RunRequest["stream_mode"]): Array<"values" | "messages"> {
  if (!value) {
    return ["values"];
  }
  return Array.isArray(value) ? value : [value];
}

function normalizedInputMessages(body: RunRequest): ProtocolMessage[] {
  if (Array.isArray(body.messages) && body.messages.length > 0) {
    return body.messages.map((message) => ({
      ...message,
      id: typeof message.id === "string" && message.id ? message.id : crypto.randomUUID(),
    }));
  }
  return [
    {
      id: crypto.randomUUID(),
      role: "user",
      content: body.input,
    },
  ];
}

async function executeDesktopRun(
  body: RunRequest,
  baseUrl: string,
): Promise<AsyncGenerator<AgentModuleEvent, void, void>> {
  return streamDesktopRun(body, baseUrl);
}

async function* streamDesktopRun(body: RunRequest, baseUrl: string): AsyncGenerator<AgentModuleEvent, void, void> {
  const threadId = typeof body.thread_id === "string" && body.thread_id.trim() ? body.thread_id.trim() : undefined;
  if (!threadId) {
    throw new Error("thread_id is required for desktop deepagent runs");
  }

  await ensureThread(baseUrl, threadId, body.if_not_exists);
  const inputMessages = normalizedInputMessages(body);
  const workspacePath = await getWorkspacePath(baseUrl, threadId);
  await patchThread(baseUrl, threadId, { messages: inputMessages });

  const streamModes = resolvedStreamModes(body.stream_mode);
  let assistantContent = "";

  try {
    const model = await resolveAgentModel(baseUrl, { ...body, messages: inputMessages });
    const saver = new SqlJsSaver();
    const workflow = createDeepAgent({
      model,
      backend: new BashkitSandboxBackend(workspacePath, (command, cwd) => executeDesktopCommand(baseUrl, command, cwd)),
      checkpointer: saver,
      systemPrompt: buildSystemPrompt(workspacePath),
    });

    const stream = await workflow.stream(
      { messages: invocationMessages({ ...body, messages: inputMessages }) },
      {
        configurable: { thread_id: threadId },
        streamMode: "messages",
      },
    );

    for await (const [messageChunk] of stream) {
      if (!("content" in messageChunk) || typeof messageChunk.content !== "string" || !messageChunk.content) {
        continue;
      }
      assistantContent += messageChunk.content;
      if (streamModes.includes("messages")) {
        yield {
          event: "message",
          data: { role: "assistant", content: messageChunk.content },
        };
      }
    }

    const finalMessage: ProtocolMessage = {
      id: crypto.randomUUID(),
      role: "assistant",
      content: assistantContent,
    };
    const checkpointStates = await collectCheckpointStates(saver, threadId);
    const finalState = checkpointStates.at(-1);
    if (checkpointStates.length > 0) {
      for (const state of checkpointStates) {
        await patchThread(baseUrl, threadId, state);
      }
    }

    const values = finalState?.values ?? { output: assistantContent };
    const messages = finalState?.messages?.length ? finalState.messages : [finalMessage];

    await patchThread(baseUrl, threadId, { values, messages });

    if (streamModes.includes("values")) {
      yield { event: "values", data: values };
    }
    if (streamModes.includes("messages")) {
      yield { event: "messages/complete", data: messages };
    }
    yield { event: "end", data: null };
  } catch (error) {
    yield {
      event: "error",
      data: { message: error instanceof Error ? error.message : String(error) },
    };
    yield { event: "end", data: null };
  }
}

async function ensureThread(
  baseUrl: string,
  threadId: string,
  ifNotExists: RunRequest["if_not_exists"],
): Promise<void> {
  const response = await fetch(`${baseUrl}/threads/${threadId}`);
  if (response.ok) {
    return;
  }
  if (response.status !== 404) {
    throw new Error(await response.text().catch(() => "failed to load thread"));
  }
  if (ifNotExists !== "create") {
    throw new Error(`Thread ${threadId} not found`);
  }
  const createResponse = await fetch(`${baseUrl}/threads`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ thread_id: threadId }),
  });
  if (!createResponse.ok) {
    throw new Error(await createResponse.text().catch(() => "failed to create thread"));
  }
}

async function getWorkspacePath(baseUrl: string, threadId: string): Promise<string> {
  const response = await fetch(`${baseUrl}/threads/${threadId}`);
  if (!response.ok) {
    throw new Error(await response.text().catch(() => "failed to read thread"));
  }
  const thread = (await response.json()) as Record<string, unknown>;
  const metadata = isObject(thread.metadata) ? thread.metadata : undefined;
  return typeof metadata?.workspace === "string" && metadata.workspace ? metadata.workspace : "/home/workspace";
}

async function patchThread(baseUrl: string, threadId: string, patch: Record<string, unknown>): Promise<void> {
  const response = await fetch(`${baseUrl}/threads/${threadId}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(patch),
  });
  if (!response.ok) {
    throw new Error(await response.text().catch(() => "failed to patch thread"));
  }
}

async function collectCheckpointStates(saver: SqlJsSaver, threadId: string): Promise<Array<Record<string, unknown>>> {
  const states: Array<Record<string, unknown>> = [];
  for await (const tuple of saver.list({ configurable: { thread_id: threadId } })) {
    const checkpoint = tuple.checkpoint as Record<string, unknown>;
    const values = isObject(checkpoint.channel_values) ? checkpoint.channel_values : checkpoint;
    const messages = extractMessagesFromValues(values);
    const checkpointFields = Object.fromEntries(
      Object.entries(checkpoint).filter(([key]) => key !== "id" && key !== "channel_values"),
    );

    states.push({
      checkpoint: {
        checkpoint_id: String(checkpoint.id ?? tuple.config?.configurable?.checkpoint_id ?? ""),
        ...checkpointFields,
      },
      values: sanitizeThreadValues(values) ?? {},
      ...(messages.length ? { messages } : {}),
    });
  }

  return states.reverse();
}

function sanitizeThreadValues(values: unknown): Record<string, unknown> | undefined {
  if (!isObject(values)) {
    return undefined;
  }
  const next = { ...values };
  if (Array.isArray(next.messages)) {
    delete next.messages;
  }
  return next;
}

function extractMessagesFromValues(values: unknown): ProtocolMessage[] {
  if (!isObject(values) || !Array.isArray(values.messages)) {
    return [];
  }
  return values.messages
    .map((message) => (isObject(message) && typeof message.role === "string" && "content" in message ? message : null))
    .filter((message): message is ProtocolMessage => message !== null)
    .map((message) => ({
      ...message,
      id: typeof message.id === "string" && message.id ? message.id : crypto.randomUUID(),
    }));
}

async function executeDesktopCommand(
  baseUrl: string,
  command: string,
  cwd: string,
): Promise<{ output: string; exitCode: number; truncated: boolean }> {
  const response = await fetch(`${baseUrl}/x/execute`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ command, cwd }),
  });
  if (!response.ok) {
    throw new Error(await response.text().catch(() => "command execution failed"));
  }
  return (await response.json()) as { output: string; exitCode: number; truncated: boolean };
}

export default {
  executeRunStream,
};
