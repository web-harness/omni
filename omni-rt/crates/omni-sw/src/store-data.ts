import {
  exists as zenExists,
  init as zenInit,
  mkdir as zenMkdir,
  readFile as zenReadFile,
  readdir as zenReadDir,
  rm as zenRm,
  writeFile as zenWriteFile,
} from "./omni-zenfs.js";
import {
  DEFAULT_WORKSPACE_ORDER,
  SCAFFOLD_FILES,
  getMockThreadFiles,
  getMockToolCalls,
  getMockToolResults,
  getMockWorkspaceFiles,
  seedThreads,
} from "./store-mocks.js";

const THREADS_DIR = "/home/db/threads";
const MESSAGES_DIR = "/home/db/messages";
const TODOS_DIR = "/home/db/todos";
const SUBAGENTS_DIR = "/home/db/subagents";
const CONFIG_DIR = "/home/config";
const ENV_FILE = "/home/config/.env";
const DEFAULT_MODEL_FILE = "/home/config/default_model";

type ProviderId = "Anthropic" | "OpenAI" | "Google" | "Ollama";

export type BootstrapPayload = {
  threads: Array<{ id: string; title: string; status: string; updated_at: string }>;
  messages: Record<string, Array<{ id: string; role: string; content: string }>>;
  todos: Record<string, Array<{ id: string; content: string; status: string }>>;
  files: Record<string, Array<{ path: string; is_dir: boolean; size: number | null }>>;
  tool_calls: Record<string, Array<{ id: string; name: string; args: unknown }>>;
  tool_results: Record<string, Array<{ tool_call_id: string; content: string; is_error: boolean }>>;
  subagents: Record<string, Array<{ id: string; name: string; description: string; status: string }>>;
  workspace_path: Record<string, string>;
  workspace_files: Record<string, Array<{ path: string; is_dir: boolean; size: number | null }>>;
  providers: Array<{ id: ProviderId; name: string; has_api_key: boolean }>;
  models: Array<{ id: string; name: string; provider: ProviderId }>;
  default_model: string;
};

type DirEntry = { name: string; is_dir: boolean; is_file: boolean };

let zenFsReady: Promise<void> | null = null;

const encoder = new TextEncoder();
const decoder = new TextDecoder();

async function ensureZenFs(): Promise<void> {
  if (!zenFsReady) {
    zenFsReady = zenInit().catch(() => {});
  }
  await zenFsReady;
}

function normalizeThreadId(id: unknown): string {
  return String(id ?? "").trim();
}

function toThreadStatus(raw: unknown): string {
  const value = String(raw ?? "Idle").toLowerCase();
  if (value === "busy") return "Busy";
  if (value === "interrupted") return "Interrupted";
  if (value === "error") return "Error";
  return "Idle";
}

function toRole(raw: unknown): string {
  const value = String(raw ?? "assistant").toLowerCase();
  if (value === "user") return "User";
  if (value === "tool") return "Tool";
  return "Assistant";
}

function toTodoStatus(raw: unknown): string {
  const value = String(raw ?? "pending").toLowerCase();
  if (value === "in_progress") return "InProgress";
  if (value === "completed") return "Completed";
  if (value === "cancelled") return "Cancelled";
  return "Pending";
}

function toSubagentStatus(raw: unknown): string {
  const value = String(raw ?? "pending").toLowerCase();
  if (value === "running") return "Running";
  if (value === "completed") return "Completed";
  if (value === "failed") return "Failed";
  return "Pending";
}

async function exists(path: string): Promise<boolean> {
  return zenExists(path).catch(() => false);
}

async function readText(path: string): Promise<string> {
  const data = await zenReadFile(path);
  return decoder.decode(data);
}

async function readJson(path: string): Promise<unknown | null> {
  try {
    const data = await readText(path);
    return JSON.parse(data);
  } catch {
    return null;
  }
}

async function readJsonFiles(dir: string): Promise<unknown[]> {
  if (!(await exists(dir))) return [];
  const entries = await zenReadDir(dir);
  const rows: unknown[] = [];
  for (const entry of entries) {
    const name = String((entry as { name?: string }).name ?? "");
    if (!name.endsWith(".json")) continue;
    const item = await readJson(`${dir}/${name}`);
    if (item) rows.push(item);
  }
  return rows;
}

async function removePath(path: string, recursive: boolean): Promise<void> {
  try {
    await zenRm(path, { recursive });
  } catch {
    // ignore
  }
}

function providerDefs(): Array<{ id: ProviderId; name: string; prefix: string }> {
  return [
    { id: "Anthropic", name: "Anthropic", prefix: "anthropic" },
    { id: "OpenAI", name: "OpenAI", prefix: "openai" },
    { id: "Google", name: "Google", prefix: "google" },
    { id: "Ollama", name: "Ollama", prefix: "ollama" },
  ];
}

function modelDefs(): Array<{ id: string; name: string; provider: ProviderId }> {
  return [
    { id: "claude-3-7-sonnet", name: "Claude 3.7 Sonnet", provider: "Anthropic" },
    { id: "claude-3-5-haiku", name: "Claude 3.5 Haiku", provider: "Anthropic" },
    { id: "gpt-5", name: "GPT-5", provider: "OpenAI" },
    { id: "gpt-4o", name: "GPT-4o", provider: "OpenAI" },
    { id: "gemini-2.5-pro", name: "Gemini 2.5 Pro", provider: "Google" },
    { id: "gemini-2.0-flash", name: "Gemini 2.0 Flash", provider: "Google" },
    { id: "llama-3.3-70b", name: "Llama 3.3 70B", provider: "Ollama" },
    { id: "deepseek-r1", name: "DeepSeek R1", provider: "Ollama" },
  ];
}

async function seedIfEmpty(): Promise<void> {
  await ensureZenFs();
  await zenMkdir(THREADS_DIR, { recursive: true });
  const existing = await readJsonFiles(THREADS_DIR);
  if (existing.length === 0) {
    for (const thread of seedThreads()) {
      const threadDoc = {
        thread_id: thread.id,
        created_at: thread.updated_at,
        updated_at: thread.updated_at,
        metadata: { title: thread.title },
        status: thread.status,
        values: null,
        messages: null,
      };
      await zenWriteFile(`${THREADS_DIR}/${thread.id}.json`, encoder.encode(JSON.stringify(threadDoc)));

      await zenMkdir(`${MESSAGES_DIR}/${thread.id}`, { recursive: true });
      for (const msg of thread.messages) {
        await zenWriteFile(
          `${MESSAGES_DIR}/${thread.id}/${msg.id}.json`,
          encoder.encode(JSON.stringify({ ...msg, thread_id: thread.id })),
        );
      }

      await zenMkdir(`${TODOS_DIR}/${thread.id}`, { recursive: true });
      for (const todo of thread.todos) {
        await zenWriteFile(
          `${TODOS_DIR}/${thread.id}/${todo.id}.json`,
          encoder.encode(JSON.stringify({ ...todo, thread_id: thread.id })),
        );
      }

      await zenMkdir(`${SUBAGENTS_DIR}/${thread.id}`, { recursive: true });
      for (const sa of thread.subagents) {
        await zenWriteFile(
          `${SUBAGENTS_DIR}/${thread.id}/${sa.id}.json`,
          encoder.encode(JSON.stringify({ ...sa, thread_id: thread.id })),
        );
      }
    }
  }

  await ensureWorkspaceScaffold();
}

async function ensureWorkspaceScaffold(): Promise<void> {
  const writeIfMissing = async (path: string, content: string): Promise<void> => {
    if (await exists(path)) return;
    await zenMkdir(path.slice(0, path.lastIndexOf("/")), { recursive: true });
    await zenWriteFile(path, encoder.encode(content));
  };

  for (const file of SCAFFOLD_FILES) {
    await writeIfMissing(file.path, file.content);
  }
}

export async function listWorkspaceFiles(
  root: string,
): Promise<Array<{ path: string; is_dir: boolean; size: number | null }>> {
  const byWorkspace = getMockWorkspaceFiles();
  if (root in byWorkspace) {
    return byWorkspace[root] ?? [];
  }

  await ensureZenFs();
  await ensureWorkspaceScaffold();
  const cleanRoot = root?.startsWith("/") ? root : "/home/workspace";
  if (!(await exists(cleanRoot))) {
    return [];
  }

  const out: Array<{ path: string; is_dir: boolean; size: number | null }> = [];

  const walk = async (dir: string, depth: number): Promise<void> => {
    if (depth > 2) return;
    const entries = await zenReadDir(dir);
    for (const raw of entries) {
      const entry = raw as DirEntry;
      const full = `${dir}/${entry.name}`.replace(/\/+/g, "/");
      if (entry.is_dir) {
        out.push({ path: full, is_dir: true, size: null });
        await walk(full, depth + 1);
      } else if (entry.is_file) {
        const bytes = await zenReadFile(full);
        out.push({ path: full, is_dir: false, size: bytes.length });
      }
    }
  };

  await walk(cleanRoot, 0);
  return out;
}

function parseEnv(env: string): Record<string, string> {
  const out: Record<string, string> = {};
  for (const line of env.split("\n")) {
    const idx = line.indexOf("=");
    if (idx <= 0) continue;
    const key = line.slice(0, idx).trim();
    const value = line.slice(idx + 1).trim();
    if (key) out[key] = value;
  }
  return out;
}

async function readEnvMap(): Promise<Record<string, string>> {
  if (!(await exists(ENV_FILE))) return {};
  return parseEnv(await readText(ENV_FILE));
}

async function writeEnvMap(env: Record<string, string>): Promise<void> {
  const body = Object.entries(env)
    .map(([k, v]) => `${k}=${v}`)
    .join("\n");
  await zenMkdir(CONFIG_DIR, { recursive: true });
  await zenWriteFile(ENV_FILE, encoder.encode(body));
}

export async function getDefaultModel(): Promise<string> {
  if (!(await exists(DEFAULT_MODEL_FILE))) return "claude-3-7-sonnet";
  return (await readText(DEFAULT_MODEL_FILE)).trim() || "claude-3-7-sonnet";
}

export async function buildBootstrap(): Promise<BootstrapPayload> {
  await ensureZenFs();
  await seedIfEmpty();

  const defaultModel = await getDefaultModel();
  const providers = await readProvidersWithKeys();
  const models = modelDefs();

  const threadRows = await readJsonFiles(THREADS_DIR);
  const threads = threadRows
    .map((row) => {
      const rec = row as Record<string, unknown>;
      const id = normalizeThreadId(rec.thread_id ?? rec.id);
      const metadata = (rec.metadata as Record<string, unknown> | undefined) ?? {};
      return {
        id,
        title: String(metadata.title ?? "New Thread"),
        status: toThreadStatus(rec.status),
        updated_at: String(rec.updated_at ?? new Date().toISOString()),
      };
    })
    .filter((t) => t.id.length > 0)
    .sort((a, b) => {
      const order = ["thread-gtd", "thread-auth", "thread-db", "thread-ci", "thread-idea"];
      const ai = order.indexOf(a.id);
      const bi = order.indexOf(b.id);
      if (ai >= 0 && bi >= 0) return ai - bi;
      if (ai >= 0) return -1;
      if (bi >= 0) return 1;
      return b.updated_at.localeCompare(a.updated_at);
    });

  const messages: BootstrapPayload["messages"] = {};
  const todos: BootstrapPayload["todos"] = {};
  const files: BootstrapPayload["files"] = {};
  const tool_calls: BootstrapPayload["tool_calls"] = {};
  const tool_results: BootstrapPayload["tool_results"] = {};
  const subagents: BootstrapPayload["subagents"] = {};
  const workspace_path: BootstrapPayload["workspace_path"] = {};
  const workspace_files = getMockWorkspaceFiles();

  const seedById = new Map(seedThreads().map((t) => [t.id, t]));

  for (const [index, thread] of threads.entries()) {
    const seeded = seedById.get(thread.id);

    const msgRows = await readJsonFiles(`${MESSAGES_DIR}/${thread.id}`);
    let parsedMessages = msgRows
      .map((row) => {
        const rec = row as Record<string, unknown>;
        return {
          id: String(rec.id ?? ""),
          role: toRole(rec.role),
          content: String(rec.content ?? ""),
          created_at: String(rec.created_at ?? ""),
        };
      })
      .sort((a, b) => a.created_at.localeCompare(b.created_at))
      .map(({ id, role, content }) => ({ id, role, content }));
    if (parsedMessages.length === 0 && seeded) {
      parsedMessages = seeded.messages.map((m) => ({ id: m.id, role: toRole(m.role), content: m.content }));
    }
    messages[thread.id] = parsedMessages;

    const todoRows = await readJsonFiles(`${TODOS_DIR}/${thread.id}`);
    let parsedTodos = todoRows.map((row) => {
      const rec = row as Record<string, unknown>;
      return {
        id: String(rec.id ?? ""),
        content: String(rec.content ?? ""),
        status: toTodoStatus(rec.status),
      };
    });
    if (parsedTodos.length === 0 && seeded) {
      parsedTodos = seeded.todos.map((t) => ({ id: t.id, content: t.content, status: toTodoStatus(t.status) }));
    }
    todos[thread.id] = parsedTodos;

    const subagentRows = await readJsonFiles(`${SUBAGENTS_DIR}/${thread.id}`);
    let parsedSubagents = subagentRows.map((row) => {
      const rec = row as Record<string, unknown>;
      return {
        id: String(rec.id ?? ""),
        name: String(rec.name ?? ""),
        description: String(rec.description ?? ""),
        status: toSubagentStatus(rec.status),
      };
    });
    if (parsedSubagents.length === 0 && seeded) {
      parsedSubagents = seeded.subagents.map((s) => ({
        id: s.id,
        name: s.name,
        description: s.description,
        status: toSubagentStatus(s.status),
      }));
    }
    subagents[thread.id] = parsedSubagents;

    files[thread.id] = getMockThreadFiles(thread.id);
    tool_calls[thread.id] = getMockToolCalls(thread.id);
    tool_results[thread.id] = getMockToolResults(thread.id);

    const ws =
      index === 1 ? DEFAULT_WORKSPACE_ORDER[1] : index === 2 ? DEFAULT_WORKSPACE_ORDER[2] : DEFAULT_WORKSPACE_ORDER[0];
    workspace_path[thread.id] = ws;
  }

  return {
    threads,
    messages,
    todos,
    files,
    tool_calls,
    tool_results,
    subagents,
    workspace_path,
    workspace_files,
    providers,
    models,
    default_model: defaultModel,
  };
}

export async function createThread(): Promise<{ id: string; title: string; status: string; updated_at: string }> {
  await ensureZenFs();
  const id = `thread-${crypto.randomUUID().replace(/-/g, "")}`;
  const now = "now";
  const doc = {
    thread_id: id,
    created_at: now,
    updated_at: now,
    metadata: { title: "New Thread" },
    status: "Idle",
    values: null,
    messages: null,
  };
  await zenMkdir(THREADS_DIR, { recursive: true });
  await zenWriteFile(`${THREADS_DIR}/${id}.json`, encoder.encode(JSON.stringify(doc)));
  return { id, title: "New Thread", status: "Idle", updated_at: now };
}

export async function deleteThread(threadId: string): Promise<void> {
  await ensureZenFs();
  await removePath(`${THREADS_DIR}/${threadId}.json`, false);
  await removePath(`${MESSAGES_DIR}/${threadId}`, true);
  await removePath(`${TODOS_DIR}/${threadId}`, true);
  await removePath(`${SUBAGENTS_DIR}/${threadId}`, true);
}

export async function readProvidersWithKeys(): Promise<Array<{ id: ProviderId; name: string; has_api_key: boolean }>> {
  const env = await readEnvMap();
  const hasAny = Object.keys(env).length > 0;
  return providerDefs().map((p) => ({
    id: p.id,
    name: p.name,
    has_api_key: hasAny ? Boolean(env[`${p.prefix.toUpperCase()}_API_KEY`]) : p.id === "Anthropic" || p.id === "Ollama",
  }));
}

export async function getApiKey(provider: string): Promise<string> {
  await ensureZenFs();
  const env = await readEnvMap();
  return env[`${provider.toUpperCase()}_API_KEY`] ?? "";
}

export async function setApiKey(provider: string, value: string): Promise<void> {
  await ensureZenFs();
  const env = await readEnvMap();
  env[`${provider.toUpperCase()}_API_KEY`] = value;
  await writeEnvMap(env);
}

export async function deleteApiKey(provider: string): Promise<void> {
  await ensureZenFs();
  const env = await readEnvMap();
  delete env[`${provider.toUpperCase()}_API_KEY`];
  await writeEnvMap(env);
}

export async function setDefaultModel(modelId: string): Promise<void> {
  await ensureZenFs();
  await zenMkdir(CONFIG_DIR, { recursive: true });
  await zenWriteFile(DEFAULT_MODEL_FILE, encoder.encode(modelId));
}
