#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use serde::Deserialize;

#[cfg(target_arch = "wasm32")]
use super::{
    FileInfo, ModelConfig, Provider, Subagent, Todo, ToolCall, ToolResult, UiMessage, UiThread,
};

#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
#[derive(Deserialize)]
pub struct BootstrapPayload {
    pub threads: Vec<UiThread>,
    pub messages: HashMap<String, Vec<UiMessage>>,
    pub todos: HashMap<String, Vec<Todo>>,
    pub files: HashMap<String, Vec<FileInfo>>,
    pub tool_calls: HashMap<String, Vec<ToolCall>>,
    pub tool_results: HashMap<String, Vec<ToolResult>>,
    pub subagents: HashMap<String, Vec<Subagent>>,
    pub workspace_path: HashMap<String, String>,
    pub workspace_files: HashMap<String, Vec<FileInfo>>,
    pub providers: Vec<Provider>,
    pub models: Vec<ModelConfig>,
    pub default_model: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Deserialize)]
struct ApiKeyResponse {
    value: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Deserialize)]
struct CreateThreadResponse {
    id: String,
    title: String,
    status: super::ThreadStatus,
    updated_at: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Deserialize)]
struct ProvidersResponseItem {
    id: super::ProviderId,
    name: String,
    has_api_key: bool,
}

#[cfg(target_arch = "wasm32")]
fn err_msg(err: impl ToString) -> std::io::Error {
    std::io::Error::other(err.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn unavailable() -> std::io::Error {
    std::io::Error::other("sw_api is only available on wasm32 targets")
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_bootstrap() -> Result<BootstrapPayload, std::io::Error> {
    let response = Request::get("/api/store/bootstrap")
        .send()
        .await
        .map_err(err_msg)?;

    if !response.ok() {
        return Err(std::io::Error::other(format!(
            "bootstrap request failed: {}",
            response.status()
        )));
    }

    response.json::<BootstrapPayload>().await.map_err(err_msg)
}

#[cfg(target_arch = "wasm32")]
pub async fn create_thread() -> Result<UiThread, std::io::Error> {
    let response = Request::post("/api/store/threads")
        .send()
        .await
        .map_err(err_msg)?;

    if !response.ok() {
        return Err(std::io::Error::other(format!(
            "create thread failed: {}",
            response.status()
        )));
    }

    let created = response
        .json::<CreateThreadResponse>()
        .await
        .map_err(err_msg)?;

    Ok(UiThread {
        id: created.id,
        title: created.title,
        status: created.status,
        updated_at: created.updated_at,
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_thread(thread_id: &str) -> Result<(), std::io::Error> {
    let response = Request::delete(&format!("/api/store/threads/{thread_id}"))
        .send()
        .await
        .map_err(err_msg)?;

    if response.ok() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "delete thread failed: {}",
            response.status()
        )))
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn get_api_key(provider: &str) -> Result<String, std::io::Error> {
    let response = Request::get(&format!("/api/store/config/api-keys/{provider}"))
        .send()
        .await
        .map_err(err_msg)?;

    if !response.ok() {
        return Err(std::io::Error::other(format!(
            "get api key failed: {}",
            response.status()
        )));
    }

    let payload = response.json::<ApiKeyResponse>().await.map_err(err_msg)?;
    Ok(payload.value)
}

#[cfg(target_arch = "wasm32")]
pub async fn set_api_key(provider: &str, value: &str) -> Result<(), std::io::Error> {
    let response = Request::put(&format!("/api/store/config/api-keys/{provider}"))
        .json(&serde_json::json!({ "value": value }))
        .map_err(err_msg)?
        .send()
        .await
        .map_err(err_msg)?;

    if response.ok() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "set api key failed: {}",
            response.status()
        )))
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_api_key(provider: &str) -> Result<(), std::io::Error> {
    let response = Request::delete(&format!("/api/store/config/api-keys/{provider}"))
        .send()
        .await
        .map_err(err_msg)?;

    if response.ok() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "delete api key failed: {}",
            response.status()
        )))
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn list_providers_with_keys() -> Result<Vec<Provider>, std::io::Error> {
    let response = Request::get("/api/store/providers")
        .send()
        .await
        .map_err(err_msg)?;

    if !response.ok() {
        return Err(std::io::Error::other(format!(
            "list providers failed: {}",
            response.status()
        )));
    }

    let providers = response
        .json::<Vec<ProvidersResponseItem>>()
        .await
        .map_err(err_msg)?;

    Ok(providers
        .into_iter()
        .map(|p| Provider {
            id: p.id,
            name: p.name,
            has_api_key: p.has_api_key,
        })
        .collect())
}

#[cfg(target_arch = "wasm32")]
pub async fn set_default_model(model_id: &str) -> Result<(), std::io::Error> {
    let response = Request::put("/api/store/config/default-model")
        .json(&serde_json::json!({ "model_id": model_id }))
        .map_err(err_msg)?
        .send()
        .await
        .map_err(err_msg)?;

    if response.ok() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "set default model failed: {}",
            response.status()
        )))
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn list_workspace_files(workspace: &str) -> Result<Vec<super::FileInfo>, std::io::Error> {
    let encoded = js_sys::encode_uri_component(workspace)
        .as_string()
        .unwrap_or_else(|| "/home/workspace".to_string());
    let response = Request::get(&format!("/api/store/files?workspace={encoded}"))
        .send()
        .await
        .map_err(err_msg)?;

    if !response.ok() {
        return Err(std::io::Error::other(format!(
            "list files failed: {}",
            response.status()
        )));
    }

    response
        .json::<Vec<super::FileInfo>>()
        .await
        .map_err(err_msg)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_workspace_files(
    _workspace: &str,
) -> Result<Vec<super::FileInfo>, std::io::Error> {
    Err(unavailable())
}
