use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use omni_rt::deepagents::model_registry::BROWSER_MODEL_SPECS;
use serde::{Deserialize, Serialize};

pub mod file_types;
pub mod fixtures;
pub mod sw_api;
#[cfg(target_arch = "wasm32")]
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
pub enum BackgroundTaskStatus {
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
    Browser,
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
pub struct BackgroundTask {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: BackgroundTaskStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEndpoint {
    pub id: String,
    pub url: String,
    pub bearer_token: String,
    pub name: String,
    pub removable: bool,
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserDownloadPhase {
    #[default]
    Idle,
    Downloading,
    Completed,
    Error,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserDownloadStatus {
    pub phase: BrowserDownloadPhase,
    pub model_id: Option<String>,
    pub loaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
    pub progress_percent: Option<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserInferenceStatus {
    pub engaged: bool,
    pub loaded_model_id: Option<String>,
    pub cached_model_ids: Vec<String>,
    pub download: BrowserDownloadStatus,
    pub last_error: Option<String>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamEvent {
    Token(String),
    ToolCall(ToolCall),
    ToolResult(ToolResult),
    Todos(Vec<Todo>),
    Done,
    Error(String),
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
    pub tab_generation: HashMap<String, u64>,
}

impl WorkspaceState {
    pub fn workspace_for(&self, thread_id: &str) -> String {
        self.workspace_path
            .get(thread_id)
            .cloned()
            .unwrap_or_else(|| "/home/workspace".to_string())
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
    pub browser_inference: BrowserInferenceStatus,
}

impl ModelState {
    pub fn selected_model_for(&self, thread_id: &str) -> String {
        self.selected_model
            .get(thread_id)
            .cloned()
            .or_else(|| self.models.first().map(|m| m.id.clone()))
            .unwrap_or_else(|| "claude-3-7-sonnet".to_string())
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
pub struct BackgroundTaskState {
    pub background_tasks: HashMap<String, Vec<BackgroundTask>>,
    pub pending_hitl: Option<HITLRequest>,
}

impl BackgroundTaskState {
    pub fn tasks_for(&self, thread_id: &str) -> Vec<BackgroundTask> {
        self.background_tasks
            .get(thread_id)
            .cloned()
            .unwrap_or_default()
    }
}

#[derive(Clone, PartialEq)]
pub struct AgentEndpointState {
    pub endpoints: Vec<AgentEndpoint>,
    pub active_agent_id: Option<String>,
    pub dicebear_style: String,
}

impl AgentEndpointState {
    pub fn ordered(&self) -> Vec<&AgentEndpoint> {
        let mut endpoints: Vec<_> = self
            .endpoints
            .iter()
            .filter(|endpoint| !endpoint.removable)
            .collect();
        endpoints.extend(self.endpoints.iter().filter(|endpoint| endpoint.removable));
        endpoints
    }

    pub fn active_endpoint(&self) -> Option<&AgentEndpoint> {
        match &self.active_agent_id {
            Some(id) => self.endpoints.iter().find(|endpoint| endpoint.id == *id),
            None => self.endpoints.iter().find(|endpoint| !endpoint.removable),
        }
    }

    pub fn upsert(&mut self, endpoint: AgentEndpoint) {
        if let Some(existing) = self
            .endpoints
            .iter_mut()
            .find(|item| item.id == endpoint.id)
        {
            *existing = endpoint;
            return;
        }
        self.endpoints.push(endpoint);
    }

    pub fn remove(&mut self, id: &str) {
        self.endpoints
            .retain(|endpoint| endpoint.id != id || !endpoint.removable);
        if self.active_agent_id.as_deref() == Some(id) {
            self.active_agent_id = None;
        }
    }
}

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

fn stable_hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

pub fn agent_config_hash(url: &str, bearer_token: &str) -> String {
    let mut bytes = Vec::with_capacity(url.len() + bearer_token.len() + 1);
    bytes.extend_from_slice(url.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(bearer_token.as_bytes());
    format!("{:016x}", stable_hash_bytes(&bytes))
}

pub fn derive_agent_name(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return "Agent".into();
    }

    let without_scheme = trimmed
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(trimmed);
    let host = without_scheme.split('/').next().unwrap_or(trimmed).trim();
    if host.is_empty() {
        "Agent".into()
    } else {
        host.into()
    }
}

pub fn builtin_main_agent() -> AgentEndpoint {
    AgentEndpoint {
        id: "main".into(),
        url: String::new(),
        bearer_token: String::new(),
        name: "Main Agent".into(),
        removable: false,
    }
}

pub fn merge_agent_endpoints(endpoints: Vec<AgentEndpoint>) -> Vec<AgentEndpoint> {
    let mut merged = vec![builtin_main_agent()];
    for endpoint in endpoints {
        if endpoint.id == "main" || !endpoint.removable {
            continue;
        }
        if merged.iter().any(|item| item.id == endpoint.id) {
            continue;
        }
        merged.push(endpoint);
    }
    merged
}

pub fn normalize_dicebear_style(style: &str) -> String {
    match style.trim() {
        "thumbs" => "thumbs".into(),
        _ => "bottts-neutral".into(),
    }
}

pub fn static_models() -> Vec<ModelConfig> {
    #[cfg(target_arch = "wasm32")]
    {
        return vec![];
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut models = vec![
            ModelConfig {
                id: "claude-3-7-sonnet".into(),
                name: "Claude 3.7 Sonnet".into(),
                provider: ProviderId::Anthropic,
            },
            ModelConfig {
                id: "claude-3-5-haiku".into(),
                name: "Claude 3.5 Haiku".into(),
                provider: ProviderId::Anthropic,
            },
            ModelConfig {
                id: "gpt-5".into(),
                name: "GPT-5".into(),
                provider: ProviderId::OpenAI,
            },
            ModelConfig {
                id: "gpt-4o".into(),
                name: "GPT-4o".into(),
                provider: ProviderId::OpenAI,
            },
            ModelConfig {
                id: "gemini-2.5-pro".into(),
                name: "Gemini 2.5 Pro".into(),
                provider: ProviderId::Google,
            },
            ModelConfig {
                id: "gemini-2.0-flash".into(),
                name: "Gemini 2.0 Flash".into(),
                provider: ProviderId::Google,
            },
            ModelConfig {
                id: "llama-3.3-70b".into(),
                name: "Llama 3.3 70B".into(),
                provider: ProviderId::Ollama,
            },
            ModelConfig {
                id: "deepseek-r1".into(),
                name: "DeepSeek R1".into(),
                provider: ProviderId::Ollama,
            },
        ];

        models.extend(BROWSER_MODEL_SPECS.into_iter().map(|model| ModelConfig {
            id: model.id.into(),
            name: model.name.into(),
            provider: ProviderId::Browser,
        }));

        models
    }
}

pub fn static_providers() -> Vec<Provider> {
    #[cfg(target_arch = "wasm32")]
    {
        return vec![];
    }

    #[cfg(not(target_arch = "wasm32"))]
    vec![
        Provider {
            id: ProviderId::Anthropic,
            name: "Anthropic".into(),
            has_api_key: false,
        },
        Provider {
            id: ProviderId::OpenAI,
            name: "OpenAI".into(),
            has_api_key: false,
        },
        Provider {
            id: ProviderId::Google,
            name: "Google".into(),
            has_api_key: false,
        },
        Provider {
            id: ProviderId::Ollama,
            name: "Ollama".into(),
            has_api_key: false,
        },
        Provider {
            id: ProviderId::Browser,
            name: "Browser".into(),
            has_api_key: true,
        },
    ]
}

pub fn default_states() -> (
    ThreadState,
    ChatState,
    TasksState,
    WorkspaceState,
    ModelState,
    UiState,
    BackgroundTaskState,
    AgentEndpointState,
) {
    let initial_theme = Theme::Dark;

    (
        ThreadState {
            threads: vec![],
            active_thread_id: None,
            show_kanban: false,
        },
        ChatState {
            messages: HashMap::new(),
            input_draft: String::new(),
            is_streaming: false,
            stream_buffer: String::new(),
            error: None,
        },
        TasksState {
            todos: HashMap::new(),
            files: HashMap::new(),
            tool_calls: HashMap::new(),
            tool_results: HashMap::new(),
        },
        WorkspaceState {
            workspace_path: HashMap::new(),
            workspace_files: HashMap::new(),
            open_tabs: HashMap::new(),
            active_tab: HashMap::new(),
            tab_generation: HashMap::new(),
        },
        ModelState {
            providers: static_providers(),
            models: static_models(),
            selected_model: HashMap::new(),
            browser_inference: BrowserInferenceStatus::default(),
        },
        UiState {
            theme: initial_theme,
            settings_open: false,
            api_key_dialog_open: false,
            api_key_provider: ProviderId::Anthropic,
            api_key_draft: String::new(),
        },
        BackgroundTaskState {
            background_tasks: HashMap::new(),
            pending_hitl: None,
        },
        AgentEndpointState {
            endpoints: vec![builtin_main_agent()],
            active_agent_id: None,
            dicebear_style: normalize_dicebear_style("bottts-neutral"),
        },
    )
}

#[cfg(target_arch = "wasm32")]
pub async fn async_init(
    mut thread_state: dioxus::prelude::Signal<ThreadState>,
    mut chat_state: dioxus::prelude::Signal<ChatState>,
    mut tasks_state: dioxus::prelude::Signal<TasksState>,
    mut workspace_state: dioxus::prelude::Signal<WorkspaceState>,
    mut model_state: dioxus::prelude::Signal<ModelState>,
    mut background_task_state: dioxus::prelude::Signal<BackgroundTaskState>,
    mut agent_endpoint_state: dioxus::prelude::Signal<AgentEndpointState>,
    iframe_agent_endpoints: Option<Vec<AgentEndpoint>>,
    iframe_dicebear_style: Option<String>,
) {
    use dioxus::signals::{ReadableExt, WritableExt};
    use gloo_timers::future::TimeoutFuture;
    let mut payload_opt = None;
    for _ in 0..20 {
        if let Ok(p) = sw_api::fetch_bootstrap().await {
            payload_opt = Some(p);
            break;
        }
        TimeoutFuture::new(200).await;
    }
    let Some(payload) = payload_opt else {
        return;
    };

    {
        let mut ms = model_state.write();
        if !payload.models.is_empty() {
            ms.models = payload.models.clone();
        }
        if !payload.providers.is_empty() {
            ms.providers = payload.providers.clone();
        }
    }

    if let Ok(status) = sw_api::get_browser_inference_status().await {
        model_state.write().browser_inference = status;
    }

    let first_model = model_state.read().models.first().map(|m| m.id.clone());
    let mut selected_model = HashMap::new();
    let threads: Vec<UiThread> = payload
        .threads
        .clone()
        .into_iter()
        .map(|t| {
            let id = t.id.clone();
            selected_model.insert(
                id.clone(),
                if model_state
                    .read()
                    .models
                    .iter()
                    .any(|m| m.id == payload.default_model)
                {
                    payload.default_model.clone()
                } else {
                    first_model
                        .clone()
                        .unwrap_or_else(|| "claude-3-7-sonnet".to_string())
                },
            );
            t
        })
        .collect();

    let active_id = threads.first().map(|t| t.id.clone());
    {
        let mut ts = thread_state.write();
        ts.threads = threads;
        ts.active_thread_id = active_id;
    }
    {
        let mut cs = chat_state.write();
        cs.messages = payload.messages;
    }
    {
        let mut tsk = tasks_state.write();
        tsk.todos = payload.todos;
        tsk.files = payload.files;
        tsk.tool_calls = payload.tool_calls;
        tsk.tool_results = payload.tool_results;
    }
    {
        let mut task_state = background_task_state.write();
        task_state.background_tasks = payload.background_tasks;
    }
    {
        let mut endpoints = agent_endpoint_state.write();
        endpoints.endpoints = iframe_agent_endpoints
            .map(merge_agent_endpoints)
            .unwrap_or_else(|| merge_agent_endpoints(payload.agent_endpoints));
        endpoints.dicebear_style = iframe_dicebear_style
            .unwrap_or_else(|| normalize_dicebear_style(&payload.dicebear_style));
    }
    model_state.write().selected_model = selected_model;

    {
        let mut ws = workspace_state.write();
        ws.workspace_path = payload.workspace_path;
        ws.workspace_files = payload.workspace_files;
    }

    let active_id = thread_state.read().active_thread_id.clone();
    if let Some(active) = active_id {
        let workspace = workspace_state.read().workspace_for(&active);
        if workspace_state
            .read()
            .workspace_files
            .get(&workspace)
            .is_none()
        {
            if let Ok(files) = sw_api::list_workspace_files(&workspace).await {
                let mut ws = workspace_state.write();
                ws.workspace_files.insert(workspace, files);
            }
        }
    }
}
