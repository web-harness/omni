use std::collections::HashMap;
use std::pin::Pin;
use std::rc::Rc;

use futures_util::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

pub mod thread_context;
pub mod utils;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadStatus {
    Idle,
    Busy,
    Interrupted,
    Error,
    Done,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    User,
    Assistant,
    Tool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubagentStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderId {
    Anthropic,
    OpenAI,
    Google,
    Ollama,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiThread {
    pub id: String,
    pub title: String,
    pub status: ThreadStatus,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiMessage {
    pub id: String,
    pub role: Role,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub content: String,
    pub status: TodoStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subagent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: SubagentStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
    pub is_error: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HITLRequest {
    pub id: String,
    pub tool_call: ToolCall,
    pub allowed_decisions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provider {
    pub id: ProviderId,
    pub name: String,
    pub has_api_key: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub provider: ProviderId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamEvent {
    Token(String),
    ToolCall(ToolCall),
    ToolResult(ToolResult),
    Todos(Vec<Todo>),
    Done,
    Error(String),
}

pub trait DataProvider {
    fn list_threads(&self) -> Vec<UiThread>;
    fn create_thread(&self) -> UiThread;
    fn delete_thread(&self, id: &str);
    fn get_thread_messages(&self, id: &str) -> Vec<UiMessage>;
    fn stream_response(
        &self,
        thread_id: &str,
        message: &str,
        model_id: &str,
    ) -> Pin<Box<dyn Stream<Item = StreamEvent>>>;
    fn list_models(&self) -> Vec<ModelConfig>;
    fn list_providers(&self) -> Vec<Provider>;
    fn list_workspace_files(&self, thread_id: &str) -> Vec<FileInfo>;
    fn list_subagents(&self, thread_id: &str) -> Vec<Subagent>;
    fn list_todos(&self, thread_id: &str) -> Vec<Todo>;
    fn get_thread_tool_calls(&self, thread_id: &str) -> Vec<ToolCall>;
    fn get_thread_tool_results(&self, thread_id: &str) -> Vec<ToolResult>;
}

#[derive(Default)]
pub struct MockDataProvider;

impl MockDataProvider {
    pub fn new() -> Self {
        Self
    }
}

impl DataProvider for MockDataProvider {
    fn list_threads(&self) -> Vec<UiThread> {
        vec![
            UiThread {
                id: "thread-gtd".to_string(),
                title: "Implement todo management sys...".to_string(),
                status: ThreadStatus::Busy,
                updated_at: "9m ago".to_string(),
            },
            UiThread {
                id: "thread-auth".to_string(),
                title: "Implement Auth Flow".to_string(),
                status: ThreadStatus::Interrupted,
                updated_at: "49m ago".to_string(),
            },
            UiThread {
                id: "thread-db".to_string(),
                title: "Database Migration".to_string(),
                status: ThreadStatus::Idle,
                updated_at: "52m ago".to_string(),
            },
            UiThread {
                id: "thread-ci".to_string(),
                title: "Setup CI Pipeline".to_string(),
                status: ThreadStatus::Idle,
                updated_at: "58m ago".to_string(),
            },
            UiThread {
                id: "thread-idea".to_string(),
                title: "What would be a good...".to_string(),
                status: ThreadStatus::Idle,
                updated_at: "1h ago".to_string(),
            },
        ]
    }

    fn create_thread(&self) -> UiThread {
        UiThread {
            id: format!("thread-{}", Uuid::new_v4().simple()),
            title: "New Thread".to_string(),
            status: ThreadStatus::Idle,
            updated_at: "now".to_string(),
        }
    }

    fn delete_thread(&self, _id: &str) {}

    fn get_thread_messages(&self, id: &str) -> Vec<UiMessage> {
        match id {
            "thread-gtd" => vec![
                UiMessage {
                    id: "m1".to_string(),
                    role: Role::User,
                    content: "Build a todo management system with three modes: GTD (Getting Things Done), Kanban, and Chaos Mode (random prioritization). Research and implement all three.".to_string(),
                },
            ],
            "thread-auth" => vec![
                UiMessage {
                    id: "m1".to_string(),
                    role: Role::User,
                    content: "Ship OAuth login with refresh rotation and audit trail.".to_string(),
                },
                UiMessage {
                    id: "m2".to_string(),
                    role: Role::Assistant,
                    content: "Copy. I will update auth middleware, add token persistence, and run smoke tests."
                        .to_string(),
                },
            ],
            "thread-db" => vec![
                UiMessage {
                    id: "m1".to_string(),
                    role: Role::User,
                    content: "Need migration plan for v3 schema.".to_string(),
                },
                UiMessage {
                    id: "m2".to_string(),
                    role: Role::Assistant,
                    content: "Migration paused pending approval to write production scripts."
                        .to_string(),
                },
            ],
            _ => vec![],
        }
    }

    fn stream_response(
        &self,
        _thread_id: &str,
        _message: &str,
        _model_id: &str,
    ) -> Pin<Box<dyn Stream<Item = StreamEvent>>> {
        let events = vec![
            StreamEvent::Token("Running plan.".to_string()),
            StreamEvent::Token(" Reading workspace.".to_string()),
            StreamEvent::Token(" Updating files.".to_string()),
            StreamEvent::Token(" Verifying behavior.".to_string()),
            StreamEvent::ToolCall(ToolCall {
                id: "tc-1".to_string(),
                name: "read_file".to_string(),
                args: json!({ "path": "src/main.rs" }),
            }),
            StreamEvent::ToolResult(ToolResult {
                tool_call_id: "tc-1".to_string(),
                content: "Read successfully".to_string(),
                is_error: false,
            }),
            StreamEvent::Todos(vec![
                Todo {
                    id: "todo-1".to_string(),
                    content: "Port layout".to_string(),
                    status: TodoStatus::InProgress,
                },
                Todo {
                    id: "todo-2".to_string(),
                    content: "Wire streaming".to_string(),
                    status: TodoStatus::Pending,
                },
            ]),
            StreamEvent::Done,
        ];

        Box::pin(stream::iter(events))
    }

    fn list_models(&self) -> Vec<ModelConfig> {
        vec![
            ModelConfig {
                id: "claude-3-7-sonnet".to_string(),
                name: "Claude 3.7 Sonnet".to_string(),
                provider: ProviderId::Anthropic,
            },
            ModelConfig {
                id: "claude-3-5-haiku".to_string(),
                name: "Claude 3.5 Haiku".to_string(),
                provider: ProviderId::Anthropic,
            },
            ModelConfig {
                id: "gpt-5".to_string(),
                name: "GPT-5".to_string(),
                provider: ProviderId::OpenAI,
            },
            ModelConfig {
                id: "gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
                provider: ProviderId::OpenAI,
            },
            ModelConfig {
                id: "gemini-2.5-pro".to_string(),
                name: "Gemini 2.5 Pro".to_string(),
                provider: ProviderId::Google,
            },
            ModelConfig {
                id: "gemini-2.0-flash".to_string(),
                name: "Gemini 2.0 Flash".to_string(),
                provider: ProviderId::Google,
            },
            ModelConfig {
                id: "llama-3.3-70b".to_string(),
                name: "Llama 3.3 70B".to_string(),
                provider: ProviderId::Ollama,
            },
            ModelConfig {
                id: "deepseek-r1".to_string(),
                name: "DeepSeek R1".to_string(),
                provider: ProviderId::Ollama,
            },
        ]
    }

    fn list_providers(&self) -> Vec<Provider> {
        vec![
            Provider {
                id: ProviderId::Anthropic,
                name: "Anthropic".to_string(),
                has_api_key: true,
            },
            Provider {
                id: ProviderId::OpenAI,
                name: "OpenAI".to_string(),
                has_api_key: false,
            },
            Provider {
                id: ProviderId::Google,
                name: "Google".to_string(),
                has_api_key: false,
            },
            Provider {
                id: ProviderId::Ollama,
                name: "Ollama".to_string(),
                has_api_key: true,
            },
        ]
    }

    fn list_workspace_files(&self, thread_id: &str) -> Vec<FileInfo> {
        match thread_id {
            "thread-gtd" => vec![
                FileInfo {
                    path: "public".to_string(),
                    is_dir: true,
                    size: None,
                },
                FileInfo {
                    path: "public/app.js".to_string(),
                    is_dir: false,
                    size: Some(2_600),
                },
                FileInfo {
                    path: "public/index.html".to_string(),
                    is_dir: false,
                    size: Some(6_900),
                },
                FileInfo {
                    path: "public/styles.css".to_string(),
                    is_dir: false,
                    size: Some(3_400),
                },
                FileInfo {
                    path: "scripts".to_string(),
                    is_dir: true,
                    size: None,
                },
                FileInfo {
                    path: "scripts/flush_todos_node_script.js".to_string(),
                    is_dir: false,
                    size: Some(381),
                },
                FileInfo {
                    path: "server".to_string(),
                    is_dir: true,
                    size: None,
                },
                FileInfo {
                    path: "server/server.js".to_string(),
                    is_dir: false,
                    size: Some(850),
                },
                FileInfo {
                    path: "server/todos.json".to_string(),
                    is_dir: false,
                    size: Some(314),
                },
                FileInfo {
                    path: "test2".to_string(),
                    is_dir: true,
                    size: None,
                },
                FileInfo {
                    path: "test2/hello_french.txt".to_string(),
                    is_dir: false,
                    size: Some(78),
                },
            ],
            _ => vec![
                FileInfo {
                    path: "src".to_string(),
                    is_dir: true,
                    size: None,
                },
                FileInfo {
                    path: "src/main.rs".to_string(),
                    is_dir: false,
                    size: Some(9_612),
                },
                FileInfo {
                    path: "src/components/chat/mod.rs".to_string(),
                    is_dir: false,
                    size: Some(14_020),
                },
                FileInfo {
                    path: "src/lib/mod.rs".to_string(),
                    is_dir: false,
                    size: Some(7_903),
                },
                FileInfo {
                    path: "README.md".to_string(),
                    is_dir: false,
                    size: Some(4_089),
                },
            ],
        }
    }

    fn list_subagents(&self, thread_id: &str) -> Vec<Subagent> {
        match thread_id {
            "thread-gtd" => vec![
                Subagent {
                    id: "sa-gtd-1".to_string(),
                    name: "General Purpose Agent".to_string(),
                    description: "Research the GTD (Getting Things Done) methodology by David Allen. I need you to provide a comprehensive report covering: 1. Core principles and philosophy of GTD 2. The key components: Inbox, Next Actions, Projects, Waiting For, Someday/Maybe, Contexts 3. The 5 stages of workflow: Capture, Clarify, Organize, Reflect, Engage".to_string(),
                    status: SubagentStatus::Running,
                },
                Subagent {
                    id: "sa-gtd-2".to_string(),
                    name: "General Purpose Agent".to_string(),
                    description: "Research the Kanban methodology for task management. I need you to provide a comprehensive report covering the core principles, board setup, WIP limits, and flow metrics.".to_string(),
                    status: SubagentStatus::Running,
                },
                Subagent {
                    id: "sa-gtd-3".to_string(),
                    name: "General Purpose Agent".to_string(),
                    description: "Research and design a Chaos Mode todo management system based on random prioritization and unpredictable task ordering. I need you to provide a creative report...".to_string(),
                    status: SubagentStatus::Pending,
                },
            ],
            _ => vec![
                Subagent {
                id: "sa-1".to_string(),
                name: "Spec Auditor".to_string(),
                description: "Cross-checks implementation against phase plan".to_string(),
                status: SubagentStatus::Running,
            },
            Subagent {
                id: "sa-2".to_string(),
                name: "Test Synth".to_string(),
                description: "Generates verification tests for touched modules".to_string(),
                status: SubagentStatus::Completed,
            },
            ],
        }
    }

    fn list_todos(&self, thread_id: &str) -> Vec<Todo> {
        match thread_id {
            "thread-gtd" => vec![
                Todo {
                    id: "t1".to_string(),
                    content: "Research GTD (Getting Things Done) methodology using subagent"
                        .to_string(),
                    status: TodoStatus::InProgress,
                },
                Todo {
                    id: "t2".to_string(),
                    content: "Research Kanban methodology using subagent".to_string(),
                    status: TodoStatus::InProgress,
                },
                Todo {
                    id: "t3".to_string(),
                    content: "Research Chaos Mode (random prioritization) approach using subagent"
                        .to_string(),
                    status: TodoStatus::Pending,
                },
                Todo {
                    id: "t4".to_string(),
                    content:
                        "Design data structure and API endpoints for three todo management systems"
                            .to_string(),
                    status: TodoStatus::Pending,
                },
                Todo {
                    id: "t5".to_string(),
                    content: "Implement GTD backend endpoints and logic in server.js".to_string(),
                    status: TodoStatus::Pending,
                },
                Todo {
                    id: "t6".to_string(),
                    content: "Implement Kanban backend endpoints and logic in server.js"
                        .to_string(),
                    status: TodoStatus::Pending,
                },
                Todo {
                    id: "t7".to_string(),
                    content: "Implement Chaos Mode backend endpoints and logic in server.js"
                        .to_string(),
                    status: TodoStatus::Pending,
                },
                Todo {
                    id: "t8".to_string(),
                    content: "Create frontend UI for GTD system".to_string(),
                    status: TodoStatus::Pending,
                },
                Todo {
                    id: "t9".to_string(),
                    content: "Create frontend UI for Kanban system".to_string(),
                    status: TodoStatus::Pending,
                },
                Todo {
                    id: "t10".to_string(),
                    content: "Create frontend UI for Chaos Mode system".to_string(),
                    status: TodoStatus::Pending,
                },
                Todo {
                    id: "t11".to_string(),
                    content: "Update README.md with documentation for new systems".to_string(),
                    status: TodoStatus::Pending,
                },
            ],
            _ => vec![
                Todo {
                    id: "t1".to_string(),
                    content: "Wire dock slots".to_string(),
                    status: TodoStatus::Completed,
                },
                Todo {
                    id: "t2".to_string(),
                    content: "Port sidebar interactions".to_string(),
                    status: TodoStatus::InProgress,
                },
                Todo {
                    id: "t3".to_string(),
                    content: "Implement model switcher".to_string(),
                    status: TodoStatus::Pending,
                },
                Todo {
                    id: "t4".to_string(),
                    content: "File viewer tabs".to_string(),
                    status: TodoStatus::Pending,
                },
                Todo {
                    id: "t5".to_string(),
                    content: "Settings dialog".to_string(),
                    status: TodoStatus::Pending,
                },
            ],
        }
    }

    fn get_thread_tool_calls(&self, thread_id: &str) -> Vec<ToolCall> {
        if thread_id != "thread-gtd" {
            return vec![];
        }
        vec![
            ToolCall {
                id: "tc-todos".to_string(),
                name: "update_todos".to_string(),
                args: json!({
                    "todos": [
                        {"content": "Research GTD (Getting Things Done) methodology using subagent", "status": "in_progress"},
                        {"content": "Research Kanban methodology using subagent", "status": "in_progress"},
                        {"content": "Research Chaos Mode (random prioritization) approach using subagent", "status": "pending"},
                        {"content": "Design data structure and API endpoints for three todo management systems", "status": "pending"},
                        {"content": "Implement GTD backend endpoints and logic in server.js", "status": "pending"},
                        {"content": "Implement Kanban backend endpoints and logic in server.js", "status": "pending"},
                        {"content": "Implement Chaos Mode backend endpoints and logic in server.js", "status": "pending"},
                        {"content": "Create frontend UI for GTD system", "status": "pending"},
                        {"content": "Create frontend UI for Kanban system", "status": "pending"},
                        {"content": "Create frontend UI for Chaos Mode system", "status": "pending"},
                        {"content": "Update README.md with documentation for new systems", "status": "pending"}
                    ]
                }),
            },
            ToolCall {
                id: "tc-sa1".to_string(),
                name: "dispatch_subagent".to_string(),
                args: json!({ "task": "Research the GTD (Getting Things Done) methodology by David Allen. I need you to provide a comprehensive report covering: 1. Core principles and philosophy of GTD 2. The key components: Inbox, Next Actions, Projects, Waiting For, Someday/Maybe, Contexts 3. The 5 stages of workflow: Capture, Clarify, Organize, Reflect, Engage" }),
            },
            ToolCall {
                id: "tc-sa2".to_string(),
                name: "dispatch_subagent".to_string(),
                args: json!({ "task": "Research the Kanban methodology for task management. I need you to provide a comprehensive report covering: 1. Core principles and philosophy of Kanban 2. The..." }),
            },
            ToolCall {
                id: "tc-sa3".to_string(),
                name: "dispatch_subagent".to_string(),
                args: json!({ "task": "Research and design a \"Chaos Mode\" todo management system based on random prioritization and unpredictable task ordering. I need you to provide a creative repor..." }),
            },
        ]
    }

    fn get_thread_tool_results(&self, thread_id: &str) -> Vec<ToolResult> {
        if thread_id != "thread-gtd" {
            return vec![];
        }
        vec![ToolResult {
            tool_call_id: "tc-todos".to_string(),
            content: "Synced".to_string(),
            is_error: false,
        }]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Clone, PartialEq)]
pub struct ThreadState {
    pub threads: Vec<UiThread>,
    pub active_thread_id: Option<String>,
    pub show_kanban: bool,
}

impl ThreadState {
    pub fn current_thread_id(&self) -> Option<&str> {
        self.active_thread_id.as_deref()
    }
}

#[derive(Clone, PartialEq)]
pub struct ChatState {
    pub messages: HashMap<String, Vec<UiMessage>>,
    pub input_draft: String,
    pub is_streaming: bool,
    pub stream_buffer: String,
    pub error: Option<String>,
}

impl ChatState {
    pub fn messages_for(&self, thread_id: &str) -> Vec<UiMessage> {
        self.messages.get(thread_id).cloned().unwrap_or_default()
    }
}

#[derive(Clone, PartialEq)]
pub struct TasksState {
    pub todos: HashMap<String, Vec<Todo>>,
    pub files: HashMap<String, Vec<FileInfo>>,
    pub tool_calls: HashMap<String, Vec<ToolCall>>,
    pub tool_results: HashMap<String, Vec<ToolResult>>,
}

impl TasksState {
    pub fn todos_for(&self, thread_id: &str) -> Vec<Todo> {
        self.todos.get(thread_id).cloned().unwrap_or_default()
    }

    pub fn files_for(&self, thread_id: &str) -> Vec<FileInfo> {
        self.files.get(thread_id).cloned().unwrap_or_default()
    }

    pub fn tool_calls_for(&self, thread_id: &str) -> Vec<ToolCall> {
        self.tool_calls.get(thread_id).cloned().unwrap_or_default()
    }

    pub fn tool_results_for(&self, thread_id: &str) -> Vec<ToolResult> {
        self.tool_results
            .get(thread_id)
            .cloned()
            .unwrap_or_default()
    }
}

#[derive(Clone, PartialEq)]
pub struct WorkspaceState {
    pub workspace_path: HashMap<String, String>,
    pub workspace_files: HashMap<String, Vec<FileInfo>>,
    pub open_tabs: HashMap<String, Vec<String>>,
    pub active_tab: HashMap<String, String>,
}

impl WorkspaceState {
    pub fn workspace_for(&self, thread_id: &str) -> String {
        self.workspace_path
            .get(thread_id)
            .cloned()
            .unwrap_or_else(|| "test".to_string())
    }

    pub fn files_for_thread(&self, thread_id: &str) -> Vec<FileInfo> {
        let workspace = self.workspace_for(thread_id);
        self.workspace_files
            .get(&workspace)
            .cloned()
            .unwrap_or_default()
    }

    pub fn open_tabs_for(&self, thread_id: &str) -> Vec<String> {
        self.open_tabs.get(thread_id).cloned().unwrap_or_default()
    }

    pub fn active_tab_for(&self, thread_id: &str) -> String {
        self.active_tab
            .get(thread_id)
            .cloned()
            .unwrap_or_else(|| "chat".to_string())
    }
}

#[derive(Clone, PartialEq)]
pub struct ModelState {
    pub providers: Vec<Provider>,
    pub models: Vec<ModelConfig>,
    pub selected_model: HashMap<String, String>,
}

impl ModelState {
    pub fn selected_model_for(&self, thread_id: &str) -> String {
        self.selected_model
            .get(thread_id)
            .cloned()
            .or_else(|| self.models.first().map(|m| m.id.clone()))
            .unwrap_or_else(|| "gpt-5".to_string())
    }
}

#[derive(Clone, PartialEq)]
pub struct UiState {
    pub theme: Theme,
    pub settings_open: bool,
    pub api_key_dialog_open: bool,
    pub api_key_provider: ProviderId,
    pub api_key_draft: String,
}

#[derive(Clone, PartialEq)]
pub struct SubagentState {
    pub subagents: HashMap<String, Vec<Subagent>>,
    pub pending_hitl: Option<HITLRequest>,
}

impl SubagentState {
    pub fn subagents_for(&self, thread_id: &str) -> Vec<Subagent> {
        self.subagents.get(thread_id).cloned().unwrap_or_default()
    }
}

pub fn bootstrap(
    provider: Rc<dyn DataProvider>,
) -> (
    ThreadState,
    ChatState,
    TasksState,
    WorkspaceState,
    ModelState,
    UiState,
    SubagentState,
) {
    let threads = provider.list_threads();
    let active_thread_id = threads.first().map(|t| t.id.clone());

    let mut messages = HashMap::new();
    let mut todos = HashMap::new();
    let mut files = HashMap::new();
    let mut subagents = HashMap::new();
    let mut tool_calls = HashMap::new();
    let mut tool_results = HashMap::new();

    for thread in &threads {
        messages.insert(thread.id.clone(), provider.get_thread_messages(&thread.id));
        todos.insert(thread.id.clone(), provider.list_todos(&thread.id));
        files.insert(thread.id.clone(), provider.list_workspace_files(&thread.id));
        subagents.insert(thread.id.clone(), provider.list_subagents(&thread.id));
        tool_calls.insert(
            thread.id.clone(),
            provider.get_thread_tool_calls(&thread.id),
        );
        tool_results.insert(
            thread.id.clone(),
            provider.get_thread_tool_results(&thread.id),
        );
    }

    let models = provider.list_models();
    let first_model = models
        .first()
        .map(|m| m.id.clone())
        .unwrap_or_else(|| "gpt-5".to_string());

    let mut selected_model: HashMap<String, String> = HashMap::new();
    let mut workspace_path: HashMap<String, String> = HashMap::new();
    let mut open_tabs: HashMap<String, Vec<String>> = HashMap::new();
    let mut active_tab: HashMap<String, String> = HashMap::new();

    for (i, thread) in threads.iter().enumerate() {
        selected_model.insert(thread.id.clone(), first_model.clone());
        let ws = match i {
            1 => "omni",
            2 => "omni-rt",
            _ => "test",
        };
        workspace_path.insert(thread.id.clone(), ws.to_string());
        open_tabs.insert(thread.id.clone(), vec![]);
        active_tab.insert(thread.id.clone(), "chat".to_string());
    }

    #[cfg(target_arch = "wasm32")]
    let initial_theme = {
        let search = web_sys::window()
            .and_then(|w| w.location().search().ok())
            .unwrap_or_default();
        if search.contains("theme=light") {
            Theme::Light
        } else {
            Theme::Dark
        }
    };
    #[cfg(not(target_arch = "wasm32"))]
    let initial_theme = Theme::Dark;

    let workspace_files = {
        let mut wf = HashMap::new();
        wf.insert(
            "test".to_string(),
            vec![
                FileInfo {
                    path: "public".to_string(),
                    is_dir: true,
                    size: None,
                },
                FileInfo {
                    path: "public/app.js".to_string(),
                    is_dir: false,
                    size: Some(2_600),
                },
                FileInfo {
                    path: "public/index.html".to_string(),
                    is_dir: false,
                    size: Some(6_900),
                },
                FileInfo {
                    path: "public/styles.css".to_string(),
                    is_dir: false,
                    size: Some(3_400),
                },
                FileInfo {
                    path: "scripts".to_string(),
                    is_dir: true,
                    size: None,
                },
                FileInfo {
                    path: "scripts/flush_todos.js".to_string(),
                    is_dir: false,
                    size: Some(381),
                },
                FileInfo {
                    path: "server".to_string(),
                    is_dir: true,
                    size: None,
                },
                FileInfo {
                    path: "server/server.js".to_string(),
                    is_dir: false,
                    size: Some(850),
                },
                FileInfo {
                    path: "server/todos.json".to_string(),
                    is_dir: false,
                    size: Some(314),
                },
            ],
        );
        wf.insert(
            "omni".to_string(),
            vec![
                FileInfo {
                    path: "src".to_string(),
                    is_dir: true,
                    size: None,
                },
                FileInfo {
                    path: "src/main.rs".to_string(),
                    is_dir: false,
                    size: Some(9_612),
                },
                FileInfo {
                    path: "src/components".to_string(),
                    is_dir: true,
                    size: None,
                },
                FileInfo {
                    path: "src/components/chat/mod.rs".to_string(),
                    is_dir: false,
                    size: Some(14_020),
                },
                FileInfo {
                    path: "src/components/sidebar/mod.rs".to_string(),
                    is_dir: false,
                    size: Some(3_400),
                },
                FileInfo {
                    path: "src/lib".to_string(),
                    is_dir: true,
                    size: None,
                },
                FileInfo {
                    path: "src/lib/mod.rs".to_string(),
                    is_dir: false,
                    size: Some(7_903),
                },
                FileInfo {
                    path: "Cargo.toml".to_string(),
                    is_dir: false,
                    size: Some(1_200),
                },
                FileInfo {
                    path: "README.md".to_string(),
                    is_dir: false,
                    size: Some(4_089),
                },
            ],
        );
        wf.insert(
            "omni-rt".to_string(),
            vec![
                FileInfo {
                    path: "crates".to_string(),
                    is_dir: true,
                    size: None,
                },
                FileInfo {
                    path: "crates/omni-protocol".to_string(),
                    is_dir: true,
                    size: None,
                },
                FileInfo {
                    path: "crates/omni-protocol/src/lib.rs".to_string(),
                    is_dir: false,
                    size: Some(5_120),
                },
                FileInfo {
                    path: "crates/omni-rt".to_string(),
                    is_dir: true,
                    size: None,
                },
                FileInfo {
                    path: "crates/omni-rt/src/main.rs".to_string(),
                    is_dir: false,
                    size: Some(3_800),
                },
                FileInfo {
                    path: "crates/omni-dock".to_string(),
                    is_dir: true,
                    size: None,
                },
                FileInfo {
                    path: "crates/omni-dock/src/omni-dock.ts".to_string(),
                    is_dir: false,
                    size: Some(8_200),
                },
                FileInfo {
                    path: "Cargo.toml".to_string(),
                    is_dir: false,
                    size: Some(980),
                },
            ],
        );
        wf
    };

    (
        ThreadState {
            threads,
            active_thread_id,
            show_kanban: false,
        },
        ChatState {
            messages,
            input_draft: String::new(),
            is_streaming: false,
            stream_buffer: String::new(),
            error: None,
        },
        TasksState {
            todos,
            files,
            tool_calls,
            tool_results,
        },
        WorkspaceState {
            workspace_path,
            workspace_files,
            open_tabs,
            active_tab,
        },
        ModelState {
            providers: provider.list_providers(),
            models,
            selected_model,
        },
        UiState {
            theme: initial_theme,
            settings_open: false,
            api_key_dialog_open: false,
            api_key_provider: ProviderId::Anthropic,
            api_key_draft: String::new(),
        },
        SubagentState {
            subagents,
            pending_hitl: None,
        },
    )
}
