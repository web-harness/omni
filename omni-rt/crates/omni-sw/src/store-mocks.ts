export type SeedThread = {
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

export const DEFAULT_WORKSPACE_ORDER = [
  "/home/user/projects/test",
  "/home/user/projects/omni",
  "/home/user/projects/omni-rt",
];

export const SCAFFOLD_FILES: Array<{ path: string; content: string }> = [
  { path: "/home/workspace/README.md", content: "# Workspace\n" },
  { path: "/home/user/projects/test/public/app.js", content: "console.log('app');\n" },
  {
    path: "/home/user/projects/test/public/index.html",
    content: "<!doctype html><html><body>test</body></html>\n",
  },
  { path: "/home/user/projects/test/public/styles.css", content: "body { font-family: monospace; }\n" },
  {
    path: "/home/user/projects/test/scripts/flush_todos.js",
    content: "#!/usr/bin/env node\nconsole.log('flush todos');\n",
  },
  { path: "/home/user/projects/test/server/server.js", content: "export const server = true;\n" },
  { path: "/home/user/projects/test/server/todos.json", content: '{\n  "todos": []\n}\n' },
  { path: "/home/user/projects/test/fixtures/sample.rs", content: 'fn main() { println!("sample"); }\n' },
  { path: "/home/user/projects/test/fixtures/sample.ts", content: "export const sample = true;\n" },
  { path: "/home/user/projects/test/fixtures/sample.py", content: "print('sample')\n" },
  { path: "/home/user/projects/test/fixtures/sample.sh", content: "#!/usr/bin/env bash\necho sample\n" },
  { path: "/home/user/projects/test/fixtures/sample.md", content: "# Sample\n" },
  { path: "/home/user/projects/test/fixtures/sample.html", content: "<!doctype html><html></html>\n" },
  { path: "/home/user/projects/test/fixtures/sample.css", content: "body{}\n" },
  { path: "/home/user/projects/test/fixtures/sample.json", content: '{"ok":true}\n' },
  { path: "/home/user/projects/test/fixtures/sample.toml", content: 'name = "sample"\n' },
  { path: "/home/user/projects/test/fixtures/sample.txt", content: "sample\n" },
  {
    path: "/home/user/projects/test/fixtures/sample.svg",
    content: '<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"></svg>\n',
  },
  { path: "/home/user/projects/test/fixtures/sample.png", content: "png\n" },
  { path: "/home/user/projects/test/fixtures/sample.jpg", content: "jpg\n" },
  { path: "/home/user/projects/test/fixtures/sample.pdf", content: "%PDF-1.4\n" },
  { path: "/home/user/projects/test/fixtures/sample.wav", content: "RIFF\n" },
  { path: "/home/user/projects/test/fixtures/sample.mp3", content: "ID3\n" },
  { path: "/home/user/projects/test/fixtures/sample.mp4", content: "mp4\n" },
  { path: "/home/user/projects/test/fixtures/sample.bin", content: "bin\n" },
  { path: "/home/user/projects/omni/src/main.rs", content: "fn main() {}\n" },
  { path: "/home/user/projects/omni/src/components/chat/mod.rs", content: "pub fn chat() {}\n" },
  { path: "/home/user/projects/omni/src/components/sidebar/mod.rs", content: "pub fn sidebar() {}\n" },
  { path: "/home/user/projects/omni/src/lib/mod.rs", content: "pub mod sample;\n" },
  { path: "/home/user/projects/omni/Cargo.toml", content: '[package]\nname = "omni"\n' },
  { path: "/home/user/projects/omni/README.md", content: "# omni\n" },
  { path: "/home/user/projects/omni-rt/crates/omni-protocol/src/lib.rs", content: "pub struct Protocol;\n" },
  { path: "/home/user/projects/omni-rt/crates/omni-rt/src/main.rs", content: "fn main() {}\n" },
  {
    path: "/home/user/projects/omni-rt/crates/omni-dock/src/omni-dock.ts",
    content: "export const dock = true;\n",
  },
  { path: "/home/user/projects/omni-rt/Cargo.toml", content: "[workspace]\n" },
];

export function seedThreads(): SeedThread[] {
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
      description: "Cross-checks implementation against phase plan",
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
      id: "thread-gtd",
      title: "Implement todo management sys...",
      status: "Busy",
      updated_at: "9m ago",
      messages: [
        {
          id: "m1",
          role: "user",
          content:
            "Build a todo management system with three modes: GTD (Getting Things Done), Kanban, and Chaos Mode (random prioritization). Research and implement all three.",
          created_at: "1",
        },
      ],
      todos: [
        { id: "t1", content: "Research GTD (Getting Things Done) methodology using subagent", status: "in_progress" },
        { id: "t2", content: "Research Kanban methodology using subagent", status: "in_progress" },
        { id: "t3", content: "Research Chaos Mode (random prioritization) approach using subagent", status: "pending" },
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
      id: "thread-auth",
      title: "Implement Auth Flow",
      status: "Interrupted",
      updated_at: "49m ago",
      messages: [
        { id: "m1", role: "user", content: "Ship OAuth login with refresh rotation and audit trail.", created_at: "1" },
        {
          id: "m2",
          role: "assistant",
          content: "Copy. I will update auth middleware, add token persistence, and run smoke tests.",
          created_at: "2",
        },
      ],
      todos: genericTodos,
      subagents: genericSubagents,
    },
    {
      id: "thread-db",
      title: "Database Migration",
      status: "Idle",
      updated_at: "52m ago",
      messages: [
        { id: "m1", role: "user", content: "Need migration plan for v3 schema.", created_at: "1" },
        {
          id: "m2",
          role: "assistant",
          content: "Migration paused pending approval to write production scripts.",
          created_at: "2",
        },
      ],
      todos: genericTodos,
      subagents: genericSubagents,
    },
    {
      id: "thread-ci",
      title: "Setup CI Pipeline",
      status: "Idle",
      updated_at: "58m ago",
      messages: [],
      todos: genericTodos,
      subagents: genericSubagents,
    },
    {
      id: "thread-idea",
      title: "What would be a good...",
      status: "Idle",
      updated_at: "1h ago",
      messages: [],
      todos: genericTodos,
      subagents: genericSubagents,
    },
  ];
}

export function getMockThreadFiles(threadId: string): Array<{ path: string; is_dir: boolean; size: number | null }> {
  if (threadId === "thread-gtd") {
    return [
      { path: "public", is_dir: true, size: null },
      { path: "public/app.js", is_dir: false, size: 2600 },
      { path: "public/index.html", is_dir: false, size: 6900 },
      { path: "public/styles.css", is_dir: false, size: 3400 },
      { path: "scripts", is_dir: true, size: null },
      { path: "scripts/flush_todos_node_script.js", is_dir: false, size: 381 },
      { path: "server", is_dir: true, size: null },
      { path: "server/server.js", is_dir: false, size: 850 },
      { path: "server/todos.json", is_dir: false, size: 314 },
      { path: "test2", is_dir: true, size: null },
      { path: "test2/hello_french.txt", is_dir: false, size: 78 },
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

export function getMockWorkspaceFiles(): Record<string, Array<{ path: string; is_dir: boolean; size: number | null }>> {
  return {
    "/home/workspace": [
      { path: "public", is_dir: true, size: null },
      { path: "public/app.js", is_dir: false, size: 2600 },
      { path: "public/index.html", is_dir: false, size: 6900 },
      { path: "public/styles.css", is_dir: false, size: 3400 },
      { path: "scripts", is_dir: true, size: null },
      { path: "scripts/flush_todos.js", is_dir: false, size: 381 },
      { path: "server", is_dir: true, size: null },
      { path: "server/server.js", is_dir: false, size: 850 },
      { path: "server/todos.json", is_dir: false, size: 314 },
      { path: "fixtures", is_dir: true, size: null },
      { path: "fixtures/sample.rs", is_dir: false, size: 512 },
      { path: "fixtures/sample.ts", is_dir: false, size: 620 },
      { path: "fixtures/sample.py", is_dir: false, size: 580 },
      { path: "fixtures/sample.sh", is_dir: false, size: 410 },
      { path: "fixtures/sample.md", is_dir: false, size: 740 },
      { path: "fixtures/sample.html", is_dir: false, size: 890 },
      { path: "fixtures/sample.css", is_dir: false, size: 660 },
      { path: "fixtures/sample.json", is_dir: false, size: 520 },
      { path: "fixtures/sample.toml", is_dir: false, size: 280 },
      { path: "fixtures/sample.txt", is_dir: false, size: 940 },
      { path: "fixtures/sample.svg", is_dir: false, size: 480 },
      { path: "fixtures/sample.png", is_dir: false, size: 120 },
      { path: "fixtures/sample.jpg", is_dir: false, size: 120 },
      { path: "fixtures/sample.pdf", is_dir: false, size: 800 },
      { path: "fixtures/sample.wav", is_dir: false, size: 46 },
      { path: "fixtures/sample.mp3", is_dir: false, size: 64 },
      { path: "fixtures/sample.mp4", is_dir: false, size: 256 },
      { path: "fixtures/sample.bin", is_dir: false, size: 128 },
    ],
    "/home/user/projects/test": [
      { path: "public", is_dir: true, size: null },
      { path: "public/app.js", is_dir: false, size: 2600 },
      { path: "public/index.html", is_dir: false, size: 6900 },
      { path: "public/styles.css", is_dir: false, size: 3400 },
      { path: "scripts", is_dir: true, size: null },
      { path: "scripts/flush_todos.js", is_dir: false, size: 381 },
      { path: "server", is_dir: true, size: null },
      { path: "server/server.js", is_dir: false, size: 850 },
      { path: "server/todos.json", is_dir: false, size: 314 },
      { path: "fixtures", is_dir: true, size: null },
      { path: "fixtures/sample.rs", is_dir: false, size: 512 },
      { path: "fixtures/sample.ts", is_dir: false, size: 620 },
      { path: "fixtures/sample.py", is_dir: false, size: 580 },
      { path: "fixtures/sample.sh", is_dir: false, size: 410 },
      { path: "fixtures/sample.md", is_dir: false, size: 740 },
      { path: "fixtures/sample.html", is_dir: false, size: 890 },
      { path: "fixtures/sample.css", is_dir: false, size: 660 },
      { path: "fixtures/sample.json", is_dir: false, size: 520 },
      { path: "fixtures/sample.toml", is_dir: false, size: 280 },
      { path: "fixtures/sample.txt", is_dir: false, size: 940 },
      { path: "fixtures/sample.svg", is_dir: false, size: 480 },
      { path: "fixtures/sample.png", is_dir: false, size: 120 },
      { path: "fixtures/sample.jpg", is_dir: false, size: 120 },
      { path: "fixtures/sample.pdf", is_dir: false, size: 800 },
      { path: "fixtures/sample.wav", is_dir: false, size: 46 },
      { path: "fixtures/sample.mp3", is_dir: false, size: 64 },
      { path: "fixtures/sample.mp4", is_dir: false, size: 256 },
      { path: "fixtures/sample.bin", is_dir: false, size: 128 },
    ],
    "/home/user/projects/omni": [
      { path: "src", is_dir: true, size: null },
      { path: "src/main.rs", is_dir: false, size: 9612 },
      { path: "src/components", is_dir: true, size: null },
      { path: "src/components/chat/mod.rs", is_dir: false, size: 14020 },
      { path: "src/components/sidebar/mod.rs", is_dir: false, size: 3400 },
      { path: "src/lib", is_dir: true, size: null },
      { path: "src/lib/mod.rs", is_dir: false, size: 7903 },
      { path: "Cargo.toml", is_dir: false, size: 1200 },
      { path: "README.md", is_dir: false, size: 4089 },
    ],
    "/home/user/projects/omni-rt": [
      { path: "crates", is_dir: true, size: null },
      { path: "crates/omni-protocol", is_dir: true, size: null },
      { path: "crates/omni-protocol/src/lib.rs", is_dir: false, size: 5120 },
      { path: "crates/omni-rt", is_dir: true, size: null },
      { path: "crates/omni-rt/src/main.rs", is_dir: false, size: 3800 },
      { path: "crates/omni-dock", is_dir: true, size: null },
      { path: "crates/omni-dock/src/omni-dock.ts", is_dir: false, size: 8200 },
      { path: "Cargo.toml", is_dir: false, size: 980 },
    ],
  };
}

export function getMockToolCalls(threadId: string): Array<{ id: string; name: string; args: unknown }> {
  if (threadId !== "thread-gtd") return [];
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
  if (threadId !== "thread-gtd") return [];
  return [{ tool_call_id: "tc-todos", content: "Synced", is_error: false }];
}
