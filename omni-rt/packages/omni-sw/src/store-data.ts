import {
  exists as zenExists,
  init as zenInit,
  mkdir as zenMkdir,
  readFile as zenReadFile,
  readdir as zenReadDir,
  rm as zenRm,
  writeFile as zenWriteFile,
} from "./zenfs.js";
import {
  createThread as createDeepagentsThread,
  deleteApiKey as deleteDeepagentsApiKey,
  deleteDefaultModel as deleteDeepagentsDefaultModel,
  deleteThread as deleteDeepagentsThread,
  deleteThreadMessages,
  getApiKey as getDeepagentsApiKey,
  getDefaultModel as getDeepagentsDefaultModel,
  getStoredDefaultModel as getDeepagentsStoredDefaultModel,
  listMessages as listDeepagentsMessages,
  listThreads as listDeepagentsThreads,
  saveMessage as saveDeepagentsMessage,
  setApiKey as setDeepagentsApiKey,
  setDefaultModel as setDeepagentsDefaultModel,
  setThreadStatus as setDeepagentsThreadStatus,
} from "./deepagents.js";
import {
  DEFAULT_WORKSPACE_ORDER,
  MOCK_THREAD_IDS,
  SCAFFOLD_FILES,
  getMockThreadFiles,
  getMockToolCalls,
  getMockToolResults,
  getMockWorkspaceFiles,
  seedAgentEndpoints,
  seedThreads,
} from "./store-mocks.js";

const TODOS_DIR = "/home/db/todos";
const SUBAGENTS_DIR = "/home/db/subagents";
const AGENT_ENDPOINTS_DIR = "/home/store/config/agent-endpoints";
const AGENT_RAIL_DIR = "/home/store/config/agent-rail";
const ALLOWED_DICEBEAR_STYLES = new Set(["bottts-neutral", "thumbs"]);
export const DEFAULT_THREAD_TITLE = "New Thread";

type ProviderId = "Anthropic" | "OpenAI" | "Google" | "Ollama";

export type BootstrapPayload = {
  threads: Array<{ id: string; title: string; status: string; updated_at: string }>;
  messages: Record<string, Array<{ id: string; role: string; content: string }>>;
  todos: Record<string, Array<{ id: string; content: string; status: string }>>;
  files: Record<string, Array<{ path: string; is_dir: boolean; size: number | null }>>;
  tool_calls: Record<string, Array<{ id: string; name: string; args: unknown }>>;
  tool_results: Record<string, Array<{ tool_call_id: string; content: string; is_error: boolean }>>;
  background_tasks: Record<string, Array<{ id: string; name: string; description: string; status: string }>>;
  workspace_path: Record<string, string>;
  workspace_files: Record<string, Array<{ path: string; is_dir: boolean; size: number | null }>>;
  providers: Array<{ id: ProviderId; name: string; has_api_key: boolean }>;
  models: Array<{ id: string; name: string; provider: ProviderId }>;
  default_model: string;
  dicebear_style: string;
  agent_endpoints: Array<{
    id: string;
    url: string;
    bearer_token: string;
    name: string;
    removable: boolean;
  }>;
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

export function isPlaceholderThreadTitle(title: unknown): boolean {
  const value = String(title ?? "").trim();
  return value.length === 0 || value === DEFAULT_THREAD_TITLE;
}

function messageContentText(content: unknown): string {
  if (typeof content === "string") {
    return content;
  }
  if (Array.isArray(content)) {
    return content
      .map((entry) => {
        if (typeof entry === "string") {
          return entry;
        }
        if (entry && typeof entry === "object" && "text" in entry && typeof entry.text === "string") {
          return entry.text;
        }
        return "";
      })
      .filter(Boolean)
      .join(" ");
  }
  if (content && typeof content === "object" && "text" in content && typeof content.text === "string") {
    return content.text;
  }
  return "";
}

function summarizeThreadTitle(content: unknown): string | null {
  const normalized = messageContentText(content).replace(/\s+/g, " ").trim();
  if (!normalized) {
    return null;
  }
  if (normalized.length <= 48) {
    return normalized;
  }
  return `${normalized.slice(0, 45).trimEnd()}...`;
}

export function deriveThreadTitle(
  messages: Array<{ role?: unknown; content?: unknown }>,
  currentTitle?: unknown,
): string {
  if (!isPlaceholderThreadTitle(currentTitle)) {
    return String(currentTitle ?? "").trim();
  }

  for (const message of messages) {
    if (String(message.role ?? "").toLowerCase() !== "user") {
      continue;
    }
    const summary = summarizeThreadTitle(message.content);
    if (summary) {
      return summary;
    }
  }

  return DEFAULT_THREAD_TITLE;
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
  const existing = await listDeepagentsThreads();
  if (existing.length === 0) {
    for (const thread of seedThreads()) {
      await createDeepagentsThread({
        thread_id: thread.id,
        metadata: { title: thread.title },
      });
      await setDeepagentsThreadStatus(thread.id, thread.status);

      for (const msg of thread.messages) {
        await saveDeepagentsMessage(thread.id, thread.updated_at, { ...msg, thread_id: thread.id });
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
  await seedMockAgentEndpoints();
}

async function seedMockAgentEndpoints(): Promise<void> {
  await zenMkdir(AGENT_ENDPOINTS_DIR, { recursive: true });
  const entries = (await zenReadDir(AGENT_ENDPOINTS_DIR).catch(() => [])) as DirEntry[];
  if (entries.some((entry) => String(entry.name ?? "").endsWith(".json"))) {
    return;
  }

  const now = new Date().toISOString();
  for (const endpoint of seedAgentEndpoints()) {
    await zenWriteFile(
      `${AGENT_ENDPOINTS_DIR}/${endpoint.id}.json`,
      encoder.encode(
        JSON.stringify({
          namespace: ["config", "agent-endpoints"],
          key: endpoint.id,
          value: endpoint,
          created_at: now,
          updated_at: now,
        }),
      ),
    );
  }
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

export async function getDefaultModel(): Promise<string> {
  return await getDeepagentsDefaultModel();
}

export async function getStoredDefaultModel(): Promise<string | null> {
  return await getDeepagentsStoredDefaultModel();
}

export async function buildBootstrap(): Promise<BootstrapPayload> {
  await ensureZenFs();
  await seedIfEmpty();

  const defaultModel = await getDefaultModel();
  const providers = await readProvidersWithKeys();
  const models = modelDefs();

  const threadRows = await listDeepagentsThreads();
  const threads = threadRows
    .map((row) => {
      const rec = row as Record<string, unknown>;
      const id = normalizeThreadId(rec.thread_id ?? rec.id);
      const metadata = (rec.metadata as Record<string, unknown> | undefined) ?? {};
      return {
        id,
        title: String(metadata.title ?? DEFAULT_THREAD_TITLE),
        status: toThreadStatus(rec.status),
        updated_at: String(rec.updated_at ?? new Date().toISOString()),
      };
    })
    .filter((t) => t.id.length > 0)
    .sort((a, b) => {
      const order: string[] = [
        MOCK_THREAD_IDS.gtd,
        MOCK_THREAD_IDS.auth,
        MOCK_THREAD_IDS.db,
        MOCK_THREAD_IDS.ci,
        MOCK_THREAD_IDS.idea,
      ];
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
  const background_tasks: BootstrapPayload["background_tasks"] = {};
  const workspace_path: BootstrapPayload["workspace_path"] = {};
  const workspace_files = getMockWorkspaceFiles();
  const railStyleItem = (await readJson(`${AGENT_RAIL_DIR}/dicebear-style.json`)) as Record<string, unknown> | null;
  const storedDicebearStyle = String(
    (railStyleItem?.value as Record<string, unknown> | undefined)?.style ?? railStyleItem?.style ?? "bottts-neutral",
  );
  const dicebearStyle = ALLOWED_DICEBEAR_STYLES.has(storedDicebearStyle) ? storedDicebearStyle : "bottts-neutral";
  const agentEndpoints = (await readJsonFiles(AGENT_ENDPOINTS_DIR))
    .map((row) => {
      const rec = row as Record<string, unknown>;
      const value = (rec.value as Record<string, unknown> | undefined) ?? rec;
      return {
        id: String(value.id ?? ""),
        url: String(value.url ?? ""),
        bearer_token: String(value.bearer_token ?? ""),
        name: String(value.name ?? ""),
        removable: value.removable !== false,
      };
    })
    .filter((endpoint) => endpoint.id.length > 0 && endpoint.removable);

  const seedById = new Map(seedThreads().map((t) => [t.id, t]));

  for (const [index, thread] of threads.entries()) {
    const seeded = seedById.get(thread.id);

    const msgRows = await listDeepagentsMessages(thread.id);
    let parsedMessages = msgRows
      .map((row) => {
        const rec = row as Record<string, unknown>;
        return {
          id: String(rec.id ?? ""),
          role: toRole(rec.role),
          content: typeof rec.content === "string" ? rec.content : JSON.stringify(rec.content ?? ""),
          created_at: String(rec.created_at ?? ""),
        };
      })
      .sort((a, b) => a.created_at.localeCompare(b.created_at))
      .map(({ id, role, content }) => ({ id, role, content }));
    if (parsedMessages.length === 0 && seeded) {
      parsedMessages = seeded.messages.map((m) => ({ id: m.id, role: toRole(m.role), content: m.content }));
    }
    thread.title = deriveThreadTitle(parsedMessages, thread.title);
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
    background_tasks[thread.id] = parsedSubagents;

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
    background_tasks,
    workspace_path,
    workspace_files,
    providers,
    models,
    default_model: defaultModel,
    dicebear_style: dicebearStyle,
    agent_endpoints: agentEndpoints,
  };
}

export async function createThread(): Promise<{ id: string; title: string; status: string; updated_at: string }> {
  const thread = await createDeepagentsThread({ metadata: { title: DEFAULT_THREAD_TITLE } });
  return {
    id: String(thread.thread_id ?? ""),
    title: String((thread.metadata as Record<string, unknown> | undefined)?.title ?? DEFAULT_THREAD_TITLE),
    status: toThreadStatus(thread.status),
    updated_at: String(thread.updated_at ?? new Date().toISOString()),
  };
}

export async function deleteThread(threadId: string): Promise<void> {
  await deleteDeepagentsThread(threadId);
  await deleteThreadMessages(threadId);
  await removePath(`${TODOS_DIR}/${threadId}`, true);
  await removePath(`${SUBAGENTS_DIR}/${threadId}`, true);
}

export async function readProvidersWithKeys(): Promise<Array<{ id: ProviderId; name: string; has_api_key: boolean }>> {
  const providers = await Promise.all(
    providerDefs().map(async (provider) => ({
      id: provider.id,
      name: provider.name,
      has_api_key: Boolean(await getDeepagentsApiKey(provider.prefix)),
    })),
  );
  return providers.map((provider) => ({
    ...provider,
    has_api_key: provider.has_api_key || provider.id === "Ollama",
  }));
}

export async function getApiKey(provider: string): Promise<string> {
  return (await getDeepagentsApiKey(provider)) ?? "";
}

export async function setApiKey(provider: string, value: string): Promise<void> {
  await setDeepagentsApiKey(provider, value);
}

export async function deleteApiKey(provider: string): Promise<void> {
  await deleteDeepagentsApiKey(provider);
}

export async function setDefaultModel(modelId: string): Promise<void> {
  await setDeepagentsDefaultModel(modelId);
}

export async function deleteDefaultModel(): Promise<void> {
  await deleteDeepagentsDefaultModel();
}
