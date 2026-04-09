use crate::workspace_seed;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// Fixed thread IDs shared between web (store-mocks.ts) and native (seed.rs).
pub const THREAD_ID_GTD: &str = "11111111-1111-4111-8111-111111111111";
pub const THREAD_ID_AUTH: &str = "22222222-2222-4222-8222-222222222222";
pub const THREAD_ID_DB: &str = "33333333-3333-4333-8333-333333333333";
pub const THREAD_ID_CI: &str = "44444444-4444-4444-8444-444444444444";
pub const THREAD_ID_IDEA: &str = "55555555-5555-4555-8555-555555555555";

#[derive(Serialize)]
pub struct MockThreadIds {
    pub gtd: &'static str,
    pub auth: &'static str,
    pub db: &'static str,
    pub ci: &'static str,
    pub idea: &'static str,
}

pub fn mock_thread_ids() -> MockThreadIds {
    MockThreadIds {
        gtd: THREAD_ID_GTD,
        auth: THREAD_ID_AUTH,
        db: THREAD_ID_DB,
        ci: THREAD_ID_CI,
        idea: THREAD_ID_IDEA,
    }
}

// --- FNV-1a hash (matches web store-mocks.ts hashAgentConfig) ---

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

pub fn hash_agent_config(url: &str, bearer_token: &str) -> String {
    let input = format!("{url}\0{bearer_token}");
    let mut hash = FNV_OFFSET_BASIS;
    for byte in input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}

// --- Seed agent endpoints ---

#[derive(Serialize)]
pub struct SeedAgentEndpoint {
    pub id: String,
    pub url: String,
    pub bearer_token: String,
    pub name: String,
    pub removable: bool,
}

pub fn seed_agent_endpoints() -> Vec<SeedAgentEndpoint> {
    vec![
        SeedAgentEndpoint {
            id: hash_agent_config("https://agent.example.com/api", "sk-mock-1"),
            url: "https://agent.example.com/api".into(),
            bearer_token: "sk-mock-1".into(),
            name: "Research Agent".into(),
            removable: true,
        },
        SeedAgentEndpoint {
            id: hash_agent_config("https://agent2.example.com/api", "sk-mock-2"),
            url: "https://agent2.example.com/api".into(),
            bearer_token: "sk-mock-2".into(),
            name: "Code Review Agent".into(),
            removable: true,
        },
    ]
}

// --- Seed thread data (matches web seedThreads()) ---

#[derive(Serialize)]
pub struct SeedMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Clone, Serialize)]
pub struct SeedTodo {
    pub id: String,
    pub content: String,
    pub status: String,
}

#[derive(Clone, Serialize)]
pub struct SeedSubagent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct SeedThread {
    pub id: String,
    pub title: String,
    pub status: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    pub messages: Vec<SeedMessage>,
    pub todos: Vec<SeedTodo>,
    pub subagents: Vec<SeedSubagent>,
}

pub fn seed_threads() -> Vec<SeedThread> {
    let four_hours_ago = chrono::Utc::now() - chrono::Duration::hours(4);
    let base_ms = four_hours_ago.timestamp_millis();
    let four_hours_ago_str = four_hours_ago.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let message_time = |minutes_after: i64| -> String {
        let ts = chrono::DateTime::from_timestamp_millis(base_ms + minutes_after * 60 * 1000)
            .unwrap_or(four_hours_ago);
        ts.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    };

    let generic_todos: Vec<SeedTodo> = vec![
        SeedTodo {
            id: "t1".into(),
            content: "Wire dock slots".into(),
            status: "completed".into(),
        },
        SeedTodo {
            id: "t2".into(),
            content: "Port sidebar interactions".into(),
            status: "in_progress".into(),
        },
        SeedTodo {
            id: "t3".into(),
            content: "Implement model switcher".into(),
            status: "pending".into(),
        },
        SeedTodo {
            id: "t4".into(),
            content: "File viewer tabs".into(),
            status: "pending".into(),
        },
        SeedTodo {
            id: "t5".into(),
            content: "Settings dialog".into(),
            status: "pending".into(),
        },
    ];

    let generic_subagents: Vec<SeedSubagent> = vec![
        SeedSubagent {
            id: "sa-1".into(),
            name: "Spec Auditor".into(),
            description: "Cross-checks implementation against the phase plan".into(),
            status: "running".into(),
        },
        SeedSubagent {
            id: "sa-2".into(),
            name: "Test Synth".into(),
            description: "Generates verification tests for touched modules".into(),
            status: "completed".into(),
        },
    ];

    vec![
        SeedThread {
            id: THREAD_ID_GTD.into(),
            title: "New Thread".into(),
            status: "Busy".into(),
            updated_at: four_hours_ago_str.clone(),
            workspace: Some("/home/user/projects/test".into()),
            messages: vec![SeedMessage {
                id: "m1".into(),
                role: "user".into(),
                content: "Build a todo management system with three modes: GTD (Getting Things Done), Kanban, and Chaos Mode (random prioritization). Research and implement all three.".into(),
                created_at: message_time(0),
            }],
            todos: vec![
                SeedTodo { id: "t1".into(), content: "Research GTD (Getting Things Done) methodology using subagent".into(), status: "in_progress".into() },
                SeedTodo { id: "t2".into(), content: "Research Kanban methodology using subagent".into(), status: "in_progress".into() },
                SeedTodo { id: "t3".into(), content: "Research Chaos Mode (random prioritization) approach using subagent".into(), status: "pending".into() },
                SeedTodo { id: "t4".into(), content: "Design data structure and API endpoints for three todo management systems".into(), status: "pending".into() },
                SeedTodo { id: "t5".into(), content: "Implement GTD backend endpoints and logic in server.js".into(), status: "pending".into() },
                SeedTodo { id: "t6".into(), content: "Implement Kanban backend endpoints and logic in server.js".into(), status: "pending".into() },
                SeedTodo { id: "t7".into(), content: "Implement Chaos Mode backend endpoints and logic in server.js".into(), status: "pending".into() },
                SeedTodo { id: "t8".into(), content: "Create frontend UI for GTD system".into(), status: "pending".into() },
                SeedTodo { id: "t9".into(), content: "Create frontend UI for Kanban system".into(), status: "pending".into() },
                SeedTodo { id: "t10".into(), content: "Create frontend UI for Chaos Mode system".into(), status: "pending".into() },
                SeedTodo { id: "t11".into(), content: "Update README.md with documentation for new systems".into(), status: "pending".into() },
            ],
            subagents: vec![
                SeedSubagent {
                    id: "sa-gtd-1".into(),
                    name: "General Purpose Agent".into(),
                    description: "Research the GTD (Getting Things Done) methodology by David Allen. I need you to provide a comprehensive report covering: 1. Core principles and philosophy of GTD 2. The key components: Inbox, Next Actions, Projects, Waiting For, Someday/Maybe, Contexts 3. The 5 stages of workflow: Capture, Clarify, Organize, Reflect, Engage".into(),
                    status: "running".into(),
                },
                SeedSubagent {
                    id: "sa-gtd-2".into(),
                    name: "General Purpose Agent".into(),
                    description: "Research the Kanban methodology for task management. I need you to provide a comprehensive report covering the core principles, board setup, WIP limits, and flow metrics.".into(),
                    status: "running".into(),
                },
                SeedSubagent {
                    id: "sa-gtd-3".into(),
                    name: "General Purpose Agent".into(),
                    description: "Research and design a Chaos Mode todo management system based on random prioritization and unpredictable task ordering. I need you to provide a creative report...".into(),
                    status: "pending".into(),
                },
            ],
        },
        SeedThread {
            id: THREAD_ID_AUTH.into(),
            title: "New Thread".into(),
            status: "Interrupted".into(),
            updated_at: four_hours_ago_str.clone(),
            workspace: None,
            messages: vec![
                SeedMessage { id: "m1".into(), role: "user".into(), content: "Ship OAuth login with refresh rotation and audit trail.".into(), created_at: message_time(1) },
                SeedMessage { id: "m2".into(), role: "assistant".into(), content: "Copy. I will update auth middleware, add token persistence, and run smoke tests.".into(), created_at: message_time(2) },
            ],
            todos: generic_todos.clone(),
            subagents: generic_subagents.clone(),
        },
        SeedThread {
            id: THREAD_ID_DB.into(),
            title: "New Thread".into(),
            status: "Idle".into(),
            updated_at: four_hours_ago_str.clone(),
            workspace: None,
            messages: vec![
                SeedMessage { id: "m1".into(), role: "user".into(), content: "Need migration plan for v3 schema.".into(), created_at: message_time(3) },
                SeedMessage { id: "m2".into(), role: "assistant".into(), content: "Migration paused pending approval to write production scripts.".into(), created_at: message_time(4) },
            ],
            todos: generic_todos.clone(),
            subagents: generic_subagents.clone(),
        },
        SeedThread {
            id: THREAD_ID_CI.into(),
            title: "New Thread".into(),
            status: "Idle".into(),
            updated_at: four_hours_ago_str.clone(),
            workspace: None,
            messages: vec![],
            todos: generic_todos.clone(),
            subagents: generic_subagents.clone(),
        },
        SeedThread {
            id: THREAD_ID_IDEA.into(),
            title: "New Thread".into(),
            status: "Idle".into(),
            updated_at: four_hours_ago_str,
            workspace: None,
            messages: vec![],
            todos: generic_todos,
            subagents: generic_subagents,
        },
    ]
}

// --- Mock thread files (matches web getMockThreadFiles) ---

#[derive(Serialize)]
pub struct MockFileEntry {
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

pub fn mock_thread_files(thread_id: &str) -> Vec<MockFileEntry> {
    if thread_id == THREAD_ID_GTD {
        return vec![
            MockFileEntry {
                path: "public".into(),
                is_dir: true,
                size: None,
            },
            MockFileEntry {
                path: "public/app.js".into(),
                is_dir: false,
                size: Some(2600),
            },
            MockFileEntry {
                path: "public/index.html".into(),
                is_dir: false,
                size: Some(6900),
            },
            MockFileEntry {
                path: "public/styles.css".into(),
                is_dir: false,
                size: Some(3400),
            },
            MockFileEntry {
                path: "scripts".into(),
                is_dir: true,
                size: None,
            },
            MockFileEntry {
                path: "scripts/flush_todos.js".into(),
                is_dir: false,
                size: Some(381),
            },
            MockFileEntry {
                path: "server".into(),
                is_dir: true,
                size: None,
            },
            MockFileEntry {
                path: "server/server.js".into(),
                is_dir: false,
                size: Some(850),
            },
            MockFileEntry {
                path: "server/todos.json".into(),
                is_dir: false,
                size: Some(314),
            },
        ];
    }
    vec![
        MockFileEntry {
            path: "src".into(),
            is_dir: true,
            size: None,
        },
        MockFileEntry {
            path: "src/main.rs".into(),
            is_dir: false,
            size: Some(9612),
        },
        MockFileEntry {
            path: "src/components/chat/mod.rs".into(),
            is_dir: false,
            size: Some(14020),
        },
        MockFileEntry {
            path: "src/lib/mod.rs".into(),
            is_dir: false,
            size: Some(7903),
        },
        MockFileEntry {
            path: "README.md".into(),
            is_dir: false,
            size: Some(4089),
        },
    ]
}

// --- Mock tool calls (matches web getMockToolCalls) ---

#[derive(Serialize)]
pub struct MockToolCall {
    pub id: String,
    pub name: String,
    pub args: serde_json::Value,
}

pub fn mock_tool_calls(thread_id: &str) -> Vec<MockToolCall> {
    if thread_id != THREAD_ID_GTD {
        return Vec::new();
    }
    vec![
        MockToolCall {
            id: "tc-todos".into(),
            name: "update_todos".into(),
            args: serde_json::json!({
                "todos": [
                    { "content": "Research GTD (Getting Things Done) methodology using subagent", "status": "in_progress" },
                    { "content": "Research Kanban methodology using subagent", "status": "in_progress" },
                    { "content": "Research Chaos Mode (random prioritization) approach using subagent", "status": "pending" },
                    { "content": "Design data structure and API endpoints for three todo management systems", "status": "pending" },
                    { "content": "Implement GTD backend endpoints and logic in server.js", "status": "pending" },
                    { "content": "Implement Kanban backend endpoints and logic in server.js", "status": "pending" },
                    { "content": "Implement Chaos Mode backend endpoints and logic in server.js", "status": "pending" },
                    { "content": "Create frontend UI for GTD system", "status": "pending" },
                    { "content": "Create frontend UI for Kanban system", "status": "pending" },
                    { "content": "Create frontend UI for Chaos Mode system", "status": "pending" },
                    { "content": "Update README.md with documentation for new systems", "status": "pending" },
                ]
            }),
        },
        MockToolCall {
            id: "tc-sa1".into(),
            name: "dispatch_subagent".into(),
            args: serde_json::json!({
                "task": "Research the GTD (Getting Things Done) methodology by David Allen. I need you to provide a comprehensive report covering: 1. Core principles and philosophy of GTD 2. The key components: Inbox, Next Actions, Projects, Waiting For, Someday/Maybe, Contexts 3. The 5 stages of workflow: Capture, Clarify, Organize, Reflect, Engage"
            }),
        },
        MockToolCall {
            id: "tc-sa2".into(),
            name: "dispatch_subagent".into(),
            args: serde_json::json!({
                "task": "Research the Kanban methodology for task management. I need you to provide a comprehensive report covering: 1. Core principles and philosophy of Kanban 2. The..."
            }),
        },
        MockToolCall {
            id: "tc-sa3".into(),
            name: "dispatch_subagent".into(),
            args: serde_json::json!({
                "task": "Research and design a \"Chaos Mode\" todo management system based on random prioritization and unpredictable task ordering. I need you to provide a creative repor..."
            }),
        },
    ]
}

// --- Mock tool results (matches web getMockToolResults) ---

#[derive(Serialize)]
pub struct MockToolResult {
    pub tool_call_id: String,
    pub content: String,
    pub is_error: bool,
}

pub fn mock_tool_results(thread_id: &str) -> Vec<MockToolResult> {
    if thread_id != THREAD_ID_GTD {
        return Vec::new();
    }
    vec![MockToolResult {
        tool_call_id: "tc-todos".into(),
        content: "Synced".into(),
        is_error: false,
    }]
}

// --- Mock workspace files (matches web getMockWorkspaceFiles) ---

pub fn mock_workspace_files() -> BTreeMap<String, Vec<MockFileEntry>> {
    let entries = workspace_seed::workspace_seed_entries();
    let mut grouped: BTreeMap<String, Vec<MockFileEntry>> = BTreeMap::new();

    for entry in &entries {
        let parts: Vec<&str> = entry.path.split('/').filter(|s| !s.is_empty()).collect();
        let root = if parts.len() >= 2 && parts[0] == "home" && parts[1] == "workspace" {
            "/home/workspace".to_string()
        } else if parts.len() >= 4
            && parts[0] == "home"
            && parts[1] == "user"
            && parts[2] == "projects"
        {
            format!("/home/user/projects/{}", parts[3])
        } else {
            continue;
        };

        if entry.path.len() <= root.len() {
            continue;
        }

        let relative = &entry.path[root.len() + 1..];

        if root == "/home/user/projects/test"
            && (relative.starts_with("fixtures/") || relative == "README.md")
        {
            continue;
        }

        let list = grouped.entry(root.clone()).or_default();
        let mut seen: BTreeSet<String> = list.iter().map(|e| e.path.clone()).collect();

        let relative_parts: Vec<&str> = relative.split('/').collect();
        let mut current = root.clone();
        for segment in &relative_parts[..relative_parts.len().saturating_sub(1)] {
            current = format!("{current}/{segment}");
            if !seen.contains(&current) {
                list.push(MockFileEntry {
                    path: current.clone(),
                    is_dir: true,
                    size: None,
                });
                seen.insert(current.clone());
            }
        }

        if !seen.contains(&entry.path) {
            list.push(MockFileEntry {
                path: entry.path.clone(),
                is_dir: false,
                size: Some(entry.size),
            });
            seen.insert(entry.path.clone());
        }
    }

    for list in grouped.values_mut() {
        list.sort_by(|a, b| a.path.cmp(&b.path));
    }

    grouped
}

// --- Scaffold files (matches web scaffoldFilesFromStore) ---

#[derive(Serialize)]
pub struct ScaffoldFile {
    pub path: String,
    pub content: String,
}

pub fn scaffold_files() -> Vec<ScaffoldFile> {
    workspace_seed::workspace_seed_entries()
        .into_iter()
        .map(|entry| {
            let content = if let Some(text) = entry.text {
                text.to_string()
            } else {
                let fallback = entry
                    .fixture
                    .unwrap_or_else(|| entry.path.rsplit('/').next().unwrap_or("sample.txt"));
                format!("seed:{fallback}\n")
            };
            ScaffoldFile {
                path: entry.path,
                content,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_matches_ts() {
        let h = hash_agent_config("https://agent.example.com/api", "sk-mock-1");
        assert_eq!(h.len(), 16);
        assert_eq!(
            h,
            hash_agent_config("https://agent.example.com/api", "sk-mock-1")
        );
    }

    #[test]
    fn seed_agent_endpoints_have_ids() {
        let endpoints = seed_agent_endpoints();
        assert_eq!(endpoints.len(), 2);
        assert_eq!(endpoints[0].name, "Research Agent");
        assert_eq!(endpoints[1].name, "Code Review Agent");
        assert!(endpoints.iter().all(|e| e.removable));
    }

    #[test]
    fn mock_thread_ids_correct() {
        let ids = mock_thread_ids();
        assert_eq!(ids.gtd, "11111111-1111-4111-8111-111111111111");
        assert_eq!(ids.auth, "22222222-2222-4222-8222-222222222222");
    }

    #[test]
    fn seed_threads_returns_five() {
        let threads = seed_threads();
        assert_eq!(threads.len(), 5);
        assert_eq!(threads[0].id, THREAD_ID_GTD);
        assert_eq!(threads[0].status, "Busy");
        assert_eq!(threads[1].status, "Interrupted");
        assert_eq!(threads[0].todos.len(), 11);
        assert_eq!(threads[0].subagents.len(), 3);
    }

    #[test]
    fn mock_thread_files_gtd() {
        let files = mock_thread_files(THREAD_ID_GTD);
        assert_eq!(files.len(), 9);
        assert!(files[0].is_dir);
        assert_eq!(files[0].path, "public");
    }

    #[test]
    fn mock_thread_files_other() {
        let files = mock_thread_files(THREAD_ID_AUTH);
        assert_eq!(files.len(), 5);
        assert_eq!(files[0].path, "src");
    }

    #[test]
    fn mock_tool_calls_gtd() {
        let calls = mock_tool_calls(THREAD_ID_GTD);
        assert_eq!(calls.len(), 4);
        assert_eq!(calls[0].name, "update_todos");
    }

    #[test]
    fn mock_tool_calls_other() {
        assert!(mock_tool_calls(THREAD_ID_AUTH).is_empty());
    }

    #[test]
    fn mock_tool_results_gtd() {
        let results = mock_tool_results(THREAD_ID_GTD);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tool_call_id, "tc-todos");
    }

    #[test]
    fn mock_workspace_files_groups_by_root() {
        let files = mock_workspace_files();
        assert!(!files.is_empty());
        assert!(files.contains_key("/home/workspace"));
    }

    #[test]
    fn scaffold_files_has_entries() {
        let files = scaffold_files();
        assert!(!files.is_empty());
        assert!(files.iter().any(|f| f.path.ends_with("README.md")));
    }
}
