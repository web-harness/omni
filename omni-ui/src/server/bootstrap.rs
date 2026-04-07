use crate::lib::sw_api::BootstrapPayload;
use crate::lib::{
    agent_config_hash, AgentEndpoint, BackgroundTask, BackgroundTaskStatus, FileInfo, ModelConfig,
    Provider, ProviderId, ThreadStatus, Todo, TodoStatus, ToolCall, ToolResult, UiMessage,
    UiThread,
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
            title: title.clone(),
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
        let bootstrap_tool_calls = seeded_tool_calls_for(&title, &thread_todos);
        let bootstrap_tool_results = seeded_tool_results_for(&title, &thread_todos);
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
        let mut bootstrap_tool_calls = bootstrap_tool_calls;
        bootstrap_tool_calls.extend(seeded_background_task_calls_for(&title, &thread_background));
        background_tasks.insert(thread_id.clone(), thread_background);

        files.insert(thread_id.clone(), Vec::new());
        tool_calls.insert(thread_id.clone(), bootstrap_tool_calls);
        tool_results.insert(thread_id, bootstrap_tool_results);
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
        agent_endpoints: seeded_agent_endpoints(read_agent_endpoints().await?),
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
            let seeded_size = omni_rt::deepagents::workspace_seed::seeded_size(&path);
            files.push(FileInfo {
                path,
                is_dir: false,
                size: seeded_size.or_else(|| stat.map(|item| item.size)),
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

const NATIVE_DEMO_THREAD_TITLE: &str = "Implement todo management sys...";

fn seeded_agent_endpoints(endpoints: Vec<AgentEndpoint>) -> Vec<AgentEndpoint> {
    if !endpoints.is_empty() {
        return endpoints;
    }

    vec![
        AgentEndpoint {
            id: agent_config_hash("https://agent.example.com/api", "sk-mock-1"),
            url: "https://agent.example.com/api".to_string(),
            bearer_token: "sk-mock-1".to_string(),
            name: "Research Agent".to_string(),
            removable: true,
        },
        AgentEndpoint {
            id: agent_config_hash("https://agent2.example.com/api", "sk-mock-2"),
            url: "https://agent2.example.com/api".to_string(),
            bearer_token: "sk-mock-2".to_string(),
            name: "Code Review Agent".to_string(),
            removable: true,
        },
    ]
}

fn seeded_tool_calls_for(title: &str, todos: &[Todo]) -> Vec<ToolCall> {
    if title != NATIVE_DEMO_THREAD_TITLE || todos.is_empty() {
        return Vec::new();
    }

    vec![ToolCall {
        id: "tc-todos".to_string(),
        name: "update_todos".to_string(),
        args: serde_json::json!({
            "todos": todos
                .iter()
                .map(|todo| serde_json::json!({
                    "content": todo.content,
                    "status": match todo.status {
                        TodoStatus::Pending => "pending",
                        TodoStatus::InProgress => "in_progress",
                        TodoStatus::Completed => "completed",
                        TodoStatus::Cancelled => "cancelled",
                    }
                }))
                .collect::<Vec<_>>()
        }),
    }]
}

fn seeded_background_task_calls_for(title: &str, tasks: &[BackgroundTask]) -> Vec<ToolCall> {
    if title != NATIVE_DEMO_THREAD_TITLE {
        return Vec::new();
    }

    tasks
        .iter()
        .map(|task| ToolCall {
            id: format!("tc-{}", task.id),
            name: "dispatch_subagent".to_string(),
            args: serde_json::json!({ "task": task.description }),
        })
        .collect()
}

fn seeded_tool_results_for(title: &str, todos: &[Todo]) -> Vec<ToolResult> {
    if title != NATIVE_DEMO_THREAD_TITLE || todos.is_empty() {
        return Vec::new();
    }

    vec![ToolResult {
        tool_call_id: "tc-todos".to_string(),
        content: "Synced".to_string(),
        is_error: false,
    }]
}

#[cfg(test)]
mod tests {
    use super::{
        seeded_agent_endpoints, seeded_background_task_calls_for, seeded_tool_calls_for,
        seeded_tool_results_for, NATIVE_DEMO_THREAD_TITLE,
    };

    #[test]
    fn falls_back_to_seeded_agent_endpoints_when_store_is_empty() {
        let endpoints = seeded_agent_endpoints(Vec::new());
        assert_eq!(endpoints.len(), 2);
        assert_eq!(endpoints[0].name, "Research Agent");
        assert_eq!(endpoints[1].name, "Code Review Agent");
        assert!(endpoints.iter().all(|endpoint| endpoint.removable));
    }

    #[test]
    fn preserves_stored_agent_endpoints_when_present() {
        let stored = vec![crate::lib::AgentEndpoint {
            id: "custom".into(),
            url: "https://custom.example.com".into(),
            bearer_token: "secret".into(),
            name: "Custom Agent".into(),
            removable: true,
        }];

        let endpoints = seeded_agent_endpoints(stored.clone());
        assert_eq!(endpoints, stored);
    }

    #[test]
    fn seeds_tool_calls_for_native_demo_thread() {
        let todos = vec![
            crate::lib::Todo {
                id: "todo1".into(),
                content: "Design TodoStore data structure".into(),
                status: crate::lib::TodoStatus::Completed,
            },
            crate::lib::Todo {
                id: "todo2".into(),
                content: "Implement CRUD operations".into(),
                status: crate::lib::TodoStatus::InProgress,
            },
        ];

        let calls = seeded_tool_calls_for(NATIVE_DEMO_THREAD_TITLE, &todos);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "update_todos");
        assert_eq!(
            calls[0]
                .args
                .get("todos")
                .and_then(serde_json::Value::as_array)
                .map(|todos| todos.len()),
            Some(2)
        );
        assert_eq!(
            calls[0]
                .args
                .get("todos")
                .and_then(serde_json::Value::as_array)
                .and_then(|todos| todos.get(1))
                .and_then(|todo| todo.get("status"))
                .and_then(serde_json::Value::as_str),
            Some("in_progress")
        );
    }

    #[test]
    fn seeds_background_task_calls_for_native_demo_thread() {
        let tasks = vec![crate::lib::BackgroundTask {
            id: "sa1".into(),
            name: "Researcher".into(),
            description: "Investigate GTD".into(),
            status: crate::lib::BackgroundTaskStatus::Running,
        }];

        let calls = seeded_background_task_calls_for(NATIVE_DEMO_THREAD_TITLE, &tasks);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "tc-sa1");
        assert_eq!(calls[0].name, "dispatch_subagent");
        assert_eq!(
            calls[0]
                .args
                .get("task")
                .and_then(serde_json::Value::as_str),
            Some("Investigate GTD")
        );
    }

    #[test]
    fn seeds_tool_results_for_native_demo_thread() {
        let todos = vec![crate::lib::Todo {
            id: "todo1".into(),
            content: "Design TodoStore data structure".into(),
            status: crate::lib::TodoStatus::Completed,
        }];

        let results = seeded_tool_results_for(NATIVE_DEMO_THREAD_TITLE, &todos);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tool_call_id, "tc-todos");
        assert!(!results[0].is_error);
    }
}
