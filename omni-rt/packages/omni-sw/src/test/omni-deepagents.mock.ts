import { exists, mkdir, readFile, readdir, rm, writeFile } from "../zenfs.js";

const encoder = new TextEncoder();
const decoder = new TextDecoder();

const THREADS_DIR = "/home/db/threads";
const MESSAGES_DIR = "/home/db/messages";
const RUNS_DIR = "/home/db/runs";
const CONFIG_DIR = "/home/config";
const ENV_FILE = `${CONFIG_DIR}/.env`;
const DEFAULT_MODEL_FILE = `${CONFIG_DIR}/default_model`;

type JsonObject = Record<string, unknown>;

export default async function initDeepagents(): Promise<void> {}

function isObject(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

async function readJson(path: string): Promise<unknown | null> {
  try {
    return JSON.parse(decoder.decode(await readFile(path)));
  } catch {
    return null;
  }
}

async function writeJson(path: string, value: unknown): Promise<void> {
  const slash = path.lastIndexOf("/");
  if (slash > 0) {
    await mkdir(path.slice(0, slash), { recursive: true });
  }
  await writeFile(path, encoder.encode(JSON.stringify(value)));
}

async function listJson(dir: string): Promise<JsonObject[]> {
  if (!(await exists(dir).catch(() => false))) {
    return [];
  }
  const rows: JsonObject[] = [];
  for (const entry of await readdir(dir)) {
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

function threadPath(threadId: string): string {
  return `${THREADS_DIR}/${threadId}.json`;
}

function runPath(runId: string): string {
  return `${RUNS_DIR}/${runId}.json`;
}

async function readEnvMap(): Promise<Record<string, string>> {
  const raw = await readJson(ENV_FILE);
  if (isObject(raw)) {
    return Object.fromEntries(Object.entries(raw).map(([key, value]) => [key, String(value)]));
  }
  try {
    const text = decoder.decode(await readFile(ENV_FILE));
    return Object.fromEntries(
      text
        .split("\n")
        .map((line) => line.split("="))
        .filter((entry) => entry.length === 2)
        .map(([key, value]) => [key.trim(), value.trim()]),
    );
  } catch {
    return {};
  }
}

async function writeEnvMap(value: Record<string, string>): Promise<void> {
  await mkdir(CONFIG_DIR, { recursive: true });
  const body = Object.entries(value)
    .map(([key, item]) => `${key}=${item}`)
    .join("\n");
  await writeFile(ENV_FILE, encoder.encode(body));
}

export async function deepagents_list_threads(): Promise<JsonObject[]> {
  const rows = await listJson(THREADS_DIR);
  rows.sort((left, right) => String(right.updated_at ?? "").localeCompare(String(left.updated_at ?? "")));
  return rows;
}

export async function deepagents_get_thread(threadId: string): Promise<JsonObject | null> {
  const value = await readJson(threadPath(threadId));
  return isObject(value) ? value : null;
}

export async function deepagents_create_thread(payload: JsonObject): Promise<JsonObject> {
  const now = new Date().toISOString();
  const thread = {
    thread_id: typeof payload.thread_id === "string" ? payload.thread_id : crypto.randomUUID(),
    created_at: now,
    updated_at: now,
    metadata: isObject(payload.metadata) ? payload.metadata : {},
    status: "idle",
    values: null,
    messages: null,
  };
  await writeJson(threadPath(String(thread.thread_id)), thread);
  return thread;
}

export async function deepagents_patch_thread(threadId: string, payload: JsonObject): Promise<JsonObject | null> {
  const current = await deepagents_get_thread(threadId);
  if (!current) {
    return null;
  }
  const next = {
    ...current,
    updated_at: new Date().toISOString(),
    ...(isObject(payload.metadata)
      ? { metadata: { ...(isObject(current.metadata) ? current.metadata : {}), ...payload.metadata } }
      : {}),
    ...(payload.values !== undefined ? { values: payload.values } : {}),
    ...(Array.isArray(payload.messages) ? { messages: payload.messages } : {}),
  };
  await writeJson(threadPath(threadId), next);
  return next;
}

export async function deepagents_save_thread(payload: JsonObject): Promise<void> {
  const threadId = String(payload.thread_id ?? "");
  await writeJson(threadPath(threadId), payload);
}

export async function deepagents_set_thread_status(threadId: string, status: string): Promise<JsonObject | null> {
  const current = await deepagents_get_thread(threadId);
  if (!current) {
    return null;
  }
  const next = {
    ...current,
    status,
    updated_at: new Date().toISOString(),
  };
  await writeJson(threadPath(threadId), next);
  return next;
}

export async function deepagents_delete_thread(threadId: string): Promise<void> {
  if (await exists(threadPath(threadId)).catch(() => false)) {
    await rm(threadPath(threadId), { recursive: false });
  }
}

export async function deepagents_list_messages(threadId: string): Promise<JsonObject[]> {
  const rows = await listJson(`${MESSAGES_DIR}/${threadId}`);
  rows.sort((left, right) => String(left.created_at ?? "").localeCompare(String(right.created_at ?? "")));
  return rows;
}

export async function deepagents_save_message(threadId: string, createdAt: string, payload: JsonObject): Promise<void> {
  const id = typeof payload.id === "string" && payload.id ? payload.id : crypto.randomUUID();
  await writeJson(`${MESSAGES_DIR}/${threadId}/${id}.json`, {
    ...payload,
    id,
    thread_id: threadId,
    created_at: createdAt,
  });
}

export async function deepagents_delete_thread_messages(threadId: string): Promise<void> {
  const dir = `${MESSAGES_DIR}/${threadId}`;
  if (await exists(dir).catch(() => false)) {
    await rm(dir, { recursive: true });
  }
}

export async function deepagents_get_default_model(): Promise<string> {
  try {
    return decoder.decode(await readFile(DEFAULT_MODEL_FILE)).trim() || "claude-3-7-sonnet";
  } catch {
    return "claude-3-7-sonnet";
  }
}

export async function deepagents_get_stored_default_model(): Promise<string | null> {
  try {
    const modelId = decoder.decode(await readFile(DEFAULT_MODEL_FILE)).trim();
    return modelId || null;
  } catch {
    return null;
  }
}

export async function deepagents_set_default_model(modelId: string): Promise<void> {
  await mkdir(CONFIG_DIR, { recursive: true });
  await writeFile(DEFAULT_MODEL_FILE, encoder.encode(modelId));
}

export async function deepagents_delete_default_model(): Promise<void> {
  if (await exists(DEFAULT_MODEL_FILE).catch(() => false)) {
    await rm(DEFAULT_MODEL_FILE, { recursive: false });
  }
}

export async function deepagents_get_api_key(provider: string): Promise<string | null> {
  const env = await readEnvMap();
  return env[`${provider.toUpperCase()}_API_KEY`] ?? null;
}

export async function deepagents_set_api_key(provider: string, value: string): Promise<void> {
  const env = await readEnvMap();
  env[`${provider.toUpperCase()}_API_KEY`] = value;
  await writeEnvMap(env);
}

export async function deepagents_delete_api_key(provider: string): Promise<void> {
  const env = await readEnvMap();
  delete env[`${provider.toUpperCase()}_API_KEY`];
  await writeEnvMap(env);
}

export async function deepagents_list_runs(): Promise<JsonObject[]> {
  const rows = await listJson(RUNS_DIR);
  rows.sort((left, right) => {
    const leftRun = isObject(left.run) ? left.run : {};
    const rightRun = isObject(right.run) ? right.run : {};
    return String(rightRun.updated_at ?? "").localeCompare(String(leftRun.updated_at ?? ""));
  });
  return rows;
}

export async function deepagents_save_run(payload: JsonObject): Promise<void> {
  const run = isObject(payload.run) ? payload.run : {};
  await writeJson(runPath(String(run.run_id ?? "")), payload);
}

export async function deepagents_get_run(runId: string): Promise<JsonObject | null> {
  const value = await readJson(runPath(runId));
  return isObject(value) ? value : null;
}

export async function deepagents_search_runs(payload: JsonObject): Promise<JsonObject[]> {
  const rows = await deepagents_list_runs();
  const metadata = isObject(payload.metadata) ? payload.metadata : undefined;
  const status = typeof payload.status === "string" ? payload.status : undefined;
  const threadId = typeof payload.thread_id === "string" ? payload.thread_id : undefined;
  const agentId = typeof payload.agent_id === "string" ? payload.agent_id : undefined;
  const offset = Math.max(0, Number(payload.offset ?? 0));
  const limit = Math.max(1, Number(payload.limit ?? 10));
  return rows
    .filter((row) => {
      const run = isObject(row.run) ? row.run : {};
      if (status && run.status !== status) {
        return false;
      }
      if (threadId && run.thread_id !== threadId) {
        return false;
      }
      if (agentId && run.agent_id !== agentId) {
        return false;
      }
      if (!metadata) {
        return true;
      }
      const actual = isObject(run.metadata) ? run.metadata : {};
      return Object.entries(metadata).every(([key, value]) => JSON.stringify(actual[key]) === JSON.stringify(value));
    })
    .slice(offset, offset + limit);
}

export async function deepagents_delete_run(runId: string): Promise<void> {
  if (await exists(runPath(runId)).catch(() => false)) {
    await rm(runPath(runId), { recursive: false });
  }
}
