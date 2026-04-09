import { workspaceSeedEntries } from "./deepagents.js";

export type SeedThread = {
  id: string;
  title: string;
  status: "Idle" | "Busy" | "Interrupted" | "Error";
  updated_at: string;
  workspace?: string;
  messages: Array<{ id: string; role: "user" | "assistant" | "tool"; content: string; created_at: string }>;
  todos: Array<{ id: string; content: string; status: "pending" | "in_progress" | "completed" | "cancelled" }>;
  subagents: Array<{
    id: string;
    name: string;
    description: string;
    status: "pending" | "running" | "completed" | "failed";
  }>;
};

export type AgentEndpoint = {
  id: string;
  url: string;
  bearer_token: string;
  name: string;
  removable: boolean;
};

const FNV_OFFSET_BASIS = 0xcbf29ce484222325n;
const FNV_PRIME = 0x100000001b3n;
const FNV_MASK = 0xffffffffffffffffn;

export function hashAgentConfig(url: string, bearerToken: string): string {
  const bytes = new TextEncoder().encode(`${url}\0${bearerToken}`);
  let hash = FNV_OFFSET_BASIS;
  for (const byte of bytes) {
    hash ^= BigInt(byte);
    hash = (hash * FNV_PRIME) & FNV_MASK;
  }
  return hash.toString(16).padStart(16, "0");
}

export function seedAgentEndpoints(): AgentEndpoint[] {
  return [
    {
      id: hashAgentConfig("https://agent.example.com/api", "sk-mock-1"),
      url: "https://agent.example.com/api",
      bearer_token: "sk-mock-1",
      name: "Research Agent",
      removable: true,
    },
    {
      id: hashAgentConfig("https://agent2.example.com/api", "sk-mock-2"),
      url: "https://agent2.example.com/api",
      bearer_token: "sk-mock-2",
      name: "Code Review Agent",
      removable: true,
    },
  ];
}
export const MOCK_THREAD_IDS = {
  gtd: "11111111-1111-4111-8111-111111111111",
  auth: "22222222-2222-4222-8222-222222222222",
  db: "33333333-3333-4333-8333-333333333333",
  ci: "44444444-4444-4444-8444-444444444444",
  idea: "55555555-5555-4555-8555-555555555555",
} as const;

export async function scaffoldFilesFromStore(): Promise<Array<{ path: string; content: string }>> {
  const entries = await workspaceSeedEntries();
  return entries.map((entry) => ({
    path: entry.path,
    content: entry.text ?? `seed:${entry.fixture ?? entry.path.split("/").pop() ?? "sample.txt"}\n`,
  }));
}

export async function getMockWorkspaceFiles(): Promise<
  Record<string, Array<{ path: string; is_dir: boolean; size: number | null }>>
> {
  const entries = await workspaceSeedEntries();
  const grouped = new Map<string, Array<{ path: string; is_dir: boolean; size: number | null }>>();

  for (const entry of entries) {
    const parts = entry.path.split("/").filter(Boolean);
    let root = "";
    if (parts[0] === "home" && parts[1] === "workspace") {
      root = "/home/workspace";
    } else if (parts[0] === "home" && parts[1] === "user" && parts[2] === "projects" && parts[3]) {
      root = `/home/user/projects/${parts[3]}`;
    }
    if (!root || entry.path.length <= root.length) continue;

    const relative = entry.path.slice(root.length + 1);

    // For the test project workspace, only show project files (not fixtures or README)
    if (root === "/home/user/projects/test") {
      if (relative.startsWith("fixtures/") || relative === "README.md") continue;
    }

    const list = grouped.get(root) ?? [];
    const seen = new Set(list.map((item) => item.path));
    const relativeParts = relative.split("/");
    let current = root;
    for (const segment of relativeParts.slice(0, -1)) {
      current = `${current}/${segment}`;
      if (!seen.has(current)) {
        list.push({ path: current, is_dir: true, size: null });
        seen.add(current);
      }
    }

    if (!seen.has(entry.path)) {
      list.push({ path: entry.path, is_dir: false, size: entry.size });
      seen.add(entry.path);
    }
    grouped.set(root, list);
  }

  return Object.fromEntries(
    Array.from(grouped.entries()).map(([root, list]) => [
      root,
      list.sort((left, right) => left.path.localeCompare(right.path)),
    ]),
  );
}

export function seedThreads(): SeedThread[] {
  const fourHoursAgo = new Date(Date.now() - 4 * 60 * 60 * 1000).toISOString();
  const messageTime = (minutesAfterStart: number) =>
    new Date(Date.now() - 4 * 60 * 60 * 1000 + minutesAfterStart * 60 * 1000).toISOString();

  const genericTodos: SeedThread["todos"] = [
    { id: "t1", content: "Wire dock slots", status: "completed" },
    { id: "t2", content: "Port sidebar interactions", status: "in_progress" },
    { id: "t3", content: "Implement model switcher", status: "pending" },
    { id: "t4", content: "File viewer tabs", status: "pending" },
    { id: "t5", content: "Settings dialog", status: "pending" },
  ];
  const genericSubagents: SeedThread["subagents"] = [
    {
      id: "sa-1",
      name: "Spec Auditor",
      description: "Cross-checks implementation against the phase plan",
      status: "running",
    },
    {
      id: "sa-2",
      name: "Test Synth",
      description: "Generates verification tests for touched modules",
      status: "completed",
    },
  ];

  return [
    {
      id: MOCK_THREAD_IDS.gtd,
      title: "New Thread",
      status: "Busy",
      updated_at: fourHoursAgo,
      workspace: "/home/user/projects/test",
      messages: [
        {
          id: "m1",
          role: "user",
          content:
            "Build a todo management system with three modes: GTD (Getting Things Done), Kanban, and Chaos Mode (random prioritization). Research and implement all three.",
          created_at: messageTime(0),
        },
      ],
      todos: [
        {
          id: "t1",
          content: "Research GTD (Getting Things Done) methodology using subagent",
          status: "in_progress",
        },
        { id: "t2", content: "Research Kanban methodology using subagent", status: "in_progress" },
        {
          id: "t3",
          content: "Research Chaos Mode (random prioritization) approach using subagent",
          status: "pending",
        },
        {
          id: "t4",
          content: "Design data structure and API endpoints for three todo management systems",
          status: "pending",
        },
        { id: "t5", content: "Implement GTD backend endpoints and logic in server.js", status: "pending" },
        { id: "t6", content: "Implement Kanban backend endpoints and logic in server.js", status: "pending" },
        { id: "t7", content: "Implement Chaos Mode backend endpoints and logic in server.js", status: "pending" },
        { id: "t8", content: "Create frontend UI for GTD system", status: "pending" },
        { id: "t9", content: "Create frontend UI for Kanban system", status: "pending" },
        { id: "t10", content: "Create frontend UI for Chaos Mode system", status: "pending" },
        { id: "t11", content: "Update README.md with documentation for new systems", status: "pending" },
      ],
      subagents: [
        {
          id: "sa-gtd-1",
          name: "General Purpose Agent",
          description:
            "Research the GTD (Getting Things Done) methodology by David Allen. I need you to provide a comprehensive report covering: 1. Core principles and philosophy of GTD 2. The key components: Inbox, Next Actions, Projects, Waiting For, Someday/Maybe, Contexts 3. The 5 stages of workflow: Capture, Clarify, Organize, Reflect, Engage",
          status: "running",
        },
        {
          id: "sa-gtd-2",
          name: "General Purpose Agent",
          description:
            "Research the Kanban methodology for task management. I need you to provide a comprehensive report covering the core principles, board setup, WIP limits, and flow metrics.",
          status: "running",
        },
        {
          id: "sa-gtd-3",
          name: "General Purpose Agent",
          description:
            "Research and design a Chaos Mode todo management system based on random prioritization and unpredictable task ordering. I need you to provide a creative report...",
          status: "pending",
        },
      ],
    },
    {
      id: MOCK_THREAD_IDS.auth,
      title: "New Thread",
      status: "Interrupted",
      updated_at: fourHoursAgo,
      messages: [
        {
          id: "m1",
          role: "user",
          content: "Ship OAuth login with refresh rotation and audit trail.",
          created_at: messageTime(1),
        },
        {
          id: "m2",
          role: "assistant",
          content: "Copy. I will update auth middleware, add token persistence, and run smoke tests.",
          created_at: messageTime(2),
        },
      ],
      todos: genericTodos,
      subagents: genericSubagents,
    },
    {
      id: MOCK_THREAD_IDS.db,
      title: "New Thread",
      status: "Idle",
      updated_at: fourHoursAgo,
      messages: [
        { id: "m1", role: "user", content: "Need migration plan for v3 schema.", created_at: messageTime(3) },
        {
          id: "m2",
          role: "assistant",
          content: "Migration paused pending approval to write production scripts.",
          created_at: messageTime(4),
        },
      ],
      todos: genericTodos,
      subagents: genericSubagents,
    },
    {
      id: MOCK_THREAD_IDS.ci,
      title: "New Thread",
      status: "Idle",
      updated_at: fourHoursAgo,
      messages: [],
      todos: genericTodos,
      subagents: genericSubagents,
    },
    {
      id: MOCK_THREAD_IDS.idea,
      title: "New Thread",
      status: "Idle",
      updated_at: fourHoursAgo,
      messages: [],
      todos: genericTodos,
      subagents: genericSubagents,
    },
  ];
}

export function getMockThreadFiles(threadId: string): Array<{ path: string; is_dir: boolean; size: number | null }> {
  if (threadId === MOCK_THREAD_IDS.gtd) {
    return [
      { path: "public", is_dir: true, size: null },
      { path: "public/app.js", is_dir: false, size: 2600 },
      { path: "public/index.html", is_dir: false, size: 6900 },
      { path: "public/styles.css", is_dir: false, size: 3400 },
      { path: "scripts", is_dir: true, size: null },
      { path: "scripts/flush_todos.js", is_dir: false, size: 381 },
      { path: "server", is_dir: true, size: null },
      { path: "server/server.js", is_dir: false, size: 850 },
      { path: "server/todos.json", is_dir: false, size: 314 },
    ];
  }
  return [
    { path: "src", is_dir: true, size: null },
    { path: "src/main.rs", is_dir: false, size: 9612 },
    { path: "src/components/chat/mod.rs", is_dir: false, size: 14020 },
    { path: "src/lib/mod.rs", is_dir: false, size: 7903 },
    { path: "README.md", is_dir: false, size: 4089 },
  ];
}

export function getMockToolCalls(threadId: string): Array<{ id: string; name: string; args: unknown }> {
  if (threadId !== MOCK_THREAD_IDS.gtd) return [];
  return [
    {
      id: "tc-todos",
      name: "update_todos",
      args: {
        todos: [
          { content: "Research GTD (Getting Things Done) methodology using subagent", status: "in_progress" },
          { content: "Research Kanban methodology using subagent", status: "in_progress" },
          { content: "Research Chaos Mode (random prioritization) approach using subagent", status: "pending" },
          { content: "Design data structure and API endpoints for three todo management systems", status: "pending" },
          { content: "Implement GTD backend endpoints and logic in server.js", status: "pending" },
          { content: "Implement Kanban backend endpoints and logic in server.js", status: "pending" },
          { content: "Implement Chaos Mode backend endpoints and logic in server.js", status: "pending" },
          { content: "Create frontend UI for GTD system", status: "pending" },
          { content: "Create frontend UI for Kanban system", status: "pending" },
          { content: "Create frontend UI for Chaos Mode system", status: "pending" },
          { content: "Update README.md with documentation for new systems", status: "pending" },
        ],
      },
    },
    {
      id: "tc-sa1",
      name: "dispatch_subagent",
      args: {
        task: "Research the GTD (Getting Things Done) methodology by David Allen. I need you to provide a comprehensive report covering: 1. Core principles and philosophy of GTD 2. The key components: Inbox, Next Actions, Projects, Waiting For, Someday/Maybe, Contexts 3. The 5 stages of workflow: Capture, Clarify, Organize, Reflect, Engage",
      },
    },
    {
      id: "tc-sa2",
      name: "dispatch_subagent",
      args: {
        task: "Research the Kanban methodology for task management. I need you to provide a comprehensive report covering: 1. Core principles and philosophy of Kanban 2. The...",
      },
    },
    {
      id: "tc-sa3",
      name: "dispatch_subagent",
      args: {
        task: 'Research and design a "Chaos Mode" todo management system based on random prioritization and unpredictable task ordering. I need you to provide a creative repor...',
      },
    },
  ];
}

export function getMockToolResults(
  threadId: string,
): Array<{ tool_call_id: string; content: string; is_error: boolean }> {
  if (threadId !== MOCK_THREAD_IDS.gtd) return [];
  return [{ tool_call_id: "tc-todos", content: "Synced", is_error: false }];
}
