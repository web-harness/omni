#[cfg(target_arch = "wasm32")]
use super::utils::app_url;
use super::{AgentEndpoint, BrowserInferenceStatus};
#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use serde::Deserialize;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use super::{
    BackgroundTask, FileInfo, ModelConfig, Provider, Todo, ToolCall, ToolResult, UiMessage,
    UiThread,
};

#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = "/public/omni-inference-client.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = getBrowserInferenceStatus)]
    async fn js_get_browser_inference_status() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = startBrowserModelDownload)]
    async fn js_start_browser_model_download(model_id: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = stopBrowserModelDownload)]
    async fn js_stop_browser_model_download(model_id: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = deleteBrowserModel)]
    async fn js_delete_browser_model(model_id: &str) -> Result<JsValue, JsValue>;
}

#[cfg(target_arch = "wasm32")]
#[derive(Deserialize)]
pub struct BootstrapPayload {
    pub threads: Vec<UiThread>,
    pub messages: HashMap<String, Vec<UiMessage>>,
    pub todos: HashMap<String, Vec<Todo>>,
    pub files: HashMap<String, Vec<FileInfo>>,
    pub tool_calls: HashMap<String, Vec<ToolCall>>,
    pub tool_results: HashMap<String, Vec<ToolResult>>,
    #[serde(default)]
    pub background_tasks: HashMap<String, Vec<BackgroundTask>>,
    pub workspace_path: HashMap<String, String>,
    pub workspace_files: HashMap<String, Vec<FileInfo>>,
    pub providers: Vec<Provider>,
    pub models: Vec<ModelConfig>,
    pub default_model: String,
    #[serde(default = "default_dicebear_style")]
    pub dicebear_style: String,
    #[serde(default)]
    pub agent_endpoints: Vec<AgentEndpoint>,
}

#[cfg(target_arch = "wasm32")]
fn default_dicebear_style() -> String {
    "bottts-neutral".into()
}

#[cfg(target_arch = "wasm32")]
#[derive(Deserialize)]
struct ItemResponse {
    value: serde_json::Value,
}

#[cfg(target_arch = "wasm32")]
#[derive(Deserialize)]
#[allow(dead_code)]
struct SearchItemsResponse {
    items: Vec<ItemResponse>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Deserialize)]
struct ProtocolThreadResponse {
    thread_id: String,
    metadata: HashMap<String, serde_json::Value>,
    status: String,
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

#[cfg(target_arch = "wasm32")]
fn js_value_to_string(value: &JsValue) -> String {
    js_sys::JSON::stringify(value)
        .ok()
        .and_then(|text| text.as_string())
        .unwrap_or_else(|| {
            value
                .as_string()
                .unwrap_or_else(|| "unknown javascript error".to_string())
        })
}

#[cfg(not(target_arch = "wasm32"))]
fn unavailable() -> std::io::Error {
    std::io::Error::other("sw_api is only available on wasm32 targets")
}

#[cfg(target_arch = "wasm32")]
fn parse_thread_status(raw: &str) -> super::ThreadStatus {
    match raw {
        "busy" => super::ThreadStatus::Busy,
        "interrupted" => super::ThreadStatus::Interrupted,
        "error" => super::ThreadStatus::Error,
        _ => super::ThreadStatus::Idle,
    }
}

#[cfg(target_arch = "wasm32")]
fn store_item_url(namespace: &[&str], key: &str) -> String {
    let mut url = app_url("store/items?");
    for segment in namespace {
        let encoded = js_sys::encode_uri_component(segment)
            .as_string()
            .unwrap_or_else(|| (*segment).to_string());
        url.push_str("namespace=");
        url.push_str(&encoded);
        url.push('&');
    }
    let encoded_key = js_sys::encode_uri_component(key)
        .as_string()
        .unwrap_or_else(|| key.to_string());
    url.push_str("key=");
    url.push_str(&encoded_key);
    url
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_bootstrap() -> Result<BootstrapPayload, std::io::Error> {
    let response = Request::get(&app_url("x/bootstrap"))
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
    let response = Request::post(&app_url("threads"))
        .json(&serde_json::json!({
            "metadata": {
                "title": "New Thread"
            }
        }))
        .map_err(err_msg)?
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
        .json::<ProtocolThreadResponse>()
        .await
        .map_err(err_msg)?;

    Ok(UiThread {
        id: created.thread_id,
        title: created
            .metadata
            .get("title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("New Thread")
            .to_string(),
        status: parse_thread_status(&created.status),
        updated_at: created.updated_at,
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_thread(thread_id: &str) -> Result<(), std::io::Error> {
    let response = Request::delete(&app_url(&format!("threads/{thread_id}")))
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
    let response = Request::get(&store_item_url(&["config", "api-keys"], provider))
        .send()
        .await
        .map_err(err_msg)?;

    if !response.ok() {
        return Err(std::io::Error::other(format!(
            "get api key failed: {}",
            response.status()
        )));
    }

    let payload = response.json::<ItemResponse>().await.map_err(err_msg)?;
    Ok(payload.value.as_str().unwrap_or_default().to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn set_api_key(provider: &str, value: &str) -> Result<(), std::io::Error> {
    let response = Request::put(&app_url("store/items"))
        .json(&serde_json::json!({
            "namespace": ["config", "api-keys"],
            "key": provider,
            "value": value,
        }))
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
    let response = Request::delete(&app_url("store/items"))
        .json(&serde_json::json!({
            "namespace": ["config", "api-keys"],
            "key": provider,
        }))
        .map_err(err_msg)?
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
#[allow(dead_code)]
pub async fn list_agent_endpoints() -> Result<Vec<AgentEndpoint>, std::io::Error> {
    let response = Request::post(&app_url("store/items/search"))
        .json(&serde_json::json!({
            "namespace_prefix": ["config", "agent-endpoints"],
            "limit": 200,
            "offset": 0,
        }))
        .map_err(err_msg)?
        .send()
        .await
        .map_err(err_msg)?;

    if !response.ok() {
        return Err(std::io::Error::other(format!(
            "list agent endpoints failed: {}",
            response.status()
        )));
    }

    let payload = response
        .json::<SearchItemsResponse>()
        .await
        .map_err(err_msg)?;
    payload
        .items
        .into_iter()
        .map(|item| serde_json::from_value::<AgentEndpoint>(item.value).map_err(err_msg))
        .collect()
}

#[cfg(target_arch = "wasm32")]
pub async fn set_agent_endpoint(endpoint: &AgentEndpoint) -> Result<(), std::io::Error> {
    let response = Request::put(&app_url("store/items"))
        .json(&serde_json::json!({
            "namespace": ["config", "agent-endpoints"],
            "key": endpoint.id,
            "value": endpoint,
        }))
        .map_err(err_msg)?
        .send()
        .await
        .map_err(err_msg)?;

    if response.ok() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "set agent endpoint failed: {}",
            response.status()
        )))
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_agent_endpoint(id: &str) -> Result<(), std::io::Error> {
    let response = Request::delete(&app_url("store/items"))
        .json(&serde_json::json!({
            "namespace": ["config", "agent-endpoints"],
            "key": id,
        }))
        .map_err(err_msg)?
        .send()
        .await
        .map_err(err_msg)?;

    if response.ok() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "delete agent endpoint failed: {}",
            response.status()
        )))
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn set_agent_rail_style(style: &str) -> Result<(), std::io::Error> {
    let response = Request::put(&app_url("store/items"))
        .json(&serde_json::json!({
            "namespace": ["config", "agent-rail"],
            "key": "dicebear-style",
            "value": {
                "style": style,
            },
        }))
        .map_err(err_msg)?
        .send()
        .await
        .map_err(err_msg)?;

    if response.ok() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "set agent rail style failed: {}",
            response.status()
        )))
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn list_providers_with_keys() -> Result<Vec<Provider>, std::io::Error> {
    let response = Request::get(&app_url("x/providers"))
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
    let response = Request::put(&app_url("store/items"))
        .json(&serde_json::json!({
            "namespace": ["config"],
            "key": "default_model",
            "value": {
                "model_id": model_id,
            },
        }))
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
pub async fn get_browser_inference_status() -> Result<BrowserInferenceStatus, std::io::Error> {
    let value = js_get_browser_inference_status()
        .await
        .map_err(|error| std::io::Error::other(js_value_to_string(&error)))?;
    serde_wasm_bindgen::from_value(value).map_err(err_msg)
}

#[cfg(target_arch = "wasm32")]
pub async fn start_browser_model_download(model_id: &str) -> Result<(), std::io::Error> {
    js_start_browser_model_download(model_id)
        .await
        .map(|_| ())
        .map_err(|error| std::io::Error::other(js_value_to_string(&error)))
}

#[cfg(target_arch = "wasm32")]
pub async fn stop_browser_model_download(model_id: &str) -> Result<(), std::io::Error> {
    js_stop_browser_model_download(model_id)
        .await
        .map(|_| ())
        .map_err(|error| std::io::Error::other(js_value_to_string(&error)))
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_browser_model(model_id: &str) -> Result<(), std::io::Error> {
    js_delete_browser_model(model_id)
        .await
        .map(|_| ())
        .map_err(|error| std::io::Error::other(js_value_to_string(&error)))
}

#[cfg(target_arch = "wasm32")]
pub async fn list_workspace_files(workspace: &str) -> Result<Vec<super::FileInfo>, std::io::Error> {
    let encoded = js_sys::encode_uri_component(workspace)
        .as_string()
        .unwrap_or_else(|| "/home/workspace".to_string());
    let response = Request::get(&app_url(&format!("x/files?workspace={encoded}")))
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

#[cfg(not(target_arch = "wasm32"))]
pub async fn set_default_model(_model_id: &str) -> Result<(), std::io::Error> {
    Err(unavailable())
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub async fn list_agent_endpoints() -> Result<Vec<AgentEndpoint>, std::io::Error> {
    Err(unavailable())
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub async fn set_agent_endpoint(_endpoint: &AgentEndpoint) -> Result<(), std::io::Error> {
    Err(unavailable())
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub async fn delete_agent_endpoint(_id: &str) -> Result<(), std::io::Error> {
    Err(unavailable())
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub async fn set_agent_rail_style(_style: &str) -> Result<(), std::io::Error> {
    Err(unavailable())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_browser_inference_status() -> Result<BrowserInferenceStatus, std::io::Error> {
    Err(unavailable())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn start_browser_model_download(_model_id: &str) -> Result<(), std::io::Error> {
    Err(unavailable())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn stop_browser_model_download(_model_id: &str) -> Result<(), std::io::Error> {
    Err(unavailable())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_browser_model(_model_id: &str) -> Result<(), std::io::Error> {
    Err(unavailable())
}
