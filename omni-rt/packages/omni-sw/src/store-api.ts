import {
  buildBootstrap,
  deleteApiKey,
  deleteDefaultModel,
  deleteThread,
  getApiKey,
  getStoredDefaultModel,
  listWorkspaceFiles,
  readProvidersWithKeys,
  setApiKey,
  setDefaultModel,
} from "./store-data.js";
import {
  createThread as createRustThread,
  getThread as getRustThread,
  listMessages as listRustMessages,
  listThreads as listRustThreads,
  patchThread as patchRustThread,
  saveMessage as saveRustMessage,
  setThreadStatus as setRustThreadStatus,
} from "./deepagents.js";
import { SqlJsSaver } from "./checkpointer.js";
import {
  exists as zenExists,
  init as zenInit,
  mkdir as zenMkdir,
  readFile as zenReadFile,
  readdir as zenReadDir,
  rm as zenRm,
  writeFile as zenWriteFile,
} from "./zenfs.js";

const TODOS_DIR = "/home/db/todos";
const SUBAGENTS_DIR = "/home/db/subagents";
const CHECKPOINTS_DIR = "/home/checkpoints";
const STORE_DIR = "/home/store";
const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const THREAD_STATUSES = new Set(["idle", "busy", "interrupted", "error"]);
const DEFAULT_LIMIT = 10;
const DEFAULT_NAMESPACE_LIMIT = 100;

const encoder = new TextEncoder();
const decoder = new TextDecoder();

const DEEP_AGENT = {
  agent_id: "deepagent",
  name: "DeepAgent",
  description: "Local DeepAgent runtime backed by the service worker.",
  metadata: {
    provider: "omni",
    runtime: "service-worker",
  },
  capabilities: {
    "ap.io.messages": true,
    "ap.io.streaming": true,
  },
};

const DEEP_AGENT_SCHEMA = {
  agent_id: "deepagent",
  input_schema: {
    type: "object",
    properties: {
      thread_id: { type: "string", format: "uuid" },
      agent_id: { type: "string" },
      input: {},
      messages: {
        type: "array",
        items: {
          type: "object",
          properties: {
            role: { type: "string" },
            content: {},
            id: { type: "string" },
            metadata: { type: "object" },
          },
          required: ["role", "content"],
          additionalProperties: true,
        },
      },
      metadata: { type: "object" },
      config: {
        type: "object",
        properties: {
          tags: { type: "array", items: { type: "string" } },
          recursion_limit: { type: "integer" },
          configurable: { type: "object" },
        },
      },
      webhook: { type: "string", format: "uri" },
      on_completion: { type: "string", enum: ["delete", "keep"] },
      on_disconnect: { type: "string", enum: ["cancel", "continue"] },
      if_not_exists: { type: "string", enum: ["create", "reject"] },
      stream_mode: {
        anyOf: [
          { type: "string", enum: ["values", "messages", "updates", "custom"] },
          {
            type: "array",
            items: { type: "string", enum: ["values", "messages", "updates", "custom"] },
          },
        ],
      },
    },
  },
  output_schema: {
    type: "object",
    properties: {
      values: { type: "object" },
      messages: {
        type: "array",
        items: {
          type: "object",
          properties: {
            role: { type: "string" },
            content: {},
            id: { type: "string" },
            metadata: { type: "object" },
          },
          required: ["role", "content"],
          additionalProperties: true,
        },
      },
    },
  },
  state_schema: {
    type: "object",
  },
  config_schema: {
    type: "object",
    properties: {
      tags: { type: "array", items: { type: "string" } },
      recursion_limit: { type: "integer" },
      configurable: { type: "object" },
    },
  },
};

export type StoreRoute =
  | "agents-search"
  | "agents-get"
  | "agents-schemas"
  | "threads-create"
  | "threads-search"
  | "threads-get"
  | "threads-patch"
  | "threads-delete"
  | "threads-history"
  | "threads-copy"
  | "store-items-get"
  | "store-items-put"
  | "store-items-delete"
  | "store-items-search"
  | "store-namespaces"
  | "x-bootstrap"
  | "x-providers"
  | "x-files"
  | null;

type ProtocolMessage = {
  role: string;
  content: unknown;
  id?: string;
  metadata?: Record<string, unknown>;
  [key: string]: unknown;
};

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

let zenFsReady: Promise<void> | null = null;

async function ensureZenFs(): Promise<void> {
  if (!zenFsReady) {
    zenFsReady = zenInit().catch(() => {});
  }
  await zenFsReady;
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

function parsePath(request: Request): string[] {
  return new URL(request.url).pathname.split("/").filter(Boolean);
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function jsonEquals(left: unknown, right: unknown): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

function validateUuid(value: string, field: string): string {
  const normalized = value.trim();
  if (!UUID_PATTERN.test(normalized)) {
    throw new HttpError(422, `${field} must be a valid UUID`, "validation_error", { field });
  }
  return normalized;
}

function normalizeStatus(raw: unknown): string {
  const value =
    String(raw ?? "idle")
      .trim()
      .toLowerCase() || "idle";
  return THREAD_STATUSES.has(value) ? value : "idle";
}

function validateStatus(raw: unknown): string | undefined {
  if (raw === undefined) {
    return undefined;
  }
  const value = String(raw).trim().toLowerCase();
  if (!THREAD_STATUSES.has(value)) {
    throw new HttpError(422, "status must be one of idle, busy, interrupted, error", "validation_error", {
      field: "status",
    });
  }
  return value;
}

function parseOptionalArrayOfStrings(raw: unknown, field: string): string[] | undefined {
  if (raw === undefined) {
    return undefined;
  }
  if (!Array.isArray(raw) || raw.some((value) => typeof value !== "string")) {
    throw new HttpError(422, `${field} must be an array of strings`, "validation_error", { field });
  }
  return raw.map((value) => value.trim());
}

function parseLimit(raw: unknown, fallback: number): number {
  if (raw === undefined) {
    return fallback;
  }
  const value = Number(raw);
  if (!Number.isInteger(value) || value < 1 || value > 1000) {
    throw new HttpError(422, "limit must be an integer between 1 and 1000", "validation_error", { field: "limit" });
  }
  return value;
}

function parseOffset(raw: unknown): number {
  if (raw === undefined) {
    return 0;
  }
  const value = Number(raw);
  if (!Number.isInteger(value) || value < 0) {
    throw new HttpError(422, "offset must be a non-negative integer", "validation_error", { field: "offset" });
  }
  return value;
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
  const message: ProtocolMessage = {
    ...value,
    role: value.role,
    content: value.content,
  };
  if (value.id !== undefined && typeof value.id !== "string") {
    throw new HttpError(422, `${field}.id must be a string`, "validation_error", { field: `${field}.id` });
  }
  if (value.metadata !== undefined && !isObject(value.metadata)) {
    throw new HttpError(422, `${field}.metadata must be an object`, "validation_error", { field: `${field}.metadata` });
  }
  return message;
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

async function parseJsonObject(request: Request): Promise<Record<string, unknown>> {
  const body = await request.json().catch(() => {
    throw new HttpError(422, "Request body must be valid JSON", "validation_error");
  });
  if (!isObject(body)) {
    throw new HttpError(422, "Request body must be a JSON object", "validation_error");
  }
  return body;
}

async function readJson(path: string): Promise<unknown | null> {
  try {
    const data = await zenReadFile(path);
    return JSON.parse(decoder.decode(data));
  } catch {
    return null;
  }
}

async function writeJson(path: string, value: unknown): Promise<void> {
  const slash = path.lastIndexOf("/");
  if (slash > 0) {
    await zenMkdir(path.slice(0, slash), { recursive: true });
  }
  await zenWriteFile(path, encoder.encode(JSON.stringify(value)));
}

async function listJsonRows(dir: string): Promise<Array<Record<string, unknown>>> {
  await ensureZenFs();
  if (!(await zenExists(dir).catch(() => false))) {
    return [];
  }

  const rows: Array<Record<string, unknown>> = [];
  for (const entry of await zenReadDir(dir)) {
    const name = String((entry as { name?: string }).name ?? "");
    if (!name.endsWith(".json")) {
      continue;
    }
    const value = await readJson(`${dir}/${name}`);
    if (isObject(value)) {
      rows.push(value);
    }
  }
  return rows;
}

function sanitizeThreadValues(values: unknown): unknown {
  if (!isObject(values)) {
    return values;
  }
  if (!Array.isArray(values.messages)) {
    return values;
  }
  const next = { ...values };
  delete next.messages;
  return next;
}

function extractMessagesFromValues(values: unknown): ProtocolMessage[] | undefined {
  if (!isObject(values) || !Array.isArray(values.messages)) {
    return undefined;
  }
  return values.messages.map((entry) => normalizeProtocolMessage(entry));
}

async function readThreadMessages(threadId: string): Promise<ProtocolMessage[] | undefined> {
  const rows = await listRustMessages(threadId);
  const messages = rows
    .map((row) => ({
      created_at: String(row.created_at ?? ""),
      message: normalizeProtocolMessage({
        ...row,
        role: String(row.role ?? "assistant").toLowerCase(),
        content: row.content,
        ...(typeof row.id === "string" ? { id: row.id } : {}),
        ...(isObject(row.metadata) ? { metadata: row.metadata } : {}),
      }),
    }))
    .sort((left, right) => left.created_at.localeCompare(right.created_at))
    .map((entry) => entry.message);
  return messages.length ? messages : undefined;
}

async function getThreadRecord(threadId: string): Promise<Record<string, unknown> | null> {
  const value = await getRustThread(threadId);
  return isObject(value) ? value : null;
}

async function toProtocolThread(record: Record<string, unknown>): Promise<Record<string, unknown>> {
  const threadId = String(record.thread_id ?? record.id ?? "");
  const storedMessages = Array.isArray(record.messages) ? normalizeMessages(record.messages) : undefined;
  const messages = storedMessages ?? extractMessagesFromValues(record.values) ?? (await readThreadMessages(threadId));
  return {
    thread_id: threadId,
    created_at: String(record.created_at ?? new Date().toISOString()),
    updated_at: String(record.updated_at ?? new Date().toISOString()),
    metadata: isObject(record.metadata) ? record.metadata : {},
    status: normalizeStatus(record.status),
    ...(record.values === undefined || record.values === null ? {} : { values: sanitizeThreadValues(record.values) }),
    ...(messages?.length ? { messages } : {}),
  };
}

async function listThreads(): Promise<Record<string, unknown>[]> {
  const rows = await listRustThreads();
  const threads = await Promise.all(rows.map((row) => toProtocolThread(row)));
  return threads.sort((left, right) => String(right.updated_at).localeCompare(String(left.updated_at)));
}

function matchesRecord(record: Record<string, unknown>, filters?: Record<string, unknown>): boolean {
  if (!filters) {
    return true;
  }
  return Object.entries(filters).every(([key, value]) => jsonEquals(record[key], value));
}

function itemPath(namespace: string[], key: string): string {
  const cleanNamespace = namespace.map((part) => part.trim()).filter(Boolean);
  const namespacePath = cleanNamespace.length ? `/${cleanNamespace.join("/")}` : "";
  return `${STORE_DIR}${namespacePath}/${key}.json`;
}

function buildItem(
  namespace: string[],
  key: string,
  value: Record<string, unknown>,
  timestamps?: { created_at?: string; updated_at?: string },
): Record<string, unknown> {
  const now = new Date().toISOString();
  return {
    namespace,
    key,
    value,
    created_at: timestamps?.created_at ?? now,
    updated_at: timestamps?.updated_at ?? now,
  };
}

function isApiKeyNamespace(namespace: string[]): boolean {
  return namespace.length === 2 && namespace[0] === "config" && namespace[1] === "api-keys";
}

function isDefaultModelItem(namespace: string[], key: string): boolean {
  return namespace.length === 1 && namespace[0] === "config" && key === "default_model";
}

function parseApiKeyValue(value: Record<string, unknown>): string {
  if (typeof value.api_key === "string") {
    return value.api_key;
  }
  if (typeof value.value === "string") {
    return value.value;
  }
  throw new HttpError(422, "config api key items must include api_key or value", "validation_error", {
    field: "value",
  });
}

function parseDefaultModelValue(value: Record<string, unknown>): string {
  if (typeof value.model_id === "string") {
    return value.model_id;
  }
  if (typeof value.value === "string") {
    return value.value;
  }
  throw new HttpError(422, "config default model items must include model_id or value", "validation_error", {
    field: "value",
  });
}

async function ensureConfigItemMirror(namespace: string[], key: string): Promise<void> {
  const path = itemPath(namespace, key);
  if (await zenExists(path).catch(() => false)) {
    return;
  }

  if (isApiKeyNamespace(namespace)) {
    const apiKey = await getApiKey(key);
    if (!apiKey) {
      return;
    }
    await writeJson(path, buildItem(namespace, key, { api_key: apiKey }));
    return;
  }

  if (isDefaultModelItem(namespace, key)) {
    const modelId = (await getStoredDefaultModel())?.trim() ?? "";
    if (!modelId) {
      return;
    }
    await writeJson(path, buildItem(namespace, key, { model_id: modelId }));
  }
}

async function getStoreItem(namespace: string[], key: string): Promise<Record<string, unknown> | null> {
  if (!key) {
    return null;
  }

  await ensureConfigItemMirror(namespace, key);
  const value = await readJson(itemPath(namespace, key));
  return isObject(value) ? (value as Record<string, unknown>) : null;
}

async function walkStore(dir: string, rows: Array<Record<string, unknown>>): Promise<void> {
  if (!(await zenExists(dir).catch(() => false))) {
    return;
  }

  for (const entry of await zenReadDir(dir)) {
    const name = String((entry as { name?: string; is_dir?: boolean }).name ?? "");
    const isDir = Boolean((entry as { is_dir?: boolean }).is_dir);
    const next = `${dir}/${name}`;
    if (isDir) {
      await walkStore(next, rows);
      continue;
    }
    if (!name.endsWith(".json")) {
      continue;
    }
    const value = await readJson(next);
    if (isObject(value)) {
      rows.push(value as Record<string, unknown>);
    }
  }
}

async function listAllItems(): Promise<Array<Record<string, unknown>>> {
  await ensureConfigItemMirror(["config"], "default_model");
  for (const provider of ["anthropic", "openai", "google", "ollama"]) {
    await ensureConfigItemMirror(["config", "api-keys"], provider);
  }

  const items: Array<Record<string, unknown>> = [];
  await walkStore(STORE_DIR, items);
  return items;
}

function matchesValueFilter(item: Record<string, unknown>, filter: unknown): boolean {
  if (filter === undefined || filter === null) {
    return true;
  }
  if (!isObject(filter)) {
    return jsonEquals(item.value, filter);
  }
  return isObject(item.value) && matchesRecord(item.value, filter);
}

function dedupeNamespaces(items: Array<Record<string, unknown>>): string[][] {
  const seen = new Set<string>();
  const namespaces: string[][] = [];
  for (const item of items) {
    const namespace = Array.isArray(item.namespace) ? item.namespace.map(String) : [];
    const hash = namespace.join("/");
    if (!seen.has(hash)) {
      seen.add(hash);
      namespaces.push(namespace);
    }
  }
  return namespaces;
}

function agentMatches(body: Record<string, unknown>): boolean {
  const metadata = validatePlainObject(body.metadata, "metadata");
  const name = body.name;
  if (name !== undefined && typeof name !== "string") {
    throw new HttpError(422, "name must be a string", "validation_error", { field: "name" });
  }
  if (typeof name === "string" && !DEEP_AGENT.name.toLowerCase().includes(name.trim().toLowerCase())) {
    return false;
  }
  return matchesRecord(DEEP_AGENT.metadata, metadata);
}

async function createThreadFromRequest(request: Request): Promise<Response> {
  const body = await parseJsonObject(request);
  const requestedId =
    typeof body.thread_id === "string" && body.thread_id.trim() ? validateUuid(body.thread_id, "thread_id") : undefined;
  const existing = requestedId ? await getThreadRecord(requestedId) : null;
  const ifExists = body.if_exists === undefined ? "raise" : String(body.if_exists);
  if (!["raise", "do_nothing"].includes(ifExists)) {
    throw new HttpError(422, "if_exists must be either raise or do_nothing", "validation_error", {
      field: "if_exists",
    });
  }

  if (existing) {
    if (ifExists === "do_nothing") {
      return Response.json(await toProtocolThread(existing));
    }
    return errorResponse(409, `Thread ${requestedId} already exists`, "thread_conflict", { thread_id: requestedId });
  }

  const metadata = validatePlainObject(body.metadata, "metadata") ?? {};
  const thread = await createRustThread({
    ...(requestedId ? { thread_id: requestedId } : {}),
    metadata,
  });
  return Response.json(await toProtocolThread(thread));
}

async function searchThreads(request: Request): Promise<Response> {
  const body = await parseJsonObject(request);
  const metadata = validatePlainObject(body.metadata, "metadata");
  const values = validatePlainObject(body.values, "values");
  const status = validateStatus(body.status);
  const offset = parseOffset(body.offset);
  const limit = parseLimit(body.limit, DEFAULT_LIMIT);

  const filtered = (await listThreads()).filter((thread) => {
    if (status && thread.status !== status) {
      return false;
    }
    if (!matchesRecord(isObject(thread.metadata) ? thread.metadata : {}, metadata)) {
      return false;
    }
    return matchesRecord(isObject(thread.values) ? thread.values : {}, values);
  });

  return Response.json(filtered.slice(offset, offset + limit));
}

function mergeValues(current: unknown, patch: Record<string, unknown> | undefined): unknown {
  if (!patch) {
    return current;
  }
  if (!isObject(current)) {
    return { ...patch };
  }
  return {
    ...current,
    ...patch,
  };
}

async function patchThread(threadId: string, request: Request): Promise<Response> {
  const current = await getThreadRecord(threadId);
  if (!current) {
    return errorResponse(404, "Thread not found", "thread_not_found", { thread_id: threadId });
  }

  const patch = await parseJsonObject(request);
  const metadataPatch = validatePlainObject(patch.metadata, "metadata");
  const valuesPatch = validatePlainObject(patch.values, "values");
  const messagesPatch = normalizeMessages(patch.messages);

  const baseValues = current.values;
  const baseMessages = Array.isArray(current.messages)
    ? normalizeMessages(current.messages)
    : (extractMessagesFromValues(current.values) ?? (await readThreadMessages(threadId)));

  if (patch.checkpoint !== undefined) {
    throw new HttpError(422, "checkpoint-based thread patching is not supported", "unsupported_checkpoint_patch", {
      field: "checkpoint",
    });
  }

  const next: Record<string, unknown> = {
    metadata: {
      ...(isObject(current.metadata) ? current.metadata : {}),
      ...(metadataPatch ?? {}),
    },
    values: mergeValues(baseValues, valuesPatch),
    ...(baseMessages?.length || messagesPatch?.length
      ? {
          messages: [...(baseMessages ?? []), ...(messagesPatch ?? [])],
        }
      : {}),
  };

  const updated = await patchRustThread(threadId, next);
  return Response.json(await toProtocolThread(updated ?? current));
}

async function copyThreadArtifacts(sourceThreadId: string, nextThreadId: string): Promise<void> {
  for (const row of await listRustMessages(sourceThreadId)) {
    const createdAt = typeof row.created_at === "string" ? row.created_at : new Date().toISOString();
    await saveRustMessage(nextThreadId, createdAt, {
      ...row,
      thread_id: nextThreadId,
    });
  }

  for (const root of [TODOS_DIR, SUBAGENTS_DIR]) {
    const sourceDir = `${root}/${sourceThreadId}`;
    if (!(await zenExists(sourceDir).catch(() => false))) {
      continue;
    }

    for (const row of await listJsonRows(sourceDir)) {
      const cloned = { ...row, thread_id: nextThreadId };
      const id = String(cloned.id ?? crypto.randomUUID());
      await writeJson(`${root}/${nextThreadId}/${id}.json`, cloned);
    }
  }
}

async function copyThread(threadId: string): Promise<Response> {
  const current = await getThreadRecord(threadId);
  if (!current) {
    return errorResponse(404, "Thread not found", "thread_not_found", { thread_id: threadId });
  }

  const nextThreadId = crypto.randomUUID();
  const metadata = isObject(current.metadata) ? { ...current.metadata } : {};
  if (typeof metadata.title === "string") {
    metadata.title = `${metadata.title} Copy`;
  }

  const clone = await createRustThread({
    thread_id: nextThreadId,
    metadata,
  });
  await patchRustThread(nextThreadId, {
    ...(current.values === undefined ? {} : { values: current.values }),
    ...(Array.isArray(current.messages) ? { messages: current.messages } : {}),
  });
  if (typeof current.status === "string" && current.status !== "idle") {
    await setRustThreadStatus(nextThreadId, current.status);
  }
  await copyThreadArtifacts(threadId, nextThreadId);
  return Response.json(await toProtocolThread(clone));
}

async function listThreadHistory(request: Request, threadId: string): Promise<Response> {
  const current = await getThreadRecord(threadId);
  if (!current) {
    return errorResponse(404, "Thread not found", "thread_not_found", { thread_id: threadId });
  }

  const url = new URL(request.url);
  const limit = parseLimit(url.searchParams.get("limit") ?? undefined, DEFAULT_LIMIT);
  const before = url.searchParams.get("before") ?? undefined;
  const beforeId = before ? validateUuid(before, "before") : undefined;
  const saver = new SqlJsSaver();
  const tuples = [];
  for await (const tuple of saver.list({ configurable: { thread_id: threadId } })) {
    tuples.push(tuple);
  }

  const filtered = beforeId
    ? tuples.slice(Math.max(0, tuples.findIndex((tuple) => tuple.config?.configurable?.checkpoint_id === beforeId) + 1))
    : tuples;

  const states = filtered.slice(0, limit).map((tuple) => {
    const checkpoint = tuple.checkpoint as Record<string, unknown>;
    const values = isObject(checkpoint.channel_values) ? checkpoint.channel_values : checkpoint;
    const messages = extractMessagesFromValues(values);
    const checkpointFields = Object.fromEntries(
      Object.entries(checkpoint).filter(([key]) => key !== "id" && key !== "channel_values"),
    );
    return {
      checkpoint: {
        checkpoint_id: String(checkpoint.id ?? tuple.config?.configurable?.checkpoint_id ?? ""),
        ...checkpointFields,
      },
      values: sanitizeThreadValues(values) ?? {},
      ...(messages?.length ? { messages } : {}),
      ...(isObject(tuple.metadata) ? { metadata: tuple.metadata } : {}),
    };
  });

  return Response.json(states);
}

async function deleteThreadCascade(threadId: string): Promise<Response> {
  const current = await getThreadRecord(threadId);
  if (!current) {
    return errorResponse(404, "Thread not found", "thread_not_found", { thread_id: threadId });
  }
  await deleteThread(threadId);
  const checkpointPath = `${CHECKPOINTS_DIR}/${threadId}.sqlite`;
  if (await zenExists(checkpointPath).catch(() => false)) {
    await zenRm(checkpointPath, { recursive: false });
  }
  return new Response(null, { status: 204 });
}

async function putStoreItem(body: Record<string, unknown>): Promise<Response> {
  const namespace = parseOptionalArrayOfStrings(body.namespace, "namespace") ?? [];
  const key = typeof body.key === "string" ? body.key.trim() : "";
  const value = validatePlainObject(body.value, "value");
  if (!key) {
    return errorResponse(422, "key is required", "validation_error", { field: "key" });
  }
  if (!value) {
    return errorResponse(422, "value is required", "validation_error", { field: "value" });
  }

  const existing = await getStoreItem(namespace, key);
  const item = buildItem(namespace, key, value, {
    created_at: existing?.created_at as string | undefined,
    updated_at: new Date().toISOString(),
  });
  await writeJson(itemPath(namespace, key), item);

  if (isApiKeyNamespace(namespace)) {
    await setApiKey(key, parseApiKeyValue(value));
  } else if (isDefaultModelItem(namespace, key)) {
    await setDefaultModel(parseDefaultModelValue(value));
  }

  return new Response(null, { status: 204 });
}

async function deleteStoreItem(body: Record<string, unknown>): Promise<Response> {
  const namespace = parseOptionalArrayOfStrings(body.namespace, "namespace") ?? [];
  const key = typeof body.key === "string" ? body.key.trim() : "";
  if (!key) {
    return errorResponse(422, "key is required", "validation_error", { field: "key" });
  }

  const existing = await getStoreItem(namespace, key);
  if (!existing) {
    return errorResponse(404, "Item not found", "item_not_found", { key, namespace });
  }

  const path = itemPath(namespace, key);
  if (await zenExists(path).catch(() => false)) {
    await zenRm(path, { recursive: false });
  }

  if (isApiKeyNamespace(namespace)) {
    await deleteApiKey(key);
  } else if (isDefaultModelItem(namespace, key)) {
    await deleteDefaultModel();
  }

  return new Response(null, { status: 204 });
}

async function searchItems(request: Request): Promise<Response> {
  const body = await parseJsonObject(request);
  const namespacePrefix = parseOptionalArrayOfStrings(body.namespace_prefix, "namespace_prefix");
  const filter = body.filter;
  if (filter !== undefined && filter !== null && !isObject(filter)) {
    throw new HttpError(422, "filter must be an object", "validation_error", { field: "filter" });
  }
  const offset = parseOffset(body.offset);
  const limit = parseLimit(body.limit, DEFAULT_LIMIT);

  const filtered = (await listAllItems()).filter((item) => {
    const namespace = Array.isArray(item.namespace) ? item.namespace.map(String) : [];
    if (namespacePrefix?.some((part, index) => namespace[index] !== part)) {
      return false;
    }
    return matchesValueFilter(item, filter);
  });

  return Response.json({ items: filtered.slice(offset, offset + limit) });
}

async function listNamespaces(request: Request): Promise<Response> {
  const body = await parseJsonObject(request);
  const prefix = parseOptionalArrayOfStrings(body.prefix, "prefix");
  const suffix = parseOptionalArrayOfStrings(body.suffix, "suffix");
  const maxDepth = body.max_depth === undefined ? undefined : Number(body.max_depth);
  if (maxDepth !== undefined && (!Number.isInteger(maxDepth) || maxDepth < 0)) {
    throw new HttpError(422, "max_depth must be a non-negative integer", "validation_error", { field: "max_depth" });
  }
  const offset = parseOffset(body.offset);
  const limit = parseLimit(body.limit, DEFAULT_NAMESPACE_LIMIT);

  const namespaces = dedupeNamespaces(await listAllItems()).filter((namespace) => {
    if (prefix?.some((part, index) => namespace[index] !== part)) {
      return false;
    }
    if (suffix && suffix.length > namespace.length) {
      return false;
    }
    if (suffix?.some((part, index) => namespace[namespace.length - suffix.length + index] !== part)) {
      return false;
    }
    if (maxDepth !== undefined && namespace.length > maxDepth) {
      return false;
    }
    return true;
  });

  return Response.json(namespaces.slice(offset, offset + limit));
}

export function matchStoreRoute(request: Request): StoreRoute {
  const parts = parsePath(request);
  const method = request.method;

  if (parts[0] === "x" && parts[1] === "bootstrap" && method === "GET") return "x-bootstrap";
  if (parts[0] === "x" && parts[1] === "providers" && method === "GET") return "x-providers";
  if (parts[0] === "x" && parts[1] === "files" && method === "GET") return "x-files";

  if (parts[0] === "agents" && parts[1] === "search" && method === "POST") return "agents-search";
  if (parts[0] === "agents" && parts.length === 2 && method === "GET") return "agents-get";
  if (parts[0] === "agents" && parts[2] === "schemas" && method === "GET") return "agents-schemas";

  if (parts[0] === "threads" && parts.length === 1 && method === "POST") return "threads-create";
  if (parts[0] === "threads" && parts[1] === "search" && method === "POST") return "threads-search";
  if (parts[0] === "threads" && parts.length === 2 && method === "GET") return "threads-get";
  if (parts[0] === "threads" && parts.length === 2 && method === "PATCH") return "threads-patch";
  if (parts[0] === "threads" && parts.length === 2 && method === "DELETE") return "threads-delete";
  if (parts[0] === "threads" && parts[2] === "history" && method === "GET") return "threads-history";
  if (parts[0] === "threads" && parts[2] === "copy" && method === "POST") return "threads-copy";

  if (parts[0] === "store" && parts[1] === "items" && method === "GET") return "store-items-get";
  if (parts[0] === "store" && parts[1] === "items" && method === "PUT") return "store-items-put";
  if (parts[0] === "store" && parts[1] === "items" && method === "DELETE") return "store-items-delete";
  if (parts[0] === "store" && parts[1] === "items" && parts[2] === "search" && method === "POST")
    return "store-items-search";
  if (parts[0] === "store" && parts[1] === "namespaces" && method === "POST") return "store-namespaces";

  return null;
}

export async function handleStoreRoute(request: Request, route: Exclude<StoreRoute, null>): Promise<Response> {
  try {
    const url = new URL(request.url);
    const parts = parsePath(request);

    if (route === "x-bootstrap") {
      return Response.json(await buildBootstrap());
    }

    if (route === "x-providers") {
      return Response.json(await readProvidersWithKeys());
    }

    if (route === "x-files") {
      const workspace = url.searchParams.get("workspace") ?? "/home/workspace";
      return Response.json(await listWorkspaceFiles(workspace));
    }

    if (route === "agents-search") {
      const body = await parseJsonObject(request);
      const offset = parseOffset(body.offset);
      const limit = parseLimit(body.limit, DEFAULT_LIMIT);
      const items = agentMatches(body) ? [DEEP_AGENT] : [];
      return Response.json(items.slice(offset, offset + limit));
    }

    if (route === "agents-get") {
      return parts[1] === DEEP_AGENT.agent_id
        ? Response.json(DEEP_AGENT)
        : errorResponse(404, "Agent not found", "agent_not_found", { agent_id: parts[1] });
    }

    if (route === "agents-schemas") {
      return parts[1] === DEEP_AGENT.agent_id
        ? Response.json(DEEP_AGENT_SCHEMA)
        : errorResponse(404, "Agent not found", "agent_not_found", { agent_id: parts[1] });
    }

    if (route === "threads-create") {
      return await createThreadFromRequest(request);
    }

    if (route === "threads-search") {
      return await searchThreads(request);
    }

    if (route === "threads-get") {
      const threadId = validateUuid(parts[1], "thread_id");
      const thread = await getThreadRecord(threadId);
      return thread
        ? Response.json(await toProtocolThread(thread))
        : errorResponse(404, "Thread not found", "thread_not_found", { thread_id: threadId });
    }

    if (route === "threads-patch") {
      return await patchThread(validateUuid(parts[1], "thread_id"), request);
    }

    if (route === "threads-delete") {
      return await deleteThreadCascade(validateUuid(parts[1], "thread_id"));
    }

    if (route === "threads-history") {
      return await listThreadHistory(request, validateUuid(parts[1], "thread_id"));
    }

    if (route === "threads-copy") {
      return await copyThread(validateUuid(parts[1], "thread_id"));
    }

    if (route === "store-items-get") {
      const namespace = url.searchParams.getAll("namespace");
      const key = url.searchParams.get("key") ?? "";
      if (!key.trim()) {
        return errorResponse(400, "key query parameter is required", "bad_request", { field: "key" });
      }
      const item = await getStoreItem(namespace, key);
      return item ? Response.json(item) : errorResponse(404, "Item not found", "item_not_found", { key, namespace });
    }

    if (route === "store-items-put") {
      return await putStoreItem(await parseJsonObject(request));
    }

    if (route === "store-items-delete") {
      return await deleteStoreItem(await parseJsonObject(request));
    }

    if (route === "store-items-search") {
      return await searchItems(request);
    }

    if (route === "store-namespaces") {
      return await listNamespaces(request);
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
