use crate::lib::sw_api::BootstrapPayload;
use crate::lib::{
    AgentEndpoint, BackgroundTask, BackgroundTaskStatus, FileInfo, ModelConfig, Provider,
    ProviderId, ThreadStatus, Todo, TodoStatus, ToolCall, ToolResult, UiMessage, UiThread,
};
use omni_rt::deepagents::{
    config_store, message_store, model_registry, seed, subagent_store, thread_store, todo_store,
};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

const AGENT_ENDPOINTS_DIR: &str = "/home/store/config/agent-endpoints";
const AGENT_RAIL_DIR: &str = "/home/store/config/agent-rail";

#[derive(Deserialize)]
struct StoredItem {
    value: serde_json::Value,
}

pub async fn build_bootstrap() -> Result<BootstrapPayload, std::io::Error> {
    seed::seed_if_empty().await?;

    let threads = thread_store::list_threads().await?;
    let mut ui_threads = Vec::new();
    let mut messages = HashMap::new();
    let mut todos = HashMap::new();
    let mut files = HashMap::new();
    let mut tool_calls: HashMap<String, Vec<ToolCall>> = HashMap::new();
    let mut tool_results: HashMap<String, Vec<ToolResult>> = HashMap::new();
    let mut background_tasks = HashMap::new();
    let mut workspace_path = HashMap::new();
    let mut workspace_files = HashMap::new();

    for thread in threads {
        let thread_id = thread.thread_id.to_string();
        let workspace = thread
            .metadata
            .get("workspace")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("/home/workspace")
            .to_string();
        let title = thread
            .metadata
            .get("title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("New Thread")
            .to_string();

        ui_threads.push(UiThread {
            id: thread_id.clone(),
            title,
            status: match thread.status {
                omni_rt::protocol::ThreadStatus::Idle => ThreadStatus::Idle,
                omni_rt::protocol::ThreadStatus::Busy => ThreadStatus::Busy,
                omni_rt::protocol::ThreadStatus::Interrupted => ThreadStatus::Interrupted,
                omni_rt::protocol::ThreadStatus::Error => ThreadStatus::Error,
            },
            updated_at: thread.updated_at.to_rfc3339(),
        });

        workspace_path.insert(thread_id.clone(), workspace.clone());
        if !workspace_files.contains_key(&workspace) {
            workspace_files.insert(workspace.clone(), list_workspace_files(&workspace).await?);
        }

        let persisted_messages = message_store::list_messages(&thread_id).await?;
        let thread_messages = if persisted_messages.is_empty() {
            thread
                .messages
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|message| UiMessage {
                    id: message.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
                    role: match message.role.as_str() {
                        "user" => crate::lib::Role::User,
                        "tool" => crate::lib::Role::Tool,
                        _ => crate::lib::Role::Assistant,
                    },
                    content: message_content(&message.content),
                })
                .collect::<Vec<_>>()
        } else {
            persisted_messages
                .into_iter()
                .map(|message| UiMessage {
                    id: message.id,
                    role: match message.role.as_str() {
                        "user" => crate::lib::Role::User,
                        "tool" => crate::lib::Role::Tool,
                        _ => crate::lib::Role::Assistant,
                    },
                    content: message_content(&message.content),
                })
                .collect::<Vec<_>>()
        };
        messages.insert(thread_id.clone(), thread_messages);

        let thread_todos = todo_store::list_todos(&thread_id)
            .await?
            .into_iter()
            .map(|todo| Todo {
                id: todo.id,
                content: todo.content,
                status: match todo.status {
                    todo_store::TodoStatus::Pending => TodoStatus::Pending,
                    todo_store::TodoStatus::InProgress => TodoStatus::InProgress,
                    todo_store::TodoStatus::Completed => TodoStatus::Completed,
                    todo_store::TodoStatus::Cancelled => TodoStatus::Cancelled,
                },
            })
            .collect::<Vec<_>>();
        todos.insert(thread_id.clone(), thread_todos);

        let thread_background = subagent_store::list_subagents(&thread_id)
            .await?
            .into_iter()
            .map(|task| BackgroundTask {
                id: task.id,
                name: task.name,
                description: task.description,
                status: match task.status {
                    subagent_store::SubagentStatus::Pending => BackgroundTaskStatus::Pending,
                    subagent_store::SubagentStatus::Running => BackgroundTaskStatus::Running,
                    subagent_store::SubagentStatus::Completed => BackgroundTaskStatus::Completed,
                    subagent_store::SubagentStatus::Failed => BackgroundTaskStatus::Failed,
                },
            })
            .collect::<Vec<_>>();
        background_tasks.insert(thread_id.clone(), thread_background);

        files.insert(thread_id.clone(), Vec::new());
        tool_calls.entry(thread_id.clone()).or_default();
        tool_results.entry(thread_id).or_default();
    }

    ui_threads.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

    let providers = model_registry::list_providers_with_keys()
        .await?
        .into_iter()
        .map(|(provider, has_api_key)| Provider {
            id: match provider.id {
                model_registry::ProviderId::Anthropic => ProviderId::Anthropic,
                model_registry::ProviderId::OpenAI => ProviderId::OpenAI,
                model_registry::ProviderId::Google => ProviderId::Google,
                model_registry::ProviderId::Ollama => ProviderId::Ollama,
                model_registry::ProviderId::Browser => ProviderId::Browser,
            },
            name: provider.name,
            has_api_key,
        })
        .collect::<Vec<_>>();

    let models = model_registry::list_models()
        .into_iter()
        .map(|model| ModelConfig {
            id: model.id,
            name: model.name,
            provider: match model.provider {
                model_registry::ProviderId::Anthropic => ProviderId::Anthropic,
                model_registry::ProviderId::OpenAI => ProviderId::OpenAI,
                model_registry::ProviderId::Google => ProviderId::Google,
                model_registry::ProviderId::Ollama => ProviderId::Ollama,
                model_registry::ProviderId::Browser => ProviderId::Browser,
            },
        })
        .collect::<Vec<_>>();

    Ok(BootstrapPayload {
        threads: ui_threads,
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
        default_model: config_store::get_default_model().await?,
        dicebear_style: read_dicebear_style().await?,
        agent_endpoints: read_agent_endpoints().await?,
    })
}

pub async fn list_workspace_files(root: &str) -> Result<Vec<FileInfo>, std::io::Error> {
    let root = if root.is_empty() {
        "/home/workspace"
    } else {
        root
    };
    if !omni_rt::zenfs::exists(root).await? {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    walk_workspace(root, root, 0, &mut files).await?;
    Ok(files)
}

async fn walk_workspace(
    root: &str,
    current: &str,
    depth: usize,
    files: &mut Vec<FileInfo>,
) -> Result<(), std::io::Error> {
    if depth > 2 {
        return Ok(());
    }

    for entry in omni_rt::zenfs::read_dir(current).await? {
        let path = if current == "/" {
            format!("/{}", entry.name)
        } else {
            format!("{}/{}", current.trim_end_matches('/'), entry.name)
        };
        if entry.is_dir {
            files.push(FileInfo {
                path: path.clone(),
                is_dir: true,
                size: None,
            });
            Box::pin(walk_workspace(root, &path, depth + 1, files)).await?;
        } else if entry.is_file {
            let stat = omni_rt::zenfs::stat(&path).await.ok();
            files.push(FileInfo {
                path,
                is_dir: false,
                size: stat.map(|item| item.size),
            });
        }
    }

    if current == root {
        files.sort_by(|left, right| left.path.cmp(&right.path));
    }
    Ok(())
}

async fn read_agent_endpoints() -> Result<Vec<AgentEndpoint>, std::io::Error> {
    if !omni_rt::zenfs::exists(AGENT_ENDPOINTS_DIR).await? {
        return Ok(Vec::new());
    }

    let mut endpoints = Vec::new();
    for entry in omni_rt::zenfs::read_dir(AGENT_ENDPOINTS_DIR).await? {
        if !entry.name.ends_with(".json") {
            continue;
        }
        let path = format!("{AGENT_ENDPOINTS_DIR}/{}", entry.name);
        let value = omni_rt::zenfs::read_file(&path).await?;
        let item = serde_json::from_slice::<StoredItem>(&value)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        if let Ok(endpoint) = serde_json::from_value::<AgentEndpoint>(item.value) {
            endpoints.push(endpoint);
        }
    }
    Ok(endpoints)
}

async fn read_dicebear_style() -> Result<String, std::io::Error> {
    let path = format!("{AGENT_RAIL_DIR}/dicebear-style.json");
    if !omni_rt::zenfs::exists(&path).await? {
        return Ok("bottts-neutral".to_string());
    }

    let bytes = omni_rt::zenfs::read_file(&path).await?;
    let item = serde_json::from_slice::<StoredItem>(&bytes)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    Ok(item
        .value
        .get("style")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("bottts-neutral")
        .to_string())
}

fn message_content(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|item| item.get("text").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>()
            .join(" "),
        _ => value.to_string(),
    }
}
