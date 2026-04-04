import { ChatAnthropic } from "@langchain/anthropic";
import { ChatGoogleGenerativeAI } from "@langchain/google-genai";
import type { BaseChatModel } from "@langchain/core/language_models/chat_models";
import { AIMessage, HumanMessage, SystemMessage, ToolMessage, type BaseMessage } from "@langchain/core/messages";
import { ChatOpenAI } from "@langchain/openai";
import { createDeepAgent } from "deepagents";
import { SqlJsSaver } from "../checkpointer.js";
import {
  createThread as createDeepagentsThread,
  deleteThread as deleteDeepagentsThread,
  deleteThreadMessages,
  getThread as getDeepagentsThread,
  listMessages as listDeepagentsMessages,
  patchThread as patchDeepagentsThread,
  saveMessage as saveDeepagentsMessage,
  saveThread as saveDeepagentsThread,
  setThreadStatus as setDeepagentsThreadStatus,
} from "../deepagents.js";
import {
  createRunRecord,
  deleteRunRecord,
  getRunRecord,
  saveRunRecord,
  searchRunRecords,
  type PersistedRun,
  type PersistedRunStatus,
  type PersistedStreamMode,
  type ProtocolMessage,
  type StoredSseEvent,
} from "../run-store.js";
import {
  deriveThreadTitle,
  getApiKey as getProviderApiKey,
  getDefaultModel,
  isPlaceholderThreadTitle,
} from "../store-data.js";
import {
  exists as zenExists,
  fs,
  init as initZenfs,
  readFile as zenReadFile,
  rm as zenRm,
  writeFile as zenWriteFile,
} from "../zenfs.js";
import initBashkit, { execute as bashkitExecute } from "./omni-bashkit.js";
import { BashkitSandboxBackend } from "./sandbox.js";
import { buildSystemPrompt } from "./system-prompt.js";

const CHECKPOINTS_DIR = "/home/checkpoints";
const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const RUN_STATUSES = new Set<PersistedRunStatus>(["pending", "error", "success", "timeout", "interrupted"]);
const STREAM_MODES = new Set<PersistedStreamMode>(["values", "messages", "updates", "custom"]);
const SUPPORTED_STREAM_MODES = new Set<PersistedStreamMode>(["values", "messages"]);
const DEFAULT_AGENT_ID = "deepagent";
const ROUTE_ROOTS = new Set(["agents", "threads", "store", "x", "runs"]);

interface RunRequest {
  thread_id?: string;
  input?: unknown;
  message?: string;
  model_id?: string;
  agent_id?: string;
  messages?: ProtocolMessage[];
  metadata?: Record<string, unknown>;
  config?: Record<string, unknown>;
  webhook?: string;
  on_completion?: "delete" | "keep";
  on_disconnect?: "cancel" | "continue";
  if_not_exists?: "create" | "reject";
  stream_mode?: PersistedStreamMode | PersistedStreamMode[];
  status?: string;
  offset?: number;
  limit?: number;
}

type RunRoute =
  | "runs-create"
  | "runs-search"
  | "runs-stream"
  | "runs-wait"
  | "run-get"
  | "run-delete"
  | "run-wait"
  | "run-stream"
  | "run-cancel";

type ActiveRun = {
  record: PersistedRun;
  abortController: AbortController;
  subscribers: Set<(frame: StoredSseEvent) => void>;
  completion: Promise<PersistedRun>;
  completed: boolean;
  createdThread: boolean;
  rollbackSnapshot?: RunRollbackSnapshot;
};

type RunRollbackSnapshot = {
  threadId: string;
  threadRecord?: Record<string, unknown>;
  messages: Array<Record<string, unknown>>;
  checkpointFile?: Uint8Array;
};

type BashkitExecute = (
  command: string,
  cwd?: string,
) => Promise<{
  output: string;
  exitCode: number;
  truncated: boolean;
}>;

class HttpError extends Error {
  constructor(
    readonly status: number,
    message: string,
    readonly code?: string,
    readonly metadata?: Record<string, unknown>,
  ) {
    super(message);
  }
}

const ACTIVE_RUNS = new Map<string, ActiveRun>();
let bashkitReadyPromise: Promise<void> | null = null;

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function errorResponse(status: number, message: string, code?: string, metadata?: Record<string, unknown>): Response {
  return Response.json(
    {
      ...(code ? { code } : {}),
      message,
      ...(metadata ? { metadata } : {}),
    },
    { status },
  );
}

function validateUuid(value: string, field: string): string {
  const normalized = value.trim();
  if (!UUID_PATTERN.test(normalized)) {
    throw new HttpError(422, `${field} must be a valid UUID`, "validation_error", { field });
  }
  return normalized;
}

function routeParts(request: Request): string[] {
  const parts = new URL(request.url).pathname.split("/").filter(Boolean);
  const rootIndex = parts.findIndex((part) => ROUTE_ROOTS.has(part));
  return rootIndex >= 0 ? parts.slice(rootIndex) : parts;
}

function validatePlainObject(raw: unknown, field: string): Record<string, unknown> | undefined {
  if (raw === undefined) {
    return undefined;
  }
  if (!isObject(raw)) {
    throw new HttpError(422, `${field} must be an object`, "validation_error", { field });
  }
  return raw;
}

function normalizeProtocolMessage(value: unknown, field = "messages"): ProtocolMessage {
  if (!isObject(value) || typeof value.role !== "string" || !("content" in value)) {
    throw new HttpError(422, `${field} must contain protocol messages`, "validation_error", { field });
  }
  if (value.id !== undefined && typeof value.id !== "string") {
    throw new HttpError(422, `${field}.id must be a string`, "validation_error", { field: `${field}.id` });
  }
  if (value.metadata !== undefined && !isObject(value.metadata)) {
    throw new HttpError(422, `${field}.metadata must be an object`, "validation_error", { field: `${field}.metadata` });
  }
  return {
    ...value,
    role: value.role,
    content: value.content,
  };
}

function normalizeMessages(value: unknown, field = "messages"): ProtocolMessage[] | undefined {
  if (value === undefined) {
    return undefined;
  }
  if (!Array.isArray(value)) {
    throw new HttpError(422, `${field} must be an array`, "validation_error", { field });
  }
  return value.map((entry) => normalizeProtocolMessage(entry, field));
}

function parseLimit(raw: unknown): number | undefined {
  if (raw === undefined) {
    return undefined;
  }
  const value = Number(raw);
  if (!Number.isInteger(value) || value < 1 || value > 1000) {
    throw new HttpError(422, "limit must be an integer between 1 and 1000", "validation_error", { field: "limit" });
  }
  return value;
}

function parseOffset(raw: unknown): number | undefined {
  if (raw === undefined) {
    return undefined;
  }
  const value = Number(raw);
  if (!Number.isInteger(value) || value < 0) {
    throw new HttpError(422, "offset must be a non-negative integer", "validation_error", { field: "offset" });
  }
  return value;
}

function parseRunStatus(raw: unknown): PersistedRunStatus | undefined {
  if (raw === undefined) {
    return undefined;
  }
  const value = String(raw).trim().toLowerCase() as PersistedRunStatus;
  if (!RUN_STATUSES.has(value)) {
    throw new HttpError(
      422,
      "status must be one of pending, error, success, timeout, interrupted",
      "validation_error",
      {
        field: "status",
      },
    );
  }
  return value;
}

function normalizeStreamModes(raw: unknown): PersistedStreamMode | PersistedStreamMode[] {
  if (raw === undefined) {
    return "values";
  }
  const modes = Array.isArray(raw) ? raw : [raw];
  if (modes.length === 0) {
    throw new HttpError(422, "stream_mode must not be empty", "validation_error", { field: "stream_mode" });
  }
  const normalized = modes.map((mode) => {
    const value = String(mode).trim().toLowerCase() as PersistedStreamMode;
    if (!STREAM_MODES.has(value)) {
      throw new HttpError(422, `unsupported stream_mode ${String(mode)}`, "validation_error", { field: "stream_mode" });
    }
    if (!SUPPORTED_STREAM_MODES.has(value)) {
      throw new HttpError(422, `stream_mode ${value} is not supported`, "validation_error", { field: "stream_mode" });
    }
    return value;
  });
  const deduped = [...new Set(normalized)];
  return Array.isArray(raw) ? deduped : deduped[0];
}

function resolvedStreamModes(value: PersistedStreamMode | PersistedStreamMode[] | undefined): PersistedStreamMode[] {
  if (!value) {
    return ["values"];
  }
  return Array.isArray(value) ? value : [value];
}

function normalizeRunRequest(body: Record<string, unknown>, includeStreamMode: boolean): RunRequest {
  const threadId =
    typeof body.thread_id === "string" && body.thread_id.trim() ? validateUuid(body.thread_id, "thread_id") : undefined;
  const agentId = body.agent_id === undefined ? undefined : String(body.agent_id).trim();
  if (agentId && agentId !== DEFAULT_AGENT_ID) {
    throw new HttpError(404, "Agent not found", "agent_not_found", { agent_id: agentId });
  }
  const webhook = body.webhook === undefined ? undefined : String(body.webhook);
  if (webhook) {
    try {
      new URL(webhook);
    } catch {
      throw new HttpError(422, "webhook must be a valid URI", "validation_error", { field: "webhook" });
    }
  }
  const onCompletion =
    body.on_completion === undefined ? (threadId ? "keep" : "delete") : String(body.on_completion).toLowerCase();
  if (!["delete", "keep"].includes(onCompletion)) {
    throw new HttpError(422, "on_completion must be delete or keep", "validation_error", { field: "on_completion" });
  }
  const onDisconnect = body.on_disconnect === undefined ? "cancel" : String(body.on_disconnect).toLowerCase();
  if (!["cancel", "continue"].includes(onDisconnect)) {
    throw new HttpError(422, "on_disconnect must be cancel or continue", "validation_error", {
      field: "on_disconnect",
    });
  }
  const ifNotExists = body.if_not_exists === undefined ? "reject" : String(body.if_not_exists).toLowerCase();
  if (!["create", "reject"].includes(ifNotExists)) {
    throw new HttpError(422, "if_not_exists must be create or reject", "validation_error", { field: "if_not_exists" });
  }
  const metadata = validatePlainObject(body.metadata, "metadata") ?? {};
  const config = validatePlainObject(body.config, "config");
  const messages = normalizeMessages(body.messages);
  const input = body.input === undefined && typeof body.message === "string" ? body.message : body.input;
  if (messages === undefined && input === undefined) {
    throw new HttpError(422, "input or messages is required", "validation_error");
  }

  return {
    ...(threadId ? { thread_id: threadId } : {}),
    ...(agentId ? { agent_id: agentId } : {}),
    ...(input === undefined ? {} : { input }),
    ...(messages?.length ? { messages } : {}),
    metadata,
    ...(config ? { config } : {}),
    ...(webhook ? { webhook } : {}),
    on_completion: onCompletion as "delete" | "keep",
    on_disconnect: onDisconnect as "cancel" | "continue",
    if_not_exists: ifNotExists as "create" | "reject",
    ...(includeStreamMode ? { stream_mode: normalizeStreamModes(body.stream_mode) } : {}),
  };
}

async function parseRunRequest(request: Request, includeStreamMode: boolean): Promise<RunRequest> {
  const body = await request.json().catch(() => {
    throw new HttpError(422, "Request body must be valid JSON", "validation_error");
  });
  if (!isObject(body)) {
    throw new HttpError(422, "Request body must be a JSON object", "validation_error");
  }
  return normalizeRunRequest(body, includeStreamMode);
}

function normalizeSearchRequest(body: Record<string, unknown>) {
  return {
    metadata: validatePlainObject(body.metadata, "metadata"),
    status: parseRunStatus(body.status),
    thread_id:
      typeof body.thread_id === "string" && body.thread_id.trim()
        ? validateUuid(body.thread_id, "thread_id")
        : undefined,
    agent_id: body.agent_id === undefined ? undefined : String(body.agent_id).trim(),
    offset: parseOffset(body.offset),
    limit: parseLimit(body.limit),
  };
}

async function ensureZenFs(): Promise<void> {
  await initZenfs();
}

async function readThreadRecord(threadId: string): Promise<Record<string, unknown> | null> {
  const value = await getDeepagentsThread(threadId);
  return isObject(value) ? value : null;
}

async function createThreadRecord(
  threadId: string,
  metadata?: Record<string, unknown>,
): Promise<Record<string, unknown>> {
  const thread = await createDeepagentsThread({
    thread_id: threadId,
    metadata: metadata ?? {},
  });
  return thread as Record<string, unknown>;
}

async function getWorkspacePath(threadId?: string): Promise<string> {
  if (!threadId) {
    return "/home/workspace";
  }
  const thread = await readThreadRecord(threadId);
  const metadata = thread && isObject(thread.metadata) ? thread.metadata : undefined;
  return typeof metadata?.workspace === "string" && metadata.workspace ? metadata.workspace : "/home/workspace";
}

function messageContentToString(content: unknown): string {
  if (typeof content === "string") {
    return content;
  }
  return JSON.stringify(content);
}

function toLangChainContent(content: unknown): string | Array<Record<string, unknown>> {
  if (typeof content === "string") {
    return content;
  }
  if (Array.isArray(content)) {
    return content.map((entry) =>
      isObject(entry) ? { ...entry } : { type: "text", text: messageContentToString(entry) },
    );
  }
  return messageContentToString(content);
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

function invocationMessages(body: RunRequest): BaseMessage[] {
  if (body.messages?.length) {
    return body.messages.map((message) => toLangChainMessage(message));
  }
  return [new HumanMessage({ content: toLangChainContent(body.input) })];
}

function modelIdFromRequest(body: RunRequest): string | undefined {
  if (typeof body.model_id === "string" && body.model_id.trim()) {
    return body.model_id.trim();
  }
  if (isObject(body.config?.configurable) && typeof body.config?.configurable.model_id === "string") {
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

function providerForModel(modelId: string): string {
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

function sseChunk(frame: StoredSseEvent): Uint8Array {
  const idLine = frame.id ? `id: ${frame.id}\n` : "";
  return new TextEncoder().encode(`${idLine}event: ${frame.event}\ndata: ${JSON.stringify(frame.data)}\n\n`);
}

async function loadBashkitExecutor(): Promise<BashkitExecute> {
  if (!bashkitReadyPromise) {
    bashkitReadyPromise = initBashkit();
  }
  await bashkitReadyPromise;
  return bashkitExecute as BashkitExecute;
}

async function executeWithBashkit(command: string, cwd: string) {
  try {
    const execute = await loadBashkitExecutor();
    return await execute(command, cwd);
  } catch (error) {
    return {
      output: `Error: ${String(error)}`,
      exitCode: 1,
      truncated: false,
    };
  }
}

async function setThreadStatus(threadId: string | undefined, status: string): Promise<void> {
  if (!threadId) {
    return;
  }
  await setDeepagentsThreadStatus(threadId, status);
}

async function appendThreadMessage(threadId: string | undefined, message: ProtocolMessage): Promise<void> {
  if (!threadId) {
    return;
  }
  const id = message.id ?? crypto.randomUUID();
  const createdAt = new Date().toISOString();
  await saveDeepagentsMessage(threadId, createdAt, {
    ...message,
    id,
  });
}

async function captureRollbackSnapshot(threadId: string | undefined): Promise<RunRollbackSnapshot | undefined> {
  if (!threadId) {
    return undefined;
  }
  await ensureZenFs();
  const snapshot: RunRollbackSnapshot = {
    threadId,
    messages: [],
  };
  const thread = await getDeepagentsThread(threadId);
  if (isObject(thread)) {
    snapshot.threadRecord = thread;
  }
  snapshot.messages = (await listDeepagentsMessages(threadId)).filter(isObject);
  const checkpointPath = `${CHECKPOINTS_DIR}/${threadId}.sqlite`;
  if (await zenExists(checkpointPath).catch(() => false)) {
    snapshot.checkpointFile = await zenReadFile(checkpointPath);
  }
  return snapshot;
}

function sanitizeCheckpointValues(values: unknown): Record<string, unknown> | undefined {
  if (!isObject(values)) {
    return undefined;
  }
  const next = { ...values };
  if (Array.isArray(next.messages)) {
    delete next.messages;
  }
  return next;
}

function extractCheckpointMessages(values: unknown): ProtocolMessage[] | undefined {
  if (!isObject(values) || !Array.isArray(values.messages)) {
    return undefined;
  }
  return values.messages.map((message) => normalizeProtocolMessage(message));
}

async function syncThreadStateFromCheckpoint(threadId: string | undefined): Promise<{
  values?: Record<string, unknown>;
  messages?: ProtocolMessage[];
}> {
  if (!threadId) {
    return {};
  }
  const saver = new SqlJsSaver();
  const tuple = await saver.getTuple({ configurable: { thread_id: threadId } });
  if (!tuple) {
    return {};
  }
  const checkpoint = tuple.checkpoint as Record<string, unknown>;
  const channelValues = isObject(checkpoint.channel_values) ? checkpoint.channel_values : checkpoint;
  const values = sanitizeCheckpointValues(channelValues);
  const messages = extractCheckpointMessages(channelValues);
  const thread = await readThreadRecord(threadId);
  if (thread) {
    await patchDeepagentsThread(threadId, {
      ...(values ? { values } : {}),
      ...(messages?.length ? { messages } : {}),
    });
  }
  return { values, messages };
}

async function emitRunEvent(active: ActiveRun, event: string, data: unknown): Promise<void> {
  const frame = {
    id: String((active.record.events?.length ?? 0) + 1),
    event,
    data,
  };
  active.record.events = [...(active.record.events ?? []), frame];
  for (const subscriber of active.subscribers) {
    subscriber(frame);
  }
}

async function persistRun(active: ActiveRun): Promise<void> {
  active.record.run.updated_at = new Date().toISOString();
  await saveRunRecord(active.record);
}

function runWaitResponse(record: PersistedRun): Record<string, unknown> {
  return {
    run: record.run,
    ...(record.values === undefined ? {} : { values: record.values }),
    ...(record.messages === undefined ? {} : { messages: record.messages }),
  };
}

function streamResponseForRecord(
  record: PersistedRun,
  active: ActiveRun | undefined,
  replayExisting: boolean,
): Response {
  let subscriber: ((frame: StoredSseEvent) => void) | undefined;
  const stream = new ReadableStream({
    start(controller) {
      const push = (frame: StoredSseEvent) => controller.enqueue(sseChunk(frame));
      if (replayExisting) {
        for (const frame of record.events ?? []) {
          push(frame);
        }
      }

      if (!active || active.completed) {
        controller.close();
        return;
      }

      subscriber = (frame: StoredSseEvent) => {
        push(frame);
        if (frame.event === "end") {
          if (subscriber) {
            active.subscribers.delete(subscriber);
          }
          controller.close();
        }
      };
      active.subscribers.add(subscriber);
    },
    cancel() {
      if (!active || !subscriber) {
        return;
      }
      active.subscribers.delete(subscriber);
      if (!active.completed && active.record.run.on_disconnect === "cancel" && active.subscribers.size === 0) {
        active.abortController.abort();
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

async function invokeWebhook(record: PersistedRun): Promise<void> {
  if (!record.run.webhook) {
    return;
  }
  try {
    await fetch(record.run.webhook, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(runWaitResponse(record)),
    });
  } catch {
    // ignore webhook delivery failures
  }
}

async function rollbackRunArtifacts(record: PersistedRun): Promise<void> {
  await deleteRunRecord(record.run.run_id);
  const active = ACTIVE_RUNS.get(record.run.run_id);
  const snapshot = active?.rollbackSnapshot;
  if (!record.run.thread_id) {
    return;
  }

  await ensureZenFs();
  await deleteThreadMessages(record.run.thread_id).catch(() => {});
  if (snapshot?.threadRecord) {
    await saveDeepagentsThread(snapshot.threadRecord).catch(() => {});
  } else {
    await deleteDeepagentsThread(record.run.thread_id).catch(() => {});
  }
  for (const message of snapshot?.messages ?? []) {
    const createdAt = typeof message.created_at === "string" ? message.created_at : new Date().toISOString();
    await saveDeepagentsMessage(record.run.thread_id, createdAt, message).catch(() => {});
  }
  const checkpointPath = `${CHECKPOINTS_DIR}/${record.run.thread_id}.sqlite`;
  if (snapshot?.checkpointFile) {
    await zenWriteFile(checkpointPath, snapshot.checkpointFile).catch(() => {});
  } else {
    await zenRm(checkpointPath, { recursive: false }).catch(() => {});
  }
}

async function completeRunSideEffects(active: ActiveRun): Promise<void> {
  await invokeWebhook(active.record);
  if (active.record.run.on_completion === "delete" && active.record.run.thread_id) {
    try {
      await deleteDeepagentsThread(active.record.run.thread_id);
    } catch {
      // ignore
    }
    try {
      await deleteThreadMessages(active.record.run.thread_id);
    } catch {
      // ignore
    }
    try {
      await fs.promises.rm(`${CHECKPOINTS_DIR}/${active.record.run.thread_id}.sqlite`);
    } catch {
      // ignore
    }
  }
}

async function executeRun(active: ActiveRun, body: RunRequest): Promise<PersistedRun> {
  const threadId = body.thread_id;
  const streamModes = resolvedStreamModes(active.record.run.stream_mode);

  try {
    await setThreadStatus(threadId, "busy");
    await persistRun(active);

    const inputMessages = invocationMessages(body);
    const echoedInputMessages =
      body.messages ?? (body.input === undefined ? undefined : [{ role: "user", content: body.input }]);
    const currentThread = threadId ? await readThreadRecord(threadId) : null;
    const currentTitle = currentThread && isObject(currentThread.metadata) ? currentThread.metadata.title : undefined;
    if (threadId && echoedInputMessages?.length && isPlaceholderThreadTitle(currentTitle)) {
      const nextTitle = deriveThreadTitle(echoedInputMessages, currentTitle);
      if (nextTitle !== String(currentTitle ?? "").trim()) {
        await patchDeepagentsThread(threadId, {
          metadata: { title: nextTitle },
        });
      }
    }
    if (echoedInputMessages?.length) {
      for (const message of echoedInputMessages) {
        await appendThreadMessage(threadId, message);
      }
    }

    const workspacePath = await getWorkspacePath(threadId);
    const modelId = modelIdFromRequest(body) ?? (await getDefaultModel());
    const provider = providerForModel(modelId);
    const apiKey = await getProviderApiKey(provider);

    if (!apiKey && provider !== "ollama") {
      throw new Error(`No API key for ${provider}`);
    }

    const model = getModelInstance(modelId, apiKey || undefined);
    const sandbox = new BashkitSandboxBackend(workspacePath, executeWithBashkit);
    const checkpointer = new SqlJsSaver();
    const systemPrompt = buildSystemPrompt(workspacePath);
    const workflow = createDeepAgent({
      model,
      backend: sandbox,
      checkpointer,
      systemPrompt,
    });

    const stream = await workflow.stream(
      { messages: inputMessages },
      {
        ...(threadId ? { configurable: { thread_id: threadId } } : {}),
        streamMode: "messages",
      },
    );

    let assistantContent = "";
    for await (const [msgChunk] of stream) {
      if (active.abortController.signal.aborted) {
        throw new Error("Run cancelled");
      }

      if ("content" in msgChunk && typeof msgChunk.content === "string" && msgChunk.content) {
        assistantContent += msgChunk.content;
        if (streamModes.includes("messages")) {
          await emitRunEvent(active, "message", { role: "assistant", content: msgChunk.content });
        }
      }
    }

    const finalMessage: ProtocolMessage = {
      id: crypto.randomUUID(),
      role: "assistant",
      content: assistantContent,
    };
    const checkpointState = await syncThreadStateFromCheckpoint(threadId);

    active.record.run.status = "success";
    active.record.values = checkpointState.values ?? { output: assistantContent };
    active.record.messages = checkpointState.messages?.length ? checkpointState.messages : [finalMessage];
    await appendThreadMessage(threadId, finalMessage);
    await setThreadStatus(threadId, "idle");

    if (streamModes.includes("values")) {
      await emitRunEvent(active, "values", active.record.values);
    }
    if (streamModes.includes("messages")) {
      await emitRunEvent(active, "messages/complete", active.record.messages);
    }
    await emitRunEvent(active, "end", null);
    await persistRun(active);
    active.completed = true;
    await completeRunSideEffects(active);
    return active.record;
  } catch (error) {
    const messageText = error instanceof Error ? error.message : String(error);
    const interrupted = active.abortController.signal.aborted || messageText === "Run cancelled";
    active.record.run.status = interrupted ? "interrupted" : "error";
    active.record.error = messageText;
    await setThreadStatus(threadId, interrupted ? "interrupted" : "error");
    await emitRunEvent(active, "error", { message: messageText });
    await emitRunEvent(active, "end", null);
    await persistRun(active);
    active.completed = true;
    await completeRunSideEffects(active);
    return active.record;
  }
}

async function ensureThreadForRun(body: RunRequest): Promise<{ threadId?: string; createdThread: boolean }> {
  if (!body.thread_id) {
    return { createdThread: false };
  }
  const current = await readThreadRecord(body.thread_id);
  if (current) {
    return { threadId: body.thread_id, createdThread: false };
  }
  if (body.if_not_exists === "create") {
    await createThreadRecord(body.thread_id, body.metadata);
    return { threadId: body.thread_id, createdThread: true };
  }
  throw new HttpError(404, "Thread not found", "thread_not_found", { thread_id: body.thread_id });
}

async function startRun(body: RunRequest): Promise<ActiveRun> {
  const { threadId, createdThread } = await ensureThreadForRun(body);
  const record = createRunRecord({
    threadId,
    agentId: body.agent_id,
    input: body.input,
    inputMessages: body.messages,
    metadata: body.metadata,
    config: body.config,
    webhook: body.webhook,
    onCompletion: body.on_completion,
    onDisconnect: body.on_disconnect,
    ifNotExists: body.if_not_exists,
    streamMode: body.stream_mode,
  });
  const active: ActiveRun = {
    record,
    abortController: new AbortController(),
    subscribers: new Set(),
    completion: Promise.resolve(record),
    completed: false,
    createdThread,
    rollbackSnapshot: createdThread
      ? threadId
        ? {
            threadId,
            messageFiles: [],
          }
        : undefined
      : await captureRollbackSnapshot(threadId),
  };

  ACTIVE_RUNS.set(record.run.run_id, active);
  await saveRunRecord(record);
  active.completion = executeRun(active, { ...body, ...(threadId ? { thread_id: threadId } : {}) });
  return active;
}

function getRunIdFromRequest(request: Request): string {
  const runId = routeParts(request)[1] ?? "";
  return validateUuid(runId, "run_id");
}

async function getKnownRun(runId: string): Promise<{ active?: ActiveRun; record: PersistedRun | null }> {
  const active = ACTIVE_RUNS.get(runId);
  if (active) {
    return { active, record: active.record };
  }
  return { record: await getRunRecord(runId) };
}

export async function handleRunRoute(request: Request, route: RunRoute): Promise<Response> {
  try {
    if (route === "runs-create") {
      const active = await startRun(await parseRunRequest(request, true));
      if (active.record.run.on_disconnect === "cancel") {
        request.signal.addEventListener("abort", () => active.abortController.abort(), { once: true });
      }
      return Response.json(active.record.run);
    }

    if (route === "runs-search") {
      const body = await request.json().catch(() => {
        throw new HttpError(422, "Request body must be valid JSON", "validation_error");
      });
      if (!isObject(body)) {
        throw new HttpError(422, "Request body must be a JSON object", "validation_error");
      }
      const search = normalizeSearchRequest(body);
      const runs = await searchRunRecords(search);
      return Response.json(runs.map((record) => record.run));
    }

    if (route === "runs-wait") {
      const active = await startRun(await parseRunRequest(request, false));
      if (active.record.run.on_disconnect === "cancel") {
        request.signal.addEventListener("abort", () => active.abortController.abort(), { once: true });
      }
      return Response.json(runWaitResponse(await active.completion));
    }

    if (route === "runs-stream") {
      const active = await startRun(await parseRunRequest(request, true));
      if (active.record.run.on_disconnect === "cancel") {
        request.signal.addEventListener("abort", () => active.abortController.abort(), { once: true });
      }
      return streamResponseForRecord(active.record, active, true);
    }

    const runId = getRunIdFromRequest(request);
    const { active, record } = await getKnownRun(runId);
    if (!record) {
      return errorResponse(404, "Run not found", "run_not_found", { run_id: runId });
    }

    if (route === "run-get") {
      return Response.json(record.run);
    }

    if (route === "run-delete") {
      if (active && !active.completed) {
        active.abortController.abort();
        await active.completion;
      }
      ACTIVE_RUNS.delete(runId);
      await deleteRunRecord(runId);
      return new Response(null, { status: 204 });
    }

    if (route === "run-wait") {
      return Response.json(runWaitResponse(active ? await active.completion : record));
    }

    if (route === "run-stream") {
      if (active && active.record.run.on_disconnect === "cancel") {
        request.signal.addEventListener("abort", () => active.abortController.abort(), { once: true });
      }
      return streamResponseForRecord(record, active, true);
    }

    if (route === "run-cancel") {
      const url = new URL(request.url);
      const wait = url.searchParams.get("wait") === "true";
      const action = (url.searchParams.get("action") ?? "interrupt").toLowerCase();
      if (!["interrupt", "rollback"].includes(action)) {
        throw new HttpError(422, "action must be interrupt or rollback", "validation_error", { field: "action" });
      }

      if (active && !active.completed) {
        active.abortController.abort();
        if (wait) {
          await active.completion;
        }
      }

      if (action === "rollback") {
        await rollbackRunArtifacts(record);
      }
      return new Response(null, { status: 204 });
    }

    return errorResponse(404, "Route not found", "not_found");
  } catch (err) {
    if (err instanceof HttpError) {
      return errorResponse(err.status, err.message, err.code, err.metadata);
    }
    const message = err instanceof Error ? err.message : String(err);
    return errorResponse(500, message, "internal_error");
  }
}
