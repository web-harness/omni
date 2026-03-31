use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub mod file_types;
pub mod fixtures;
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
pub struct SubagentState {
    pub subagents: HashMap<String, Vec<Subagent>>,
    pub pending_hitl: Option<HITLRequest>,
}

impl SubagentState {
    pub fn subagents_for(&self, thread_id: &str) -> Vec<Subagent> {
        self.subagents.get(thread_id).cloned().unwrap_or_default()
    }
}

pub fn static_models() -> Vec<ModelConfig> {
    #[cfg(target_arch = "wasm32")]
    {
        return omni_rt::deepagents::model_registry::list_models()
            .into_iter()
            .map(|m| ModelConfig {
                id: m.id,
                name: m.name,
                provider: match m.provider {
                    omni_rt::deepagents::model_registry::ProviderId::Anthropic => {
                        ProviderId::Anthropic
                    }
                    omni_rt::deepagents::model_registry::ProviderId::OpenAI => ProviderId::OpenAI,
                    omni_rt::deepagents::model_registry::ProviderId::Google => ProviderId::Google,
                    omni_rt::deepagents::model_registry::ProviderId::Ollama => ProviderId::Ollama,
                },
            })
            .collect();
    }

    #[cfg(not(target_arch = "wasm32"))]
    vec![
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
    ]
}

pub fn static_providers() -> Vec<Provider> {
    #[cfg(target_arch = "wasm32")]
    {
        return omni_rt::deepagents::model_registry::list_providers()
            .into_iter()
            .map(|p| Provider {
                id: match p.id {
                    omni_rt::deepagents::model_registry::ProviderId::Anthropic => {
                        ProviderId::Anthropic
                    }
                    omni_rt::deepagents::model_registry::ProviderId::OpenAI => ProviderId::OpenAI,
                    omni_rt::deepagents::model_registry::ProviderId::Google => ProviderId::Google,
                    omni_rt::deepagents::model_registry::ProviderId::Ollama => ProviderId::Ollama,
                },
                name: p.name,
                has_api_key: false,
            })
            .collect();
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
    ]
}

pub fn default_states() -> (
    ThreadState,
    ChatState,
    TasksState,
    WorkspaceState,
    ModelState,
    UiState,
    SubagentState,
) {
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
        },
        UiState {
            theme: initial_theme,
            settings_open: false,
            api_key_dialog_open: false,
            api_key_provider: ProviderId::Anthropic,
            api_key_draft: String::new(),
        },
        SubagentState {
            subagents: HashMap::new(),
            pending_hitl: None,
        },
    )
}

#[cfg(target_arch = "wasm32")]
pub async fn async_init(
    mut thread_state: dioxus::prelude::Signal<ThreadState>,
    mut chat_state: dioxus::prelude::Signal<ChatState>,
    mut tasks_state: dioxus::prelude::Signal<TasksState>,
    mut model_state: dioxus::prelude::Signal<ModelState>,
    mut subagent_state: dioxus::prelude::Signal<SubagentState>,
) {
    use dioxus::signals::{ReadableExt, WritableExt};
    use omni_rt::deepagents::{
        config_store, message_store, model_registry, seed, subagent_store, thread_store, todo_store,
    };
    use omni_rt::zenfs;

    fn map_provider_id(id: model_registry::ProviderId) -> ProviderId {
        match id {
            model_registry::ProviderId::Anthropic => ProviderId::Anthropic,
            model_registry::ProviderId::OpenAI => ProviderId::OpenAI,
            model_registry::ProviderId::Google => ProviderId::Google,
            model_registry::ProviderId::Ollama => ProviderId::Ollama,
        }
    }

    if zenfs::init().await.is_err() {
        return;
    }

    {
        let models = model_registry::list_models()
            .into_iter()
            .map(|m| ModelConfig {
                id: m.id,
                name: m.name,
                provider: map_provider_id(m.provider),
            })
            .collect::<Vec<_>>();

        let providers = model_registry::list_providers_with_keys()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|(p, has_api_key)| Provider {
                id: map_provider_id(p.id),
                name: p.name,
                has_api_key,
            })
            .collect::<Vec<_>>();

        let mut ms = model_state.write();
        if !models.is_empty() {
            ms.models = models;
        }
        if !providers.is_empty() {
            ms.providers = providers;
        }
    }

    let _ = seed::seed_if_empty().await;

    let stored = match thread_store::list_threads().await {
        Ok(t) => {
            if t.is_empty() {
                return;
            }
            t
        }
        Err(_) => return,
    };

    let default_model = config_store::get_default_model()
        .await
        .unwrap_or_else(|_| "claude-3-7-sonnet".to_string());
    let first_model = model_state.read().models.first().map(|m| m.id.clone());
    let mut selected_model = HashMap::new();
    let mut messages: HashMap<String, Vec<UiMessage>> = HashMap::new();
    let mut todos: HashMap<String, Vec<Todo>> = HashMap::new();
    let mut subagents: HashMap<String, Vec<Subagent>> = HashMap::new();

    let threads: Vec<UiThread> = stored
        .into_iter()
        .map(|t| {
            let id = t.thread_id.simple().to_string();
            selected_model.insert(
                id.clone(),
                if model_state
                    .read()
                    .models
                    .iter()
                    .any(|m| m.id == default_model)
                {
                    default_model.clone()
                } else {
                    first_model
                        .clone()
                        .unwrap_or_else(|| "claude-3-7-sonnet".to_string())
                },
            );
            let title = t
                .metadata
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("New Thread")
                .to_string();
            UiThread {
                id,
                title,
                status: match t.status {
                    omni_rt::protocol::ThreadStatus::Busy => ThreadStatus::Busy,
                    omni_rt::protocol::ThreadStatus::Interrupted => ThreadStatus::Interrupted,
                    omni_rt::protocol::ThreadStatus::Error => ThreadStatus::Error,
                    omni_rt::protocol::ThreadStatus::Idle => ThreadStatus::Idle,
                },
                updated_at: t.updated_at.to_rfc3339(),
            }
        })
        .collect();

    for t in &threads {
        if let Ok(msgs) = message_store::list_messages(&t.id).await {
            messages.insert(
                t.id.clone(),
                msgs.into_iter()
                    .map(|m| UiMessage {
                        id: m.id,
                        role: match m.role {
                            message_store::Role::User => Role::User,
                            message_store::Role::Assistant => Role::Assistant,
                            message_store::Role::Tool => Role::Tool,
                        },
                        content: m.content,
                    })
                    .collect(),
            );
        }
        if let Ok(tdos) = todo_store::list_todos(&t.id).await {
            todos.insert(
                t.id.clone(),
                tdos.into_iter()
                    .map(|td| Todo {
                        id: td.id,
                        content: td.content,
                        status: match td.status {
                            todo_store::TodoStatus::Pending => TodoStatus::Pending,
                            todo_store::TodoStatus::InProgress => TodoStatus::InProgress,
                            todo_store::TodoStatus::Completed => TodoStatus::Completed,
                            todo_store::TodoStatus::Cancelled => TodoStatus::Cancelled,
                        },
                    })
                    .collect(),
            );
        }
        if let Ok(sas) = subagent_store::list_subagents(&t.id).await {
            subagents.insert(
                t.id.clone(),
                sas.into_iter()
                    .map(|sa| Subagent {
                        id: sa.id,
                        name: sa.name,
                        description: sa.description,
                        status: match sa.status {
                            subagent_store::SubagentStatus::Pending => SubagentStatus::Pending,
                            subagent_store::SubagentStatus::Running => SubagentStatus::Running,
                            subagent_store::SubagentStatus::Completed => SubagentStatus::Completed,
                            subagent_store::SubagentStatus::Failed => SubagentStatus::Failed,
                        },
                    })
                    .collect(),
            );
        }
    }

    let active_id = threads.first().map(|t| t.id.clone());
    {
        let mut ts = thread_state.write();
        ts.threads = threads;
        ts.active_thread_id = active_id;
    }
    {
        let mut cs = chat_state.write();
        cs.messages = messages;
    }
    {
        let mut tsk = tasks_state.write();
        tsk.todos = todos;
    }
    {
        let mut ss = subagent_state.write();
        ss.subagents = subagents;
    }
    model_state.write().selected_model = selected_model;
}
