import {
  exists as zenExists,
  init as zenInit,
  mkdir as zenMkdir,
  readFile as zenReadFile,
  readdir as zenReadDir,
  rm as zenRm,
  writeFile as zenWriteFile,
} from "./omni-zenfs.js";

const THREADS_DIR = "/home/db/threads";
const MESSAGES_DIR = "/home/db/messages";
const TODOS_DIR = "/home/db/todos";
const SUBAGENTS_DIR = "/home/db/subagents";
const CONFIG_DIR = "/home/config";
const ENV_FILE = "/home/config/.env";
const DEFAULT_MODEL_FILE = "/home/config/default_model";

type ProviderId = "Anthropic" | "OpenAI" | "Google" | "Ollama";

type BootstrapPayload = {
  threads: Array<{ id: string; title: string; status: string; updated_at: string }>;
  messages: Record<string, Array<{ id: string; role: string; content: string }>>;
  todos: Record<string, Array<{ id: string; content: string; status: string }>>;
  subagents: Record<string, Array<{ id: string; name: string; description: string; status: string }>>;
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
  return String(id ?? "")
    .trim()
    .replace(/-/g, "")
    .toLowerCase();
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

async function writeText(path: string, content: string): Promise<void> {
  await zenMkdir(path.slice(0, path.lastIndexOf("/")), { recursive: true });
  await zenWriteFile(path, encoder.encode(content));
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

type SeedThread = {
  id: string;
  title: string;
  status: "Idle" | "Busy" | "Interrupted" | "Error";
  updated_at: string;
  messages: Array<{ id: string; role: "user" | "assistant" | "tool"; content: string; created_at: string }>;
  todos: Array<{ id: string; content: string; status: "pending" | "in_progress" | "completed" | "cancelled" }>;
  subagents: Array<{
    id: string;
    name: string;
    description: string;
    status: "pending" | "running" | "completed" | "failed";
  }>;
};

function seedThreads(): SeedThread[] {
  const now = Date.now();
  const iso = (offsetHours: number) => new Date(now - offsetHours * 3600_000).toISOString();
  return [
    {
      id: "d70f96097b314118bc8388bbdcdb5a46",
      title: "Implement todo management sys...",
      status: "Busy",
      updated_at: iso(0),
      messages: [
        {
          id: "m1",
          role: "user",
          content:
            "I need a full todo management system with CRUD operations, filtering by status, and persistence via ZenFS.",
          created_at: iso(0),
        },
        {
          id: "m2",
          role: "assistant",
          content: "I'll implement a comprehensive todo management system. Let me start by setting up the data layer.",
          created_at: iso(0),
        },
      ],
      todos: [
        { id: "todo1", content: "Design TodoStore data structure", status: "completed" },
        { id: "todo2", content: "Implement CRUD operations", status: "in_progress" },
        { id: "todo3", content: "Add ZenFS persistence", status: "pending" },
      ],
      subagents: [
        { id: "sa1", name: "FileWriter", description: "Writes and edits source files", status: "completed" },
        { id: "sa2", name: "TestRunner", description: "Runs test suite and reports results", status: "running" },
      ],
    },
    {
      id: "b5373aef87d94f5bad2af75dbcd4ec0f",
      title: "Implement Auth Flow",
      status: "Interrupted",
      updated_at: iso(16),
      messages: [
        {
          id: "m3",
          role: "user",
          content: "Set up JWT-based auth with refresh tokens and protected routes.",
          created_at: iso(16),
        },
      ],
      todos: [
        { id: "todo4", content: "Set up JWT middleware", status: "completed" },
        { id: "todo5", content: "Implement refresh token rotation", status: "in_progress" },
      ],
      subagents: [{ id: "sa3", name: "SecurityAuditor", description: "Audits auth flow", status: "pending" }],
    },
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
  await zenMkdir("/home/workspace", { recursive: true });
  if (!(await exists("/home/workspace/README.md"))) {
    await zenWriteFile(
      "/home/workspace/README.md",
      encoder.encode("# Workspace\n\nThis is a seeded workspace from the service worker data plane.\n"),
    );
  }
  await zenMkdir("/home/workspace/src", { recursive: true });
  if (!(await exists("/home/workspace/src/main.ts"))) {
    await zenWriteFile(
      "/home/workspace/src/main.ts",
      encoder.encode("export const hello = () => 'hello from workspace';\n"),
    );
  }

  const presetRoots = ["/home/user/projects/test", "/home/user/projects/omni", "/home/user/projects/omni-rt"];

  for (const root of presetRoots) {
    await zenMkdir(`${root}/src`, { recursive: true });
    if (!(await exists(`${root}/README.md`))) {
      await zenWriteFile(
        `${root}/README.md`,
        encoder.encode(`# ${root.split("/").pop()}\n\nSeeded workspace content.\n`),
      );
    }
    if (!(await exists(`${root}/src/index.ts`))) {
      await zenWriteFile(`${root}/src/index.ts`, encoder.encode("export const seeded = true;\n"));
    }
  }
}

async function listWorkspaceFiles(
  root: string,
): Promise<Array<{ path: string; is_dir: boolean; size: number | null }>> {
  await ensureZenFs();
  await ensureWorkspaceScaffold();
  const cleanRoot = root && root.startsWith("/") ? root : "/home/workspace";
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

async function getDefaultModel(): Promise<string> {
  if (!(await exists(DEFAULT_MODEL_FILE))) return "claude-3-7-sonnet";
  return (await readText(DEFAULT_MODEL_FILE)).trim() || "claude-3-7-sonnet";
}

async function buildBootstrap(): Promise<BootstrapPayload> {
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
    .sort((a, b) => b.updated_at.localeCompare(a.updated_at));

  const messages: BootstrapPayload["messages"] = {};
  const todos: BootstrapPayload["todos"] = {};
  const subagents: BootstrapPayload["subagents"] = {};

  for (const thread of threads) {
    const msgRows = await readJsonFiles(`${MESSAGES_DIR}/${thread.id}`);
    messages[thread.id] = msgRows
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

    const todoRows = await readJsonFiles(`${TODOS_DIR}/${thread.id}`);
    todos[thread.id] = todoRows.map((row) => {
      const rec = row as Record<string, unknown>;
      return {
        id: String(rec.id ?? ""),
        content: String(rec.content ?? ""),
        status: toTodoStatus(rec.status),
      };
    });

    const subagentRows = await readJsonFiles(`${SUBAGENTS_DIR}/${thread.id}`);
    subagents[thread.id] = subagentRows.map((row) => {
      const rec = row as Record<string, unknown>;
      return {
        id: String(rec.id ?? ""),
        name: String(rec.name ?? ""),
        description: String(rec.description ?? ""),
        status: toSubagentStatus(rec.status),
      };
    });
  }

  return {
    threads,
    messages,
    todos,
    subagents,
    providers,
    models,
    default_model: defaultModel,
  };
}

async function createThread(): Promise<{ id: string; title: string; status: string; updated_at: string }> {
  await ensureZenFs();
  const id = crypto.randomUUID().replace(/-/g, "");
  const now = new Date().toISOString();
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

async function deleteThread(threadId: string): Promise<void> {
  await ensureZenFs();
  await removePath(`${THREADS_DIR}/${threadId}.json`, false);
  await removePath(`${MESSAGES_DIR}/${threadId}`, true);
  await removePath(`${TODOS_DIR}/${threadId}`, true);
  await removePath(`${SUBAGENTS_DIR}/${threadId}`, true);
}

async function readProvidersWithKeys(): Promise<Array<{ id: ProviderId; name: string; has_api_key: boolean }>> {
  const env = await readEnvMap();
  return providerDefs().map((p) => ({
    id: p.id,
    name: p.name,
    has_api_key: Boolean(env[`${p.prefix.toUpperCase()}_API_KEY`]),
  }));
}

async function getApiKey(provider: string): Promise<string> {
  await ensureZenFs();
  const env = await readEnvMap();
  return env[`${provider.toUpperCase()}_API_KEY`] ?? "";
}

async function setApiKey(provider: string, value: string): Promise<void> {
  await ensureZenFs();
  const env = await readEnvMap();
  env[`${provider.toUpperCase()}_API_KEY`] = value;
  await writeEnvMap(env);
}

async function deleteApiKey(provider: string): Promise<void> {
  await ensureZenFs();
  const env = await readEnvMap();
  delete env[`${provider.toUpperCase()}_API_KEY`];
  await writeEnvMap(env);
}

async function setDefaultModel(modelId: string): Promise<void> {
  await ensureZenFs();
  await zenMkdir(CONFIG_DIR, { recursive: true });
  await zenWriteFile(DEFAULT_MODEL_FILE, encoder.encode(modelId));
}

export type StoreRoute =
  | "store-bootstrap"
  | "store-create-thread"
  | "store-delete-thread"
  | "store-providers"
  | "store-get-api-key"
  | "store-set-api-key"
  | "store-delete-api-key"
  | "store-get-default-model"
  | "store-set-default-model"
  | "store-files"
  | null;

export function matchStoreRoute(request: Request): StoreRoute {
  const url = new URL(request.url);
  if (url.pathname === "/api/store/bootstrap" && request.method === "GET") return "store-bootstrap";
  if (url.pathname === "/api/store/threads" && request.method === "POST") return "store-create-thread";
  if (url.pathname.startsWith("/api/store/threads/") && request.method === "DELETE") return "store-delete-thread";
  if (url.pathname === "/api/store/providers" && request.method === "GET") return "store-providers";
  if (url.pathname.startsWith("/api/store/config/api-keys/") && request.method === "GET") return "store-get-api-key";
  if (url.pathname.startsWith("/api/store/config/api-keys/") && request.method === "PUT") return "store-set-api-key";
  if (url.pathname.startsWith("/api/store/config/api-keys/") && request.method === "DELETE")
    return "store-delete-api-key";
  if (url.pathname === "/api/store/config/default-model" && request.method === "GET") return "store-get-default-model";
  if (url.pathname === "/api/store/config/default-model" && request.method === "PUT") return "store-set-default-model";
  if (url.pathname === "/api/store/files" && request.method === "GET") return "store-files";
  return null;
}

export async function handleStoreRoute(request: Request, route: Exclude<StoreRoute, null>): Promise<Response> {
  try {
    const url = new URL(request.url);

    if (route === "store-bootstrap") {
      return Response.json(await buildBootstrap());
    }

    if (route === "store-create-thread") {
      return Response.json(await createThread());
    }

    if (route === "store-delete-thread") {
      const threadId = normalizeThreadId(url.pathname.split("/").pop());
      await deleteThread(threadId);
      return new Response(null, { status: 204 });
    }

    if (route === "store-providers") {
      return Response.json(await readProvidersWithKeys());
    }

    if (route === "store-get-api-key") {
      const provider = url.pathname.split("/").pop() ?? "";
      return Response.json({ value: await getApiKey(provider) });
    }

    if (route === "store-set-api-key") {
      const provider = url.pathname.split("/").pop() ?? "";
      const body = (await request.json()) as { value?: string };
      await setApiKey(provider, body.value ?? "");
      return new Response(null, { status: 204 });
    }

    if (route === "store-delete-api-key") {
      const provider = url.pathname.split("/").pop() ?? "";
      await deleteApiKey(provider);
      return new Response(null, { status: 204 });
    }

    if (route === "store-get-default-model") {
      return Response.json({ model_id: await getDefaultModel() });
    }

    if (route === "store-set-default-model") {
      const body = (await request.json()) as { model_id?: string };
      if (!body.model_id) {
        return Response.json({ error: "model_id is required" }, { status: 400 });
      }
      await setDefaultModel(body.model_id);
      return new Response(null, { status: 204 });
    }

    if (route === "store-files") {
      const workspace = url.searchParams.get("workspace") ?? "/home/workspace";
      return Response.json(await listWorkspaceFiles(workspace));
    }

    return Response.json({ error: "Unknown route" }, { status: 404 });
  } catch (err) {
    const message =
      err instanceof Error ? `${err.name}: ${err.message}${err.stack ? `\n${err.stack}` : ""}` : String(err);
    return Response.json({ error: message }, { status: 500 });
  }
}
