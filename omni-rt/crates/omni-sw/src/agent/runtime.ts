import { ChatAnthropic } from "@langchain/anthropic";
import { ChatOpenAI } from "@langchain/openai";
import { ChatGoogleGenerativeAI } from "@langchain/google-genai";
import { BaseChatModel } from "@langchain/core/language_models/chat_models";
import { HumanMessage } from "@langchain/core/messages";
import { StateGraph, MessagesAnnotation } from "@langchain/langgraph";
import { fs, configure } from "@zenfs/core";
import { IndexedDB } from "@zenfs/dom";
import { SqlJsSaver } from "../checkpointer.js";
import { buildSystemPrompt } from "./system-prompt.js";

interface RunRequest {
  thread_id: string;
  message: string;
  model_id: string;
}

async function ensureZenFs(): Promise<void> {
  try {
    await configure({ backend: IndexedDB, storeName: "omni" });
  } catch {
    // already configured
  }
}

async function readZenFsFile(path: string): Promise<string | null> {
  try {
    await ensureZenFs();
    const data = await fs.promises.readFile(path, "utf-8");
    return data as string;
  } catch {
    return null;
  }
}

async function getApiKey(provider: string): Promise<string | undefined> {
  const env = await readZenFsFile("/home/config/.env");
  if (!env) return undefined;
  for (const line of env.split("\n")) {
    const [key, ...rest] = line.split("=");
    if (key?.trim() === provider.toUpperCase() + "_API_KEY") {
      return rest.join("=").trim();
    }
  }
  return undefined;
}

async function getWorkspacePath(threadId: string): Promise<string> {
  const data = await readZenFsFile(`/home/db/threads/${threadId}.json`);
  if (data) {
    try {
      const thread = JSON.parse(data);
      return thread.metadata?.workspace ?? "/home/workspace";
    } catch {
      // ignore
    }
  }
  return "/home/workspace";
}

export function getModelInstance(modelId: string, apiKey: string): BaseChatModel {
  if (modelId.startsWith("claude-")) {
    return new ChatAnthropic({ model: modelId, anthropicApiKey: apiKey, streaming: true });
  }
  if (modelId.startsWith("gpt-")) {
    return new ChatOpenAI({ model: modelId, openAIApiKey: apiKey, streaming: true });
  }
  if (modelId.startsWith("gemini-")) {
    return new ChatGoogleGenerativeAI({ model: modelId, apiKey, streaming: true });
  }
  // fallback to OpenAI-compatible (Ollama etc.)
  return new ChatOpenAI({
    model: modelId,
    openAIApiKey: apiKey ?? "ollama",
    configuration: { baseURL: "http://localhost:11434/v1" },
    streaming: true,
  });
}

function providerForModel(modelId: string): string {
  if (modelId.startsWith("claude-")) return "anthropic";
  if (modelId.startsWith("gpt-")) return "openai";
  if (modelId.startsWith("gemini-")) return "google";
  return "ollama";
}

function sseChunk(data: string): Uint8Array {
  return new TextEncoder().encode(`data: ${data}\n\n`);
}

export async function handleRunStream(request: Request): Promise<Response> {
  const body = (await request.json()) as RunRequest;
  const { thread_id, message, model_id } = body;

  const workspacePath = await getWorkspacePath(thread_id);
  const provider = providerForModel(model_id);
  const apiKey = await getApiKey(provider);

  const stream = new ReadableStream({
    async start(controller) {
      try {
        if (!apiKey) {
          controller.enqueue(sseChunk(JSON.stringify({ type: "error", data: `No API key for ${provider}` })));
          controller.enqueue(sseChunk(JSON.stringify({ type: "done" })));
          controller.close();
          return;
        }

        const model = getModelInstance(model_id, apiKey);
        const checkpointer = new SqlJsSaver();
        const systemPrompt = buildSystemPrompt(workspacePath);

        const workflow = new StateGraph(MessagesAnnotation)
          .addNode("agent", async (state) => {
            const response = await model.invoke([{ role: "system", content: systemPrompt }, ...state.messages]);
            return { messages: [response] };
          })
          .addEdge("__start__", "agent")
          .addEdge("agent", "__end__")
          .compile({ checkpointer });

        const config = { configurable: { thread_id } };
        const result = await workflow.stream(
          { messages: [new HumanMessage(message)] },
          { ...config, streamMode: "messages" },
        );

        for await (const [msgChunk] of result) {
          if ("content" in msgChunk && typeof msgChunk.content === "string") {
            controller.enqueue(sseChunk(JSON.stringify({ type: "token", data: msgChunk.content })));
          }
        }

        controller.enqueue(sseChunk(JSON.stringify({ type: "done" })));
      } catch (err) {
        controller.enqueue(sseChunk(JSON.stringify({ type: "error", data: String(err) })));
      } finally {
        controller.close();
      }
    },
  });

  return new Response(stream, {
    headers: {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache",
      "Access-Control-Allow-Origin": "*",
    },
  });
}

export async function handleRunWait(request: Request): Promise<Response> {
  const body = (await request.json()) as RunRequest;
  const { thread_id, message, model_id } = body;

  const workspacePath = await getWorkspacePath(thread_id);
  const provider = providerForModel(model_id);
  const apiKey = await getApiKey(provider);

  if (!apiKey) {
    return Response.json({ error: `No API key for ${provider}` }, { status: 400 });
  }

  const model = getModelInstance(model_id, apiKey);
  const checkpointer = new SqlJsSaver(new JsonPlusSerializer());
  const systemPrompt = buildSystemPrompt(workspacePath);

  const workflow = new StateGraph(MessagesAnnotation)
    .addNode("agent", async (state) => {
      const response = await model.invoke([{ role: "system", content: systemPrompt }, ...state.messages]);
      return { messages: [response] };
    })
    .addEdge("__start__", "agent")
    .addEdge("agent", "__end__")
    .compile({ checkpointer });

  const config = { configurable: { thread_id } };
  const result = await workflow.invoke({ messages: [new HumanMessage(message)] }, config);

  const lastMsg = result.messages[result.messages.length - 1];
  return Response.json({ content: lastMsg?.content ?? "" });
}
