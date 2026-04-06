import { ChatAnthropic } from "@langchain/anthropic";
import { ChatGoogleGenerativeAI } from "@langchain/google-genai";
import type { BaseChatModel } from "@langchain/core/language_models/chat_models";
import { AIMessage, HumanMessage, SystemMessage, ToolMessage, type BaseMessage } from "@langchain/core/messages";
import { ChatOpenAI } from "@langchain/openai";
import type { ProtocolMessage } from "../run-store.js";

export interface AgentRunRequest {
  input?: unknown;
  message?: string;
  model_id?: string;
  messages?: ProtocolMessage[];
  metadata?: Record<string, unknown>;
  config?: Record<string, unknown>;
}

export function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function normalizeProtocolMessage(value: unknown, field = "messages"): ProtocolMessage {
  if (!isObject(value) || typeof value.role !== "string" || !("content" in value)) {
    throw new Error(`${field} must contain protocol messages`);
  }
  if (value.id !== undefined && typeof value.id !== "string") {
    throw new Error(`${field}.id must be a string`);
  }
  if (value.metadata !== undefined && !isObject(value.metadata)) {
    throw new Error(`${field}.metadata must be an object`);
  }
  return {
    ...value,
    role: value.role,
    content: value.content,
  };
}

function toLangChainContent(content: unknown): string | Array<Record<string, unknown>> {
  if (typeof content === "string") {
    return content;
  }
  if (Array.isArray(content)) {
    return content.map((entry) => (isObject(entry) ? { ...entry } : { type: "text", text: JSON.stringify(entry) }));
  }
  return JSON.stringify(content);
}

function toLangChainMessage(message: ProtocolMessage): BaseMessage {
  const role = message.role.trim().toLowerCase();
  const metadata = isObject(message.metadata) ? { ...message.metadata } : undefined;
  const baseFields = {
    content: toLangChainContent(message.content),
    ...(message.id ? { id: message.id } : {}),
    ...(metadata ? { additional_kwargs: metadata } : {}),
  };

  if (role === "assistant" || role === "ai") {
    return new AIMessage(baseFields);
  }
  if (role === "system") {
    return new SystemMessage(baseFields);
  }
  if (role === "developer") {
    return new SystemMessage({
      ...baseFields,
      additional_kwargs: {
        ...(baseFields.additional_kwargs ?? {}),
        __openai_role__: "developer",
      },
    });
  }
  if (role === "tool") {
    const toolCallId =
      typeof message.tool_call_id === "string"
        ? message.tool_call_id
        : typeof metadata?.tool_call_id === "string"
          ? metadata.tool_call_id
          : (message.id ?? crypto.randomUUID());
    return new ToolMessage({
      ...baseFields,
      tool_call_id: toolCallId,
      ...(typeof message.name === "string" ? { name: message.name } : {}),
    });
  }
  return new HumanMessage(baseFields);
}

export function invocationMessages(body: AgentRunRequest): BaseMessage[] {
  if (body.messages?.length) {
    return body.messages.map((message) => toLangChainMessage(message));
  }
  const content = body.input === undefined && typeof body.message === "string" ? body.message : body.input;
  return [new HumanMessage({ content: toLangChainContent(content) })];
}

export function modelIdFromRequest(body: AgentRunRequest): string | undefined {
  if (typeof body.model_id === "string" && body.model_id.trim()) {
    return body.model_id.trim();
  }
  if (isObject(body.config?.configurable) && typeof body.config.configurable.model_id === "string") {
    return body.config.configurable.model_id;
  }
  if (typeof body.config?.model_id === "string") {
    return body.config.model_id;
  }
  if (isObject(body.metadata) && typeof body.metadata.model_id === "string") {
    return body.metadata.model_id;
  }
  return undefined;
}

export function getModelInstance(modelId: string, apiKey?: string): BaseChatModel {
  if (modelId.startsWith("claude-")) {
    return new ChatAnthropic({ model: modelId, anthropicApiKey: apiKey, streaming: true });
  }
  if (modelId.startsWith("gpt-")) {
    return new ChatOpenAI({ model: modelId, openAIApiKey: apiKey, streaming: true });
  }
  if (modelId.startsWith("gemini-")) {
    return new ChatGoogleGenerativeAI({ model: modelId, apiKey, streaming: true });
  }
  return new ChatOpenAI({
    model: modelId,
    openAIApiKey: apiKey ?? "ollama",
    configuration: { baseURL: "http://localhost:11434/v1" },
    streaming: true,
  });
}

export function providerForModel(modelId: string): string {
  if (modelId.startsWith("claude-")) {
    return "anthropic";
  }
  if (modelId.startsWith("gpt-")) {
    return "openai";
  }
  if (modelId.startsWith("gemini-")) {
    return "google";
  }
  return "ollama";
}

export async function resolveAgentModel(baseUrl: string, body: AgentRunRequest): Promise<BaseChatModel> {
  const modelId = modelIdFromRequest(body) ?? (await getDefaultModel(baseUrl));
  const provider = providerForModel(modelId);
  const apiKey = await getProviderApiKey(baseUrl, provider);
  if (!apiKey && provider !== "ollama") {
    throw new Error(`No API key for ${provider}`);
  }
  return getModelInstance(modelId, apiKey || undefined);
}

async function getDefaultModel(baseUrl: string): Promise<string> {
  const payload = await getStoreItem(baseUrl, ["config"], "default_model");
  const value = isObject(payload?.value) ? payload.value : payload;
  if (isObject(value) && typeof value.model_id === "string" && value.model_id.trim()) {
    return value.model_id.trim();
  }
  if (typeof value === "string" && value.trim()) {
    return value.trim();
  }
  throw new Error("default_model is not configured");
}

async function getProviderApiKey(baseUrl: string, provider: string): Promise<string | null> {
  const payload = await getStoreItem(baseUrl, ["config", "api-keys"], provider);
  const value = payload?.value;
  if (typeof value === "string" && value.trim()) {
    return value.trim();
  }
  return null;
}

async function getStoreItem(
  baseUrl: string,
  namespace: string[],
  key: string,
): Promise<Record<string, unknown> | null> {
  const url = new URL(`${baseUrl}/store/items`);
  for (const segment of namespace) {
    url.searchParams.append("namespace", segment);
  }
  url.searchParams.set("key", key);
  const response = await fetch(url);
  if (response.status === 404) {
    return null;
  }
  if (!response.ok) {
    throw new Error(await response.text().catch(() => `store item request failed for ${key}`));
  }
  const payload = (await response.json()) as Record<string, unknown>;
  return isObject(payload) ? payload : null;
}
