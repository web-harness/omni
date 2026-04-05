import type { ChatCompletionOptions, CompletionChunk, WllamaChatMessage } from "@wllama/wllama";
import { hasModel } from "./model-cache.js";
import { type BrowserInferenceStatus, getInferenceEngine } from "./engine.js";
import { BROWSER_MODELS, getModelSpec } from "./models.js";
import type { InferenceRoute } from "./protocol.js";

type OpenAIMessage = {
  role?: string;
  content?: unknown;
};

type OpenAIBaseRequest = {
  model?: string;
  stream?: boolean;
  temperature?: number;
  top_p?: number;
  max_tokens?: number;
};

type OpenAIChatRequest = OpenAIBaseRequest & {
  messages?: OpenAIMessage[];
};

type OpenAICompletionRequest = OpenAIBaseRequest & {
  prompt?: unknown;
};

const encoder = new TextEncoder();
const decoder = new TextDecoder();

function jsonResponse(body: unknown, init?: ResponseInit): Response {
  return Response.json(body, init);
}

function errorResponse(status: number, message: string): Response {
  return jsonResponse({ error: { message } }, { status });
}

function nowSeconds(): number {
  return Math.floor(Date.now() / 1000);
}

function completionId(prefix: string): string {
  return `${prefix}-${Date.now()}`;
}

function normalizeContent(content: unknown): string {
  if (typeof content === "string") {
    return content;
  }

  if (!Array.isArray(content)) {
    return "";
  }

  return content
    .map((part) => {
      if (typeof part === "string") {
        return part;
      }
      if (part && typeof part === "object" && "text" in part && typeof part.text === "string") {
        return part.text;
      }
      return "";
    })
    .filter(Boolean)
    .join("\n");
}

function normalizePrompt(prompt: unknown): string {
  if (typeof prompt === "string") {
    return prompt;
  }

  if (typeof prompt === "number") {
    return String(prompt);
  }

  if (!Array.isArray(prompt)) {
    return "";
  }

  return prompt
    .map((part) => {
      if (typeof part === "string") {
        return part;
      }
      if (typeof part === "number") {
        return String(part);
      }
      return "";
    })
    .filter(Boolean)
    .join("\n");
}

function toChatMessages(messages: OpenAIMessage[]): WllamaChatMessage[] {
  return messages.map((message) => {
    const role = String(message.role ?? "user").toLowerCase();
    return {
      role: role === "system" || role === "assistant" ? role : "user",
      content: normalizeContent(message.content),
    };
  });
}

function promptToMessages(prompt: string): WllamaChatMessage[] {
  return [{ role: "user", content: prompt }];
}

function toChatOptions(body: OpenAIBaseRequest): ChatCompletionOptions {
  return {
    nPredict: body.max_tokens,
    sampling: {
      temp: body.temperature,
      top_p: body.top_p,
    },
  };
}

function enqueueSse(controller: ReadableStreamDefaultController<Uint8Array>, lines: string[]): void {
  controller.enqueue(encoder.encode(lines.join("\n") + "\n\n"));
}

function statusStreamResponse(
  run: (controller: ReadableStreamDefaultController<Uint8Array>) => Promise<void>,
): Response {
  const stream = new ReadableStream<Uint8Array>({
    start(controller) {
      void run(controller).finally(() => controller.close());
    },
  });

  return new Response(stream, {
    headers: {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache, no-transform",
      Connection: "keep-alive",
    },
  });
}

async function handleModelsList(): Promise<Response> {
  const status = await getInferenceEngine().getStatus();
  return jsonResponse({
    object: "list",
    data: BROWSER_MODELS.map((spec) => ({
      id: spec.id,
      object: "model",
      created: 0,
      owned_by: "browser",
      permission: [],
      metadata: {
        cached: status.cached_model_ids.includes(spec.id),
        loaded: status.loaded_model_id === spec.id,
      },
    })),
  });
}

async function handleStatus(): Promise<Response> {
  return jsonResponse(await getInferenceEngine().getStatus());
}

async function parseJsonBody<T>(request: Request): Promise<T | null> {
  try {
    return (await request.json()) as T;
  } catch {
    return null;
  }
}

async function validateReadyModel(modelId: string): Promise<Response | null> {
  if (!getModelSpec(modelId)) {
    return errorResponse(404, `Unsupported browser model: ${modelId || "<empty>"}`);
  }

  if (!(await hasModel(modelId))) {
    return errorResponse(409, `Model is not cached: ${modelId}`);
  }

  return null;
}

async function downloadWithEngine(
  modelId: string,
  controller: ReadableStreamDefaultController<Uint8Array>,
): Promise<void> {
  const engine = getInferenceEngine();
  await engine.downloadModel(modelId, (status) => {
    enqueueSse(controller, ["event: message", `data: ${JSON.stringify(status)}`]);
  });
}

async function handleDownload(request: Request): Promise<Response> {
  const body = await parseJsonBody<{ model_id?: string }>(request);
  const modelId = String(body?.model_id ?? "").trim();
  if (!getModelSpec(modelId)) {
    return errorResponse(404, `Unknown browser model: ${modelId || "<empty>"}`);
  }

  return statusStreamResponse(async (controller) => {
    try {
      await downloadWithEngine(modelId, controller);
      const status = await getInferenceEngine().getStatus();
      enqueueSse(controller, ["event: message", `data: ${JSON.stringify(status)}`]);
      enqueueSse(controller, ["event: end", "data: {}"]);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      enqueueSse(controller, ["event: error", `data: ${JSON.stringify({ message })}`]);
    }
  });
}

function createChatCompletionChunk(
  requestId: string,
  modelId: string,
  delta: Record<string, unknown>,
  finishReason: string | null,
): string {
  return JSON.stringify({
    id: requestId,
    object: "chat.completion.chunk",
    created: nowSeconds(),
    model: modelId,
    choices: [
      {
        index: 0,
        delta,
        finish_reason: finishReason,
      },
    ],
  });
}

function createTextCompletionChunk(
  requestId: string,
  modelId: string,
  text: string,
  finishReason: string | null,
): string {
  return JSON.stringify({
    id: requestId,
    object: "text_completion",
    created: nowSeconds(),
    model: modelId,
    choices: [
      {
        index: 0,
        text,
        logprobs: null,
        finish_reason: finishReason,
      },
    ],
  });
}

async function handleChatCompletion(request: Request): Promise<Response> {
  const body = await parseJsonBody<OpenAIChatRequest>(request);
  const modelId = String(body?.model ?? "").trim();
  const messages = Array.isArray(body?.messages) ? body.messages : [];

  if (!messages.length) {
    return errorResponse(422, "messages must contain at least one entry");
  }

  const modelError = await validateReadyModel(modelId);
  if (modelError) {
    return modelError;
  }

  const chatMessages = toChatMessages(messages);
  const completionOptions = toChatOptions(body ?? {});
  const requestId = completionId("chatcmpl");
  const engine = getInferenceEngine();

  try {
    if (body?.stream) {
      const chunks = await engine.createChatCompletion(modelId, chatMessages, {
        ...completionOptions,
        stream: true,
      });

      return statusStreamResponse(async (controller) => {
        enqueueSse(controller, [`data: ${createChatCompletionChunk(requestId, modelId, { role: "assistant" }, null)}`]);

        for await (const chunk of chunks as AsyncIterable<CompletionChunk>) {
          const piece = decoder.decode(chunk.piece, { stream: true });
          if (!piece) {
            continue;
          }
          enqueueSse(controller, [`data: ${createChatCompletionChunk(requestId, modelId, { content: piece }, null)}`]);
        }

        enqueueSse(controller, [`data: ${createChatCompletionChunk(requestId, modelId, {}, "stop")}`, "data: [DONE]"]);
      });
    }

    const content = await engine.createChatCompletion(modelId, chatMessages, {
      ...completionOptions,
      stream: false,
    });

    return jsonResponse({
      id: requestId,
      object: "chat.completion",
      created: nowSeconds(),
      model: modelId,
      choices: [
        {
          index: 0,
          message: {
            role: "assistant",
            content,
          },
          finish_reason: "stop",
        },
      ],
    });
  } catch (error) {
    return errorResponse(500, error instanceof Error ? error.message : String(error));
  }
}

async function handleCompletion(request: Request): Promise<Response> {
  const body = await parseJsonBody<OpenAICompletionRequest>(request);
  const modelId = String(body?.model ?? "").trim();
  const prompt = normalizePrompt(body?.prompt);
  const modelError = await validateReadyModel(modelId);
  if (modelError) {
    return modelError;
  }

  const requestId = completionId("cmpl");
  const completionOptions = toChatOptions(body ?? {});
  const promptMessages = promptToMessages(prompt);
  const engine = getInferenceEngine();

  try {
    if (body?.stream) {
      const chunks = await engine.createChatCompletion(modelId, promptMessages, {
        ...completionOptions,
        stream: true,
      });

      return statusStreamResponse(async (controller) => {
        for await (const chunk of chunks as AsyncIterable<CompletionChunk>) {
          const piece = decoder.decode(chunk.piece, { stream: true });
          if (!piece) {
            continue;
          }
          enqueueSse(controller, [`data: ${createTextCompletionChunk(requestId, modelId, piece, null)}`]);
        }

        enqueueSse(controller, [`data: ${createTextCompletionChunk(requestId, modelId, "", "stop")}`, "data: [DONE]"]);
      });
    }

    const content = await engine.createChatCompletion(modelId, promptMessages, {
      ...completionOptions,
      stream: false,
    });

    return jsonResponse({
      id: requestId,
      object: "text_completion",
      created: nowSeconds(),
      model: modelId,
      choices: [
        {
          index: 0,
          text: content,
          logprobs: null,
          finish_reason: "stop",
        },
      ],
    });
  } catch (error) {
    return errorResponse(500, error instanceof Error ? error.message : String(error));
  }
}

export async function handleInferenceRoute(request: Request, route: InferenceRoute): Promise<Response> {
  switch (route) {
    case "chat-completions":
      return handleChatCompletion(request);
    case "completions":
      return handleCompletion(request);
    case "models-list":
      return handleModelsList();
    case "download-model":
      return handleDownload(request);
    case "inference-status":
      return handleStatus();
  }
}

export type { BrowserInferenceStatus };
