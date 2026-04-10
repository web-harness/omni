use crate::server::bootstrap;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use omni_rt::deepagents::{
    checkpoint_store, config_store, message_store, subagent_store, thread_store, todo_store,
};
use omni_rt::protocol::{
    Agent, AgentCapabilities, AgentSchema, TableCheckpoint, ThreadCreate, ThreadPatch, ThreadState,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

const STORE_DIR: &str = "/home/store";

pub fn router() -> Router {
    Router::new()
        .route("/x/bootstrap", get(get_bootstrap))
        .route("/x/providers", get(get_providers))
        .route("/agents/search", post(search_agents))
        .route("/agents/{id}", get(get_agent))
        .route("/agents/{id}/schemas", get(get_agent_schema))
        .route("/threads", post(create_thread))
        .route("/threads/search", post(search_threads))
        .route(
            "/threads/{id}",
            get(get_thread).patch(patch_thread).delete(delete_thread),
        )
        .route("/threads/{id}/history", get(thread_history))
        .route("/threads/{id}/copy", post(copy_thread))
        .route(
            "/store/items",
            get(get_store_item)
                .put(put_store_item)
                .delete(delete_store_item),
        )
        .route("/store/items/search", post(search_store_items))
        .route("/store/namespaces", post(list_store_namespaces))
}

#[derive(Debug)]
pub(crate) struct ApiError(StatusCode, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(serde_json::json!({ "message": self.1 }))).into_response()
    }
}

type ApiResult<T> = Result<Json<T>, ApiError>;

#[derive(Deserialize)]
struct ThreadSearchBody {
    #[serde(default)]
    metadata: HashMap<String, Value>,
    values: Option<Value>,
    status: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}

#[derive(Deserialize)]
struct ThreadHistoryQuery {
    before: Option<Uuid>,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(Deserialize)]
struct StoreItemQuery {
    #[serde(default)]
    namespace: Vec<String>,
    key: String,
}

#[derive(Deserialize)]
struct StoreItemBody {
    #[serde(default)]
    namespace: Vec<String>,
    key: String,
    value: Value,
}

#[derive(Deserialize)]
struct StoreDeleteBody {
    #[serde(default)]
    namespace: Vec<String>,
    key: String,
}

#[derive(Deserialize)]
struct SearchItemsBody {
    namespace_prefix: Option<Vec<String>>,
    filter: Option<Value>,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}

#[derive(Deserialize)]
struct NamespaceSearchBody {
    prefix: Option<Vec<String>>,
    suffix: Option<Vec<String>>,
    max_depth: Option<usize>,
    #[serde(default = "default_namespace_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}

#[derive(Deserialize)]
struct AgentSearchBody {
    name: Option<String>,
    metadata: Option<HashMap<String, Value>>,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}

#[derive(Serialize, Deserialize, Clone)]
struct StoredItem {
    namespace: Vec<String>,
    key: String,
    value: Value,
    created_at: String,
    updated_at: String,
}

async fn get_bootstrap() -> ApiResult<crate::lib::sw_api::BootstrapPayload> {
    bootstrap::build_bootstrap()
        .await
        .map(Json)
        .map_err(io_error)
}

async fn get_providers() -> ApiResult<Vec<crate::lib::Provider>> {
    let payload = bootstrap::build_bootstrap().await.map_err(io_error)?;
    Ok(Json(payload.providers))
}

async fn search_agents(Json(body): Json<AgentSearchBody>) -> ApiResult<Vec<Agent>> {
    let agent = deepagent();
    let name_matches = body
        .name
        .as_deref()
        .map(|name| agent.name.to_lowercase().contains(&name.to_lowercase()))
        .unwrap_or(true);
    let metadata_matches = body
        .metadata
        .as_ref()
        .map(matches_agent_metadata)
        .unwrap_or(true);
    let agents = if name_matches && metadata_matches {
        vec![agent]
    } else {
        Vec::new()
    };
    Ok(Json(
        agents
            .into_iter()
            .skip(body.offset)
            .take(body.limit)
            .collect(),
    ))
}

async fn get_agent(Path(id): Path<String>) -> ApiResult<Agent> {
    if id == "deepagent" {
        Ok(Json(deepagent()))
    } else {
        Err(ApiError(
            StatusCode::NOT_FOUND,
            "Agent not found".to_string(),
        ))
    }
}

async fn get_agent_schema(Path(id): Path<String>) -> ApiResult<AgentSchema> {
    if id == "deepagent" {
        Ok(Json(deepagent_schema()))
    } else {
        Err(ApiError(
            StatusCode::NOT_FOUND,
            "Agent not found".to_string(),
        ))
    }
}

async fn create_thread(
    Json(body): Json<serde_json::Value>,
) -> ApiResult<omni_rt::protocol::Thread> {
    let request = serde_json::from_value::<ThreadCreate>(body).map_err(bad_request)?;
    thread_store::create_thread_from_request(request)
        .await
        .map(Json)
        .map_err(io_error)
}

async fn search_threads(
    Json(body): Json<ThreadSearchBody>,
) -> ApiResult<Vec<omni_rt::protocol::Thread>> {
    let mut threads = thread_store::list_threads().await.map_err(io_error)?;
    threads.retain(|thread| {
        let metadata_matches = body
            .metadata
            .iter()
            .all(|(key, value)| thread.metadata.get(key) == Some(value));
        let values_matches = body
            .values
            .as_ref()
            .map(|value| thread.values.as_ref() == Some(value))
            .unwrap_or(true);
        let status_matches = body
            .status
            .as_deref()
            .map(|status| thread.status == parse_status(status))
            .unwrap_or(true);
        metadata_matches && values_matches && status_matches
    });
    threads.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(Json(
        threads
            .into_iter()
            .skip(body.offset)
            .take(body.limit)
            .collect(),
    ))
}

async fn get_thread(Path(id): Path<String>) -> ApiResult<omni_rt::protocol::Thread> {
    thread_store::get_thread(&id)
        .await
        .map_err(io_error)?
        .map(Json)
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "Thread not found".to_string()))
}

async fn patch_thread(
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<omni_rt::protocol::Thread> {
    let patch = serde_json::from_value::<ThreadPatch>(body).map_err(bad_request)?;
    let checkpoint = patch.checkpoint.clone();
    let values_patch = patch.values.clone();
    let persisted_messages = patch.messages.clone();
    let has_messages_patch = persisted_messages.is_some();
    thread_store::update_thread(&id, patch)
        .await
        .map_err(io_error)?
        .map(|thread| async move {
            if let Some(messages) = persisted_messages {
                for message in messages {
                    let stored = message_store::StoredMessage::from_protocol_message(
                        id.clone(),
                        chrono::Utc::now().to_rfc3339(),
                        message,
                    );
                    message_store::save_message(&stored)
                        .await
                        .map_err(io_error)?;
                }
            }
            if values_patch.is_some() || has_messages_patch || checkpoint.is_some() {
                checkpoint_store::append_thread_state(
                    &id,
                    thread
                        .values
                        .clone()
                        .unwrap_or_else(|| serde_json::Value::Object(Default::default())),
                    thread.messages.clone(),
                    if thread.metadata.is_empty() {
                        None
                    } else {
                        Some(thread.metadata.clone())
                    },
                    checkpoint,
                )
                .await
                .map_err(io_error)?;
            }
            Ok(Json(thread))
        })
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "Thread not found".to_string()))?
        .await
}

async fn delete_thread(Path(id): Path<String>) -> Result<StatusCode, ApiError> {
    thread_store::delete_thread(&id).await.map_err(io_error)?;
    checkpoint_store::delete_thread_states(&id)
        .await
        .map_err(io_error)?;
    message_store::delete_thread_messages(&id)
        .await
        .map_err(io_error)?;
    todo_store::delete_thread_todos(&id)
        .await
        .map_err(io_error)?;
    subagent_store::delete_thread_subagents(&id)
        .await
        .map_err(io_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn thread_history(
    Path(id): Path<String>,
    Query(query): Query<ThreadHistoryQuery>,
) -> ApiResult<Vec<ThreadState>> {
    let Some(thread) = thread_store::get_thread(&id).await.map_err(io_error)? else {
        return Err(ApiError(
            StatusCode::NOT_FOUND,
            "Thread not found".to_string(),
        ));
    };

    let states = checkpoint_store::list_thread_states(&id)
        .await
        .map_err(io_error)?;

    let mut states = if let Some(before) = query.before {
        states
            .into_iter()
            .skip_while(|state| state.checkpoint.checkpoint_id != before)
            .skip(1)
            .take(query.limit)
            .collect::<Vec<_>>()
    } else {
        states.into_iter().take(query.limit).collect::<Vec<_>>()
    };

    if states.is_empty() && query.before.is_none() {
        let messages = message_store::list_messages(&id)
            .await
            .map_err(io_error)?
            .into_iter()
            .map(|message| message.into_protocol_message())
            .collect::<Vec<_>>();
        states.push(ThreadState {
            checkpoint: TableCheckpoint {
                checkpoint_id: Uuid::new_v4(),
                extra: HashMap::new(),
            },
            values: thread
                .values
                .unwrap_or_else(|| serde_json::Value::Object(Default::default())),
            messages: if messages.is_empty() {
                thread.messages
            } else {
                Some(messages)
            },
            metadata: if thread.metadata.is_empty() {
                None
            } else {
                Some(thread.metadata)
            },
        });
    }

    Ok(Json(states))
}

async fn copy_thread(Path(id): Path<String>) -> ApiResult<omni_rt::protocol::Thread> {
    let Some(source) = thread_store::get_thread(&id).await.map_err(io_error)? else {
        return Err(ApiError(
            StatusCode::NOT_FOUND,
            "Thread not found".to_string(),
        ));
    };

    let mut metadata = source.metadata.clone();
    if let Some(title) = metadata.get("title").and_then(Value::as_str) {
        metadata.insert("title".to_string(), Value::String(format!("{title} Copy")));
    }
    let copy = thread_store::create_thread_from_request(ThreadCreate {
        thread_id: None,
        metadata: Some(metadata),
        if_exists: None,
    })
    .await
    .map_err(io_error)?;
    let new_id = copy.thread_id.to_string();

    for message in message_store::list_messages(&id).await.map_err(io_error)? {
        message_store::save_message(&message_store::StoredMessage {
            thread_id: new_id.clone(),
            id: Uuid::new_v4().to_string(),
            ..message
        })
        .await
        .map_err(io_error)?;
    }

    for todo in todo_store::list_todos(&id).await.map_err(io_error)? {
        todo_store::save_todo(&todo_store::StoredTodo {
            thread_id: new_id.clone(),
            id: Uuid::new_v4().to_string(),
            ..todo
        })
        .await
        .map_err(io_error)?;
    }

    for subagent in subagent_store::list_subagents(&id)
        .await
        .map_err(io_error)?
    {
        subagent_store::save_subagent(&subagent_store::StoredSubagent {
            thread_id: new_id.clone(),
            id: Uuid::new_v4().to_string(),
            ..subagent
        })
        .await
        .map_err(io_error)?;
    }

    checkpoint_store::copy_thread_states(&id, &new_id)
        .await
        .map_err(io_error)?;

    Ok(Json(copy))
}

async fn get_store_item(Query(query): Query<StoreItemQuery>) -> ApiResult<StoredItem> {
    let item = ensure_store_item(query.namespace, query.key)
        .await
        .map_err(io_error)?;
    item.map(Json)
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "Item not found".to_string()))
}

async fn put_store_item(Json(body): Json<StoreItemBody>) -> Result<StatusCode, ApiError> {
    save_store_item(body.namespace, body.key, body.value)
        .await
        .map_err(io_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_store_item(Json(body): Json<StoreDeleteBody>) -> Result<StatusCode, ApiError> {
    delete_store_value(body.namespace, body.key)
        .await
        .map_err(io_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn search_store_items(Json(body): Json<SearchItemsBody>) -> ApiResult<serde_json::Value> {
    let mut items = list_all_items().await.map_err(io_error)?;
    items.retain(|item| {
        let prefix_matches = body
            .namespace_prefix
            .as_ref()
            .map(|prefix| item.namespace.starts_with(prefix))
            .unwrap_or(true);
        let filter_matches = body
            .filter
            .as_ref()
            .map(|filter| item.value == *filter)
            .unwrap_or(true);
        prefix_matches && filter_matches
    });
    Ok(Json(serde_json::json!({
        "items": items.into_iter().skip(body.offset).take(body.limit).collect::<Vec<_>>()
    })))
}

async fn list_store_namespaces(
    Json(body): Json<NamespaceSearchBody>,
) -> ApiResult<Vec<Vec<String>>> {
    let items = list_all_items().await.map_err(io_error)?;
    let mut namespaces = BTreeSet::new();
    for item in items {
        let prefix_matches = body
            .prefix
            .as_ref()
            .map(|prefix| item.namespace.starts_with(prefix))
            .unwrap_or(true);
        let suffix_matches = body
            .suffix
            .as_ref()
            .map(|suffix| item.namespace.ends_with(suffix))
            .unwrap_or(true);
        let depth_matches = body
            .max_depth
            .map(|depth| item.namespace.len() <= depth)
            .unwrap_or(true);
        if prefix_matches && suffix_matches && depth_matches {
            namespaces.insert(item.namespace);
        }
    }

    Ok(Json(
        namespaces
            .into_iter()
            .skip(body.offset)
            .take(body.limit)
            .collect(),
    ))
}

async fn ensure_store_item(
    namespace: Vec<String>,
    key: String,
) -> Result<Option<StoredItem>, std::io::Error> {
    mirror_config_item(&namespace, &key).await?;
    read_store_item(&namespace, &key).await
}

async fn read_store_item(
    namespace: &[String],
    key: &str,
) -> Result<Option<StoredItem>, std::io::Error> {
    let path = store_item_path(&namespace, &key);
    if !omni_rt::zenfs::exists(&path).await? {
        return Ok(None);
    }
    let bytes = omni_rt::zenfs::read_file(&path).await?;
    serde_json::from_slice::<StoredItem>(&bytes)
        .map(Some)
        .map_err(|error| std::io::Error::other(error.to_string()))
}

async fn save_store_item(
    namespace: Vec<String>,
    key: String,
    value: Value,
) -> Result<(), std::io::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let existing = read_store_item(&namespace, &key).await?;
    let item = StoredItem {
        namespace: namespace.clone(),
        key: key.clone(),
        value: value.clone(),
        created_at: existing
            .as_ref()
            .map(|item| item.created_at.clone())
            .unwrap_or_else(|| now.clone()),
        updated_at: now,
    };
    let path = store_item_path(&namespace, &key);
    if let Some(parent) = path.rsplit_once('/') {
        omni_rt::zenfs::mkdir(parent.0, true).await?;
    }
    omni_rt::zenfs::write_file(
        &path,
        &serde_json::to_vec(&item).map_err(|error| std::io::Error::other(error.to_string()))?,
    )
    .await?;

    if is_api_key_namespace(&namespace) {
        if let Some(text) = value.as_str() {
            config_store::set_api_key(&key, text).await?;
        }
    } else if namespace == ["config".to_string()] && key == "default_model" {
        let model_id = value
            .get("model_id")
            .and_then(Value::as_str)
            .or_else(|| value.as_str())
            .unwrap_or_default();
        config_store::set_default_model(model_id).await?;
    }

    Ok(())
}

async fn delete_store_value(namespace: Vec<String>, key: String) -> Result<(), std::io::Error> {
    let path = store_item_path(&namespace, &key);
    if omni_rt::zenfs::exists(&path).await? {
        omni_rt::zenfs::remove(&path, false).await?;
    }
    if is_api_key_namespace(&namespace) {
        config_store::delete_api_key(&key).await?;
    } else if namespace == ["config".to_string()] && key == "default_model" {
        config_store::delete_default_model().await?;
    }
    Ok(())
}

async fn list_all_items() -> Result<Vec<StoredItem>, std::io::Error> {
    let mut items = Vec::new();
    if !omni_rt::zenfs::exists(STORE_DIR).await? {
        return Ok(items);
    }
    walk_store(STORE_DIR, &mut items).await?;
    Ok(items)
}

async fn walk_store(dir: &str, items: &mut Vec<StoredItem>) -> Result<(), std::io::Error> {
    for entry in omni_rt::zenfs::read_dir(dir).await? {
        let path = format!("{}/{}", dir.trim_end_matches('/'), entry.name);
        if entry.is_dir {
            Box::pin(walk_store(&path, items)).await?;
            continue;
        }
        if !entry.name.ends_with(".json") {
            continue;
        }
        let bytes = omni_rt::zenfs::read_file(&path).await?;
        if let Ok(item) = serde_json::from_slice::<StoredItem>(&bytes) {
            items.push(item);
        }
    }
    Ok(())
}

async fn mirror_config_item(namespace: &[String], key: &str) -> Result<(), std::io::Error> {
    if is_api_key_namespace(namespace) {
        if let Some(value) = config_store::get_api_key(key).await? {
            let path = store_item_path(namespace, key);
            if !omni_rt::zenfs::exists(&path).await? {
                save_store_item(namespace.to_vec(), key.to_string(), Value::String(value)).await?;
            }
        }
    } else if namespace == ["config".to_string()] && key == "default_model" {
        let value = config_store::get_stored_default_model().await?;
        if let Some(value) = value {
            let path = store_item_path(namespace, key);
            if !omni_rt::zenfs::exists(&path).await? {
                save_store_item(
                    namespace.to_vec(),
                    key.to_string(),
                    serde_json::json!({ "model_id": value }),
                )
                .await?;
            }
        }
    }
    Ok(())
}

fn store_item_path(namespace: &[String], key: &str) -> String {
    if namespace.is_empty() {
        format!("{STORE_DIR}/{key}.json")
    } else {
        format!("{STORE_DIR}/{}/{}.json", namespace.join("/"), key)
    }
}

fn is_api_key_namespace(namespace: &[String]) -> bool {
    namespace == ["config".to_string(), "api-keys".to_string()]
}

fn deepagent() -> Agent {
    Agent {
        agent_id: "deepagent".to_string(),
        name: "DeepAgent".to_string(),
        description: Some("Desktop DeepAgent runtime".to_string()),
        metadata: Some(HashMap::from([
            ("provider".to_string(), Value::String("omni".to_string())),
            ("runtime".to_string(), Value::String("desktop".to_string())),
        ])),
        capabilities: AgentCapabilities {
            ap_io_messages: Some(true),
            ap_io_streaming: Some(true),
            custom: HashMap::new(),
        },
    }
}

fn deepagent_schema() -> AgentSchema {
    AgentSchema {
        agent_id: "deepagent".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "thread_id": { "type": "string", "format": "uuid" },
                "input": {},
                "messages": { "type": "array" },
                "metadata": { "type": "object" }
            }
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "values": { "type": "object" },
                "messages": { "type": "array" }
            }
        }),
        state_schema: Some(serde_json::json!({ "type": "object" })),
        config_schema: Some(serde_json::json!({ "type": "object" })),
    }
}

fn matches_agent_metadata(metadata: &HashMap<String, Value>) -> bool {
    deepagent()
        .metadata
        .unwrap_or_default()
        .iter()
        .all(|(key, value)| metadata.get(key) == Some(value))
}

fn parse_status(raw: &str) -> omni_rt::protocol::ThreadStatus {
    match raw {
        "busy" => omni_rt::protocol::ThreadStatus::Busy,
        "interrupted" => omni_rt::protocol::ThreadStatus::Interrupted,
        "error" => omni_rt::protocol::ThreadStatus::Error,
        _ => omni_rt::protocol::ThreadStatus::Idle,
    }
}

fn default_limit() -> usize {
    10
}

fn default_namespace_limit() -> usize {
    100
}

fn bad_request(error: serde_json::Error) -> ApiError {
    ApiError(StatusCode::UNPROCESSABLE_ENTITY, error.to_string())
}

pub(crate) fn io_error(error: std::io::Error) -> ApiError {
    ApiError(StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
