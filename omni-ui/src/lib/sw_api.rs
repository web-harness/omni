use super::utils::app_url;
use super::{AgentEndpoint, BrowserInferenceStatus};
use reqwest::Method;
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use super::{
    BackgroundTask, FileInfo, ModelConfig, Provider, Todo, ToolCall, ToolResult, UiMessage,
    UiThread,
};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = "/omni-inference-client.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = getBrowserInferenceStatus)]
    async fn js_get_browser_inference_status() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = startBrowserModelDownload)]
    async fn js_start_browser_model_download(model_id: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = stopBrowserModelDownload)]
    async fn js_stop_browser_model_download(model_id: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = deleteBrowserModel)]
    async fn js_delete_browser_model(model_id: &str) -> Result<JsValue, JsValue>;

    type BrowserInferenceStatusStreamHandle;

    #[wasm_bindgen(catch, js_name = startBrowserInferenceStatusStream)]
    fn js_start_browser_inference_status_stream(
        callback: &Closure<dyn FnMut(JsValue)>,
    ) -> Result<BrowserInferenceStatusStreamHandle, JsValue>;

    #[wasm_bindgen(method, js_name = stop)]
    fn stop_browser_inference_status_stream(this: &BrowserInferenceStatusStreamHandle);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = "/omni-sw-register.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = loadBootstrapPayloadJson)]
    async fn js_load_bootstrap_payload_json() -> Result<JsValue, JsValue>;
}

#[cfg(target_arch = "wasm32")]
pub struct BrowserInferenceStatusSubscription {
    callback: Closure<dyn FnMut(JsValue)>,
    handle: BrowserInferenceStatusStreamHandle,
}

#[cfg(target_arch = "wasm32")]
impl Drop for BrowserInferenceStatusSubscription {
    fn drop(&mut self) {
        let _ = &self.callback;
        self.handle.stop_browser_inference_status_stream();
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct BrowserInferenceStatusSubscription;

#[derive(Deserialize, Serialize)]
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

fn default_dicebear_style() -> String {
    "bottts-neutral".into()
}

#[derive(Deserialize)]
struct ItemResponse {
    value: serde_json::Value,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct SearchItemsResponse {
    items: Vec<ItemResponse>,
}

#[derive(Deserialize)]
struct ProtocolThreadResponse {
    thread_id: String,
    metadata: HashMap<String, serde_json::Value>,
    status: String,
    updated_at: String,
}

#[derive(Deserialize)]
struct ProvidersResponseItem {
    id: super::ProviderId,
    name: String,
    has_api_key: bool,
}

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

fn parse_thread_status(raw: &str) -> super::ThreadStatus {
    match raw {
        "busy" => super::ThreadStatus::Busy,
        "interrupted" => super::ThreadStatus::Interrupted,
        "error" => super::ThreadStatus::Error,
        _ => super::ThreadStatus::Idle,
    }
}

fn store_item_url(namespace: &[&str], key: &str) -> String {
    let mut url = app_url("store/items?");
    for segment in namespace {
        let encoded = urlencoding::encode(segment).into_owned();
        url.push_str("namespace=");
        url.push_str(&encoded);
        url.push('&');
    }
    let encoded_key = urlencoding::encode(key).into_owned();
    url.push_str("key=");
    url.push_str(&encoded_key);
    url
}

fn http_client() -> reqwest::Client {
    reqwest::Client::new()
}

async fn send_json_request<T: serde::Serialize + ?Sized>(
    method: Method,
    url: String,
    body: Option<&T>,
) -> Result<reqwest::Response, std::io::Error> {
    let request = http_client().request(method, url);
    let request = if let Some(body) = body {
        request.json(body)
    } else {
        request
    };
    request.send().await.map_err(err_msg)
}

pub async fn fetch_bootstrap() -> Result<BootstrapPayload, std::io::Error> {
    #[cfg(target_arch = "wasm32")]
    {
        let value = js_load_bootstrap_payload_json()
            .await
            .map_err(|error| err_msg(js_value_to_string(&error)))?;
        let payload = value
            .as_string()
            .ok_or_else(|| err_msg("bootstrap payload JSON bridge returned a non-string value"))?;
        return serde_json::from_str::<BootstrapPayload>(&payload).map_err(err_msg);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let response =
            send_json_request(Method::GET, app_url("x/bootstrap"), Option::<&()>::None).await?;

        if !response.status().is_success() {
            return Err(std::io::Error::other(format!(
                "bootstrap request failed: {}",
                response.status()
            )));
        }

        response.json::<BootstrapPayload>().await.map_err(err_msg)
    }
}

pub async fn create_thread() -> Result<UiThread, std::io::Error> {
    let response = send_json_request(
        Method::POST,
        app_url("threads"),
        Some(&serde_json::json!({
            "metadata": {
                "title": "New Thread"
            }
        })),
    )
    .await?;

    if !response.status().is_success() {
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

pub async fn delete_thread(thread_id: &str) -> Result<(), std::io::Error> {
    let response = send_json_request(
        Method::DELETE,
        app_url(&format!("threads/{thread_id}")),
        Option::<&()>::None,
    )
    .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "delete thread failed: {}",
            response.status()
        )))
    }
}

pub async fn get_api_key(provider: &str) -> Result<String, std::io::Error> {
    let response = send_json_request(
        Method::GET,
        store_item_url(&["config", "api-keys"], provider),
        Option::<&()>::None,
    )
    .await?;

    if !response.status().is_success() {
        return Err(std::io::Error::other(format!(
            "get api key failed: {}",
            response.status()
        )));
    }

    let payload = response.json::<ItemResponse>().await.map_err(err_msg)?;
    Ok(payload.value.as_str().unwrap_or_default().to_string())
}

pub async fn set_api_key(provider: &str, value: &str) -> Result<(), std::io::Error> {
    let response = send_json_request(
        Method::PUT,
        app_url("store/items"),
        Some(&serde_json::json!({
            "namespace": ["config", "api-keys"],
            "key": provider,
            "value": value,
        })),
    )
    .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "set api key failed: {}",
            response.status()
        )))
    }
}

pub async fn delete_api_key(provider: &str) -> Result<(), std::io::Error> {
    let response = send_json_request(
        Method::DELETE,
        app_url("store/items"),
        Some(&serde_json::json!({
            "namespace": ["config", "api-keys"],
            "key": provider,
        })),
    )
    .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "delete api key failed: {}",
            response.status()
        )))
    }
}

#[allow(dead_code)]
pub async fn list_agent_endpoints() -> Result<Vec<AgentEndpoint>, std::io::Error> {
    let response = send_json_request(
        Method::POST,
        app_url("store/items/search"),
        Some(&serde_json::json!({
            "namespace_prefix": ["config", "agent-endpoints"],
            "limit": 200,
            "offset": 0,
        })),
    )
    .await?;

    if !response.status().is_success() {
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

pub async fn set_agent_endpoint(endpoint: &AgentEndpoint) -> Result<(), std::io::Error> {
    let response = send_json_request(
        Method::PUT,
        app_url("store/items"),
        Some(&serde_json::json!({
            "namespace": ["config", "agent-endpoints"],
            "key": endpoint.id,
            "value": endpoint,
        })),
    )
    .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "set agent endpoint failed: {}",
            response.status()
        )))
    }
}

pub async fn delete_agent_endpoint(id: &str) -> Result<(), std::io::Error> {
    let response = send_json_request(
        Method::DELETE,
        app_url("store/items"),
        Some(&serde_json::json!({
            "namespace": ["config", "agent-endpoints"],
            "key": id,
        })),
    )
    .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "delete agent endpoint failed: {}",
            response.status()
        )))
    }
}

pub async fn set_agent_rail_style(style: &str) -> Result<(), std::io::Error> {
    let response = send_json_request(
        Method::PUT,
        app_url("store/items"),
        Some(&serde_json::json!({
            "namespace": ["config", "agent-rail"],
            "key": "dicebear-style",
            "value": {
                "style": style,
            },
        })),
    )
    .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "set agent rail style failed: {}",
            response.status()
        )))
    }
}

pub async fn list_providers_with_keys() -> Result<Vec<Provider>, std::io::Error> {
    let response =
        send_json_request(Method::GET, app_url("x/providers"), Option::<&()>::None).await?;

    if !response.status().is_success() {
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

pub async fn set_default_model(model_id: &str) -> Result<(), std::io::Error> {
    let response = send_json_request(
        Method::PUT,
        app_url("store/items"),
        Some(&serde_json::json!({
            "namespace": ["config"],
            "key": "default_model",
            "value": {
                "model_id": model_id,
            },
        })),
    )
    .await?;

    if response.status().is_success() {
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
pub fn subscribe_browser_inference_status<F>(
    on_status: F,
) -> Result<BrowserInferenceStatusSubscription, std::io::Error>
where
    F: FnMut(BrowserInferenceStatus) + 'static,
{
    let callback = Rc::new(RefCell::new(on_status));
    let closure = {
        let callback = callback.clone();
        Closure::wrap(Box::new(move |value: JsValue| {
            if let Ok(status) = serde_wasm_bindgen::from_value::<BrowserInferenceStatus>(value) {
                callback.borrow_mut()(status);
            }
        }) as Box<dyn FnMut(JsValue)>)
    };

    let handle = js_start_browser_inference_status_stream(&closure)
        .map_err(|error| std::io::Error::other(js_value_to_string(&error)))?;

    Ok(BrowserInferenceStatusSubscription {
        callback: closure,
        handle,
    })
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn subscribe_browser_inference_status<F>(
    _: F,
) -> Result<BrowserInferenceStatusSubscription, std::io::Error>
where
    F: FnMut(BrowserInferenceStatus) + 'static,
{
    Err(unavailable())
}

pub async fn list_workspace_files(workspace: &str) -> Result<Vec<super::FileInfo>, std::io::Error> {
    let encoded = urlencoding::encode(workspace).into_owned();
    let response = send_json_request(
        Method::GET,
        app_url(&format!("x/files?workspace={encoded}")),
        Option::<&()>::None,
    )
    .await?;

    if !response.status().is_success() {
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
