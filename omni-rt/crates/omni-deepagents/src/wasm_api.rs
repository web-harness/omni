use crate::{config_store, message_store, mock_data, run_store, thread_store, workspace_seed};
use omni_protocol::{Message, RunSearchRequest, ThreadCreate, ThreadPatch, ThreadStatus};
use serde::Serialize;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;

fn to_js_error(error: std::io::Error) -> JsValue {
    JsValue::from_str(&error.to_string())
}

fn serialize<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    value
        .serialize(&serializer)
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub async fn deepagents_list_threads() -> Result<JsValue, JsValue> {
    serialize(&thread_store::list_threads().await.map_err(to_js_error)?)
}

#[wasm_bindgen]
pub async fn deepagents_get_thread(thread_id: String) -> Result<JsValue, JsValue> {
    serialize(
        &thread_store::get_thread(&thread_id)
            .await
            .map_err(to_js_error)?,
    )
}

#[wasm_bindgen]
pub async fn deepagents_create_thread(payload: JsValue) -> Result<JsValue, JsValue> {
    let request = from_value::<ThreadCreate>(payload)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    serialize(
        &thread_store::create_thread_from_request(request)
            .await
            .map_err(to_js_error)?,
    )
}

#[wasm_bindgen]
pub async fn deepagents_patch_thread(
    thread_id: String,
    payload: JsValue,
) -> Result<JsValue, JsValue> {
    let patch = from_value::<ThreadPatch>(payload)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    serialize(
        &thread_store::update_thread(&thread_id, patch)
            .await
            .map_err(to_js_error)?,
    )
}

#[wasm_bindgen]
pub async fn deepagents_save_thread(payload: JsValue) -> Result<(), JsValue> {
    let thread = from_value::<omni_protocol::Thread>(payload)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    thread_store::save_thread(&thread)
        .await
        .map_err(to_js_error)
}

#[wasm_bindgen]
pub async fn deepagents_set_thread_status(
    thread_id: String,
    status: String,
) -> Result<JsValue, JsValue> {
    let status = match status.trim().to_lowercase().as_str() {
        "idle" => ThreadStatus::Idle,
        "busy" => ThreadStatus::Busy,
        "interrupted" => ThreadStatus::Interrupted,
        "error" => ThreadStatus::Error,
        _ => return Err(JsValue::from_str("invalid thread status")),
    };
    serialize(
        &thread_store::set_thread_status(&thread_id, status)
            .await
            .map_err(to_js_error)?,
    )
}

#[wasm_bindgen]
pub async fn deepagents_delete_thread(thread_id: String) -> Result<(), JsValue> {
    thread_store::delete_thread(&thread_id)
        .await
        .map_err(to_js_error)
}

#[wasm_bindgen]
pub async fn deepagents_list_messages(thread_id: String) -> Result<JsValue, JsValue> {
    serialize(
        &message_store::list_messages(&thread_id)
            .await
            .map_err(to_js_error)?,
    )
}

#[wasm_bindgen]
pub async fn deepagents_save_message(
    thread_id: String,
    created_at: String,
    payload: JsValue,
) -> Result<(), JsValue> {
    let message =
        from_value::<Message>(payload).map_err(|error| JsValue::from_str(&error.to_string()))?;
    let stored =
        message_store::StoredMessage::from_protocol_message(thread_id, created_at, message);
    message_store::save_message(&stored)
        .await
        .map_err(to_js_error)
}

#[wasm_bindgen]
pub async fn deepagents_delete_thread_messages(thread_id: String) -> Result<(), JsValue> {
    message_store::delete_thread_messages(&thread_id)
        .await
        .map_err(to_js_error)
}

#[wasm_bindgen]
pub fn deepagents_workspace_seed_entries() -> Result<JsValue, JsValue> {
    serialize(&workspace_seed::workspace_seed_entry_views())
}

#[wasm_bindgen]
pub async fn deepagents_get_default_model() -> Result<JsValue, JsValue> {
    serialize(
        &config_store::get_default_model()
            .await
            .map_err(to_js_error)?,
    )
}

#[wasm_bindgen]
pub async fn deepagents_get_stored_default_model() -> Result<JsValue, JsValue> {
    serialize(
        &config_store::get_stored_default_model()
            .await
            .map_err(to_js_error)?,
    )
}

#[wasm_bindgen]
pub async fn deepagents_set_default_model(model_id: String) -> Result<(), JsValue> {
    config_store::set_default_model(&model_id)
        .await
        .map_err(to_js_error)
}

#[wasm_bindgen]
pub async fn deepagents_delete_default_model() -> Result<(), JsValue> {
    config_store::delete_default_model()
        .await
        .map_err(to_js_error)
}

#[wasm_bindgen]
pub async fn deepagents_get_api_key(provider: String) -> Result<JsValue, JsValue> {
    serialize(
        &config_store::get_api_key(&provider)
            .await
            .map_err(to_js_error)?,
    )
}

#[wasm_bindgen]
pub async fn deepagents_set_api_key(provider: String, value: String) -> Result<(), JsValue> {
    config_store::set_api_key(&provider, &value)
        .await
        .map_err(to_js_error)
}

#[wasm_bindgen]
pub async fn deepagents_delete_api_key(provider: String) -> Result<(), JsValue> {
    config_store::delete_api_key(&provider)
        .await
        .map_err(to_js_error)
}

#[wasm_bindgen]
pub async fn deepagents_list_runs() -> Result<JsValue, JsValue> {
    serialize(&run_store::list_runs().await.map_err(to_js_error)?)
}

#[wasm_bindgen]
pub async fn deepagents_save_run(payload: JsValue) -> Result<(), JsValue> {
    let run = from_value::<run_store::StoredRun>(payload)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    run_store::save_run(&run).await.map_err(to_js_error)
}

#[wasm_bindgen]
pub async fn deepagents_get_run(run_id: String) -> Result<JsValue, JsValue> {
    let run_id =
        uuid::Uuid::parse_str(&run_id).map_err(|error| JsValue::from_str(&error.to_string()))?;
    serialize(&run_store::get_run(run_id).await.map_err(to_js_error)?)
}

#[wasm_bindgen]
pub async fn deepagents_search_runs(payload: JsValue) -> Result<JsValue, JsValue> {
    let request = from_value::<RunSearchRequest>(payload)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    serialize(
        &run_store::search_runs(&request)
            .await
            .map_err(to_js_error)?,
    )
}

#[wasm_bindgen]
pub async fn deepagents_delete_run(run_id: String) -> Result<(), JsValue> {
    let run_id =
        uuid::Uuid::parse_str(&run_id).map_err(|error| JsValue::from_str(&error.to_string()))?;
    run_store::delete_run(run_id).await.map_err(to_js_error)
}

#[wasm_bindgen]
pub fn deepagents_mock_thread_ids() -> Result<JsValue, JsValue> {
    serialize(&mock_data::mock_thread_ids())
}

#[wasm_bindgen]
pub fn deepagents_seed_threads() -> Result<JsValue, JsValue> {
    serialize(&mock_data::seed_threads())
}

#[wasm_bindgen]
pub fn deepagents_seed_agent_endpoints() -> Result<JsValue, JsValue> {
    serialize(&mock_data::seed_agent_endpoints())
}

#[wasm_bindgen]
pub fn deepagents_hash_agent_config(url: String, bearer_token: String) -> String {
    mock_data::hash_agent_config(&url, &bearer_token)
}

#[wasm_bindgen]
pub fn deepagents_mock_thread_files(thread_id: String) -> Result<JsValue, JsValue> {
    serialize(&mock_data::mock_thread_files(&thread_id))
}

#[wasm_bindgen]
pub fn deepagents_mock_tool_calls(thread_id: String) -> Result<JsValue, JsValue> {
    serialize(&mock_data::mock_tool_calls(&thread_id))
}

#[wasm_bindgen]
pub fn deepagents_mock_tool_results(thread_id: String) -> Result<JsValue, JsValue> {
    serialize(&mock_data::mock_tool_results(&thread_id))
}

#[wasm_bindgen]
pub fn deepagents_mock_workspace_files() -> Result<JsValue, JsValue> {
    serialize(&mock_data::mock_workspace_files())
}

#[wasm_bindgen]
pub fn deepagents_scaffold_files() -> Result<JsValue, JsValue> {
    serialize(&mock_data::scaffold_files())
}
