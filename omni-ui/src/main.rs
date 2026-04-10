#![allow(special_module_name)]

use dioxus::prelude::*;
use serde::Deserialize;
use serde_json::json;

#[cfg(target_arch = "wasm32")]
use gloo_events::EventListener;
#[cfg(target_arch = "wasm32")]
use std::cell::{Cell, RefCell};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

mod components;
mod lib;
mod routes;
#[cfg(feature = "desktop")]
mod server;

use components::{
    AgentRail, BackgroundTasksSection, Button, ButtonVariant, ChatContainer, Dialog, FilesSection,
    Input, KanbanView, TasksSection, ThreadSidebar,
};
use lib::{default_states, ModelState, Theme, ThreadState, UiState, WorkspaceState};
use routes::Route;

#[cfg(target_arch = "wasm32")]
const IFRAME_BOOTSTRAP_EVENT: &str = "omni-iframe-config";
#[cfg(target_arch = "wasm32")]
const IFRAME_READY_EVENT: &str = "omni-iframe-ready";

#[derive(Clone, Deserialize)]
struct IframeAgentConfig {
    #[serde(default)]
    name: String,
    url: String,
    #[serde(rename = "apiKey")]
    api_key: String,
}

#[derive(Clone, Default, Deserialize)]
struct IframeBootstrapConfig {
    #[serde(default)]
    theme: String,
    #[serde(default, rename = "dicebearStyle")]
    dicebear_style: String,
    #[serde(default)]
    agents: Vec<IframeAgentConfig>,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[derive(Deserialize)]
struct DesktopExecuteRequest {
    command: String,
    cwd: Option<String>,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[derive(serde::Serialize)]
struct DesktopExecuteResponse {
    output: String,
    #[serde(rename = "exitCode")]
    exit_code: i32,
    truncated: bool,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Default, Deserialize)]
struct IframeBootstrapEnvelope {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    payload: IframeBootstrapConfig,
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static IFRAME_BOOTSTRAP: RefCell<Option<IframeBootstrapConfig>> = RefCell::new(None);
    static IFRAME_BOOTSTRAP_LISTENER: RefCell<Option<EventListener>> = RefCell::new(None);
    static IFRAME_APP_STARTED: Cell<bool> = Cell::new(false);
    static IFRAME_RUNTIME_LISTENER: RefCell<Option<EventListener>> = RefCell::new(None);
}

fn iframe_agent_endpoints(configs: Vec<IframeAgentConfig>) -> Vec<lib::AgentEndpoint> {
    configs
        .into_iter()
        .filter_map(|agent| {
            let name = agent.name.trim().to_string();
            let url = agent.url.trim().to_string();
            if url.is_empty() {
                return None;
            }
            let bearer_token = agent.api_key.trim().to_string();
            Some(lib::AgentEndpoint {
                id: lib::agent_config_hash(&url, &bearer_token),
                name: if name.is_empty() {
                    lib::derive_agent_name(&url)
                } else {
                    name
                },
                url,
                bearer_token,
                removable: true,
            })
        })
        .collect()
}

fn iframe_theme(theme: &str) -> Option<Theme> {
    match theme.trim() {
        "" => None,
        theme if theme.eq_ignore_ascii_case("light") => Some(Theme::Light),
        _ => Some(Theme::Dark),
    }
}

fn iframe_dicebear_style(style: &str) -> Option<String> {
    let trimmed = style.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(lib::normalize_dicebear_style(trimmed))
    }
}

fn iframe_agent_endpoints_override(
    config: &IframeBootstrapConfig,
) -> Option<Vec<lib::AgentEndpoint>> {
    if config.agents.is_empty() {
        None
    } else {
        Some(iframe_agent_endpoints(config.agents.clone()))
    }
}

#[cfg(target_arch = "wasm32")]
fn apply_iframe_config(
    mut ui_state: Signal<UiState>,
    mut agent_endpoint_state: Signal<lib::AgentEndpointState>,
    config: &IframeBootstrapConfig,
) {
    if let Some(theme) = iframe_theme(&config.theme) {
        ui_state.write().theme = theme;
    }

    if let Some(dicebear_style) = iframe_dicebear_style(&config.dicebear_style) {
        let should_persist = agent_endpoint_state.read().dicebear_style != dicebear_style;
        agent_endpoint_state.write().dicebear_style = dicebear_style.clone();
        if should_persist {
            spawn(async move {
                let _ = lib::sw_api::set_agent_rail_style(&dicebear_style).await;
            });
        }
    }

    if let Some(endpoints) = iframe_agent_endpoints_override(config) {
        let mut state = agent_endpoint_state.write();
        let previous_active_agent_id = state.active_agent_id.clone();
        state.endpoints = lib::merge_agent_endpoints(endpoints);
        state.active_agent_id = previous_active_agent_id
            .filter(|id| state.endpoints.iter().any(|endpoint| endpoint.id == *id));
    }
}

#[cfg(target_arch = "wasm32")]
fn take_iframe_bootstrap() -> Option<IframeBootstrapConfig> {
    IFRAME_BOOTSTRAP.with(|bootstrap| bootstrap.borrow_mut().take())
}

#[cfg(not(target_arch = "wasm32"))]
fn take_iframe_bootstrap() -> Option<IframeBootstrapConfig> {
    None
}

#[cfg(target_arch = "wasm32")]
fn install_iframe_runtime_listener(
    ui_state: Signal<UiState>,
    agent_endpoint_state: Signal<lib::AgentEndpointState>,
) {
    IFRAME_RUNTIME_LISTENER.with(|listener_slot| {
        if listener_slot.borrow().is_some() {
            return;
        }

        let Some(window) = web_sys::window() else {
            return;
        };

        let listener = EventListener::new(&window, "message", move |event| {
            let Some(envelope) = event
                .dyn_ref::<web_sys::MessageEvent>()
                .and_then(|message: &web_sys::MessageEvent| message.data().as_string())
                .and_then(|raw| serde_json::from_str::<IframeBootstrapEnvelope>(&raw).ok())
            else {
                return;
            };

            if envelope.kind != IFRAME_BOOTSTRAP_EVENT {
                return;
            }

            apply_iframe_config(ui_state, agent_endpoint_state, &envelope.payload);
        });

        listener_slot.replace(Some(listener));
    });
}

// The embedded iframe bootstraps app config through window messages before Dioxus mounts; this is the one approved web_sys/gloo-events exception.
#[cfg(target_arch = "wasm32")]
fn wait_for_iframe_bootstrap() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let is_embedded = window
        .parent()
        .ok()
        .flatten()
        .is_some_and(|parent| !js_sys::Object::is(parent.as_ref(), window.as_ref()));
    if !is_embedded {
        return false;
    }

    IFRAME_BOOTSTRAP_LISTENER.with(|listener_slot| {
        let listener = EventListener::new(&window, "message", move |event| {
            let Some(envelope) = event
                .dyn_ref::<web_sys::MessageEvent>()
                .and_then(|message: &web_sys::MessageEvent| message.data().as_string())
                .and_then(|raw| serde_json::from_str::<IframeBootstrapEnvelope>(&raw).ok())
            else {
                return;
            };

            if envelope.kind != IFRAME_BOOTSTRAP_EVENT {
                return;
            }

            let already_started = IFRAME_APP_STARTED.with(|started| {
                if started.get() {
                    true
                } else {
                    started.set(true);
                    false
                }
            });
            if already_started {
                return;
            }

            IFRAME_BOOTSTRAP.with(|bootstrap| {
                bootstrap.replace(Some(envelope.payload));
            });
            dioxus::launch(App);
        });
        listener_slot.replace(Some(listener));
    });

    if let Ok(Some(parent)) = window.parent() {
        let ready = serde_json::json!({ "type": IFRAME_READY_EVENT }).to_string();
        let _ = parent.post_message(&wasm_bindgen::JsValue::from_str(&ready), "*");
    }

    true
}

#[cfg(target_arch = "wasm32")]
fn provider_prefix(provider: &lib::ProviderId) -> &'static str {
    match provider {
        lib::ProviderId::Anthropic => "anthropic",
        lib::ProviderId::OpenAI => "openai",
        lib::ProviderId::Google => "google",
        lib::ProviderId::Ollama => "ollama",
        lib::ProviderId::Browser => "browser",
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    if wait_for_iframe_bootstrap() {
        return;
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    start_desktop_server();

    dioxus::launch(App);
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn start_desktop_server() {
    let (port_tx, port_rx) = std::sync::mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("omni-desktop-server".to_string())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to build desktop runtime");
            runtime.block_on(async {
                let app = axum::Router::new()
                    .merge(crate::server::store_api::router())
                    .route("/x/execute", axum::routing::post(desktop_execute))
                    .fallback(axum::routing::any(crate::server::assets::serve))
                    .layer(tower_http::cors::CorsLayer::permissive());
                let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
                    .await
                    .expect("failed to bind desktop api server");
                let port = listener
                    .local_addr()
                    .expect("failed to read desktop api socket address")
                    .port();
                port_tx
                    .send(port)
                    .expect("failed to publish desktop api port");
                axum::serve(listener, app)
                    .await
                    .expect("desktop api server exited unexpectedly");
            });
        })
        .expect("failed to spawn desktop api server");
    let port = port_rx
        .recv()
        .expect("failed to receive bound desktop api port");
    crate::lib::utils::set_desktop_api_port(port);
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
async fn desktop_execute(
    axum::Json(body): axum::Json<DesktopExecuteRequest>,
) -> Result<axum::Json<DesktopExecuteResponse>, crate::server::store_api::ApiError> {
    let (output, exit_code, truncated) = omni_rt::bashkit::execute_native(body.command, body.cwd)
        .await
        .map_err(crate::server::store_api::io_error)?;
    Ok(axum::Json(DesktopExecuteResponse {
        output,
        exit_code,
        truncated,
    }))
}

#[component]
fn App() -> Element {
    let (threads, chat, tasks, workspace, model, mut ui, background_tasks, mut agent_endpoints) =
        default_states();
    let iframe_config = take_iframe_bootstrap();
    let iframe_agent_endpoints_override = iframe_config
        .as_ref()
        .and_then(iframe_agent_endpoints_override);
    let iframe_dicebear_style_override = iframe_config
        .as_ref()
        .and_then(|config| iframe_dicebear_style(&config.dicebear_style));
    if let Some(iframe_config) = iframe_config {
        if let Some(theme) = iframe_theme(&iframe_config.theme) {
            ui.theme = theme;
        }
        if let Some(dicebear_style) = iframe_dicebear_style_override.clone() {
            agent_endpoints.dicebear_style = dicebear_style;
        }
        if let Some(endpoints) = iframe_agent_endpoints_override.clone() {
            agent_endpoints.endpoints = lib::merge_agent_endpoints(endpoints);
        }
    }
    #[cfg(target_arch = "wasm32")]
    let favicon_url = asset!("/assets/favicon.ico");
    #[cfg(not(target_arch = "wasm32"))]
    let favicon_url = lib::utils::api_url("assets/favicon.ico");

    #[cfg(target_arch = "wasm32")]
    let main_css_url = asset!("/assets/main.css");
    #[cfg(not(target_arch = "wasm32"))]
    let main_css_url = lib::utils::api_url("assets/main.css");

    #[cfg(target_arch = "wasm32")]
    let tailwind_css_url = asset!("/assets/tailwind.css");
    #[cfg(not(target_arch = "wasm32"))]
    let tailwind_css_url = lib::utils::api_url("assets/tailwind.css");

    #[cfg(target_arch = "wasm32")]
    let font_regular_url = asset!("/assets/fonts/JetBrainsMono-Regular.woff2");
    #[cfg(not(target_arch = "wasm32"))]
    let font_regular_url = lib::utils::api_url("assets/fonts/JetBrainsMono-Regular.woff2");

    #[cfg(target_arch = "wasm32")]
    let font_medium_url = asset!("/assets/fonts/JetBrainsMono-Medium.woff2");
    #[cfg(not(target_arch = "wasm32"))]
    let font_medium_url = lib::utils::api_url("assets/fonts/JetBrainsMono-Medium.woff2");

    #[cfg(target_arch = "wasm32")]
    let font_semibold_url = asset!("/assets/fonts/JetBrainsMono-SemiBold.woff2");
    #[cfg(not(target_arch = "wasm32"))]
    let font_semibold_url = lib::utils::api_url("assets/fonts/JetBrainsMono-SemiBold.woff2");

    #[cfg(target_arch = "wasm32")]
    let font_bold_url = asset!("/assets/fonts/JetBrainsMono-Bold.woff2");
    #[cfg(not(target_arch = "wasm32"))]
    let font_bold_url = lib::utils::api_url("assets/fonts/JetBrainsMono-Bold.woff2");
    let dock_url = lib::utils::api_url("omni-dock.js");
    let dicebear_url = lib::utils::api_url("omni-dicebear.js");
    let monaco_url = lib::utils::api_url("omni-monaco.js");
    let marked_url = lib::utils::api_url("omni-marked.js");
    let sheetjs_url = lib::utils::api_url("omni-sheetjs.js");
    let docxjs_url = lib::utils::api_url("omni-docxjs.js");
    let pdfjs_worker_url = lib::utils::api_url("omni-pdfjs.worker.js");
    let pdfjs_url = lib::utils::api_url("omni-pdfjs.js");
    let pptx_renderer_url = lib::utils::api_url("omni-pptx-renderer.js");
    let plyr_url = lib::utils::api_url("omni-plyr.js");
    let pretext_url = lib::utils::api_url("omni-pretext.js");
    let inference_url = lib::utils::api_url("omni-inference.js");
    let inference_register_url = lib::utils::api_url("omni-inference-register.js");
    let sw_url = lib::utils::api_url("omni-sw.js");
    let sw_register_url = lib::utils::api_url("omni-sw-register.js");

    let thread_signal = use_context_provider(|| Signal::new(threads));
    let chat_signal = use_context_provider(|| Signal::new(chat));
    let tasks_signal = use_context_provider(|| Signal::new(tasks));
    let workspace_signal = use_context_provider(|| Signal::new(workspace));
    let model_signal = use_context_provider(|| Signal::new(model));
    let _ui_signal = use_context_provider(|| Signal::new(ui));
    let background_task_signal = use_context_provider(|| Signal::new(background_tasks));
    let agent_endpoint_signal = use_context_provider(|| Signal::new(agent_endpoints));
    let _floating_dock_signal =
        use_context_provider(|| Signal::new(lib::FloatingDockState::default()));
    let _add_agent_draft_signal =
        use_context_provider(|| Signal::new(lib::AddAgentDraft::default()));

    #[cfg(target_arch = "wasm32")]
    install_iframe_runtime_listener(_ui_signal, agent_endpoint_signal);

    use_future(move || {
        let iframe_agent_endpoints_override = iframe_agent_endpoints_override.clone();
        let iframe_dicebear_style_override = iframe_dicebear_style_override.clone();
        async move {
            lib::async_init(
                thread_signal,
                chat_signal,
                tasks_signal,
                workspace_signal,
                model_signal,
                background_task_signal,
                agent_endpoint_signal,
                iframe_agent_endpoints_override,
                iframe_dicebear_style_override.clone(),
            )
            .await;

            if let Some(dicebear_style) = iframe_dicebear_style_override {
                let _ = lib::sw_api::set_agent_rail_style(&dicebear_style).await;
            }
        }
    });

    rsx! {
        document::Link { rel: "icon", href: favicon_url }
        document::Link { rel: "stylesheet", href: main_css_url }
        document::Link { rel: "stylesheet", href: tailwind_css_url }
        document::Style {
            "@font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:400;font-display:swap;src:url('{font_regular_url}') format('woff2')}}
            @font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:500;font-display:swap;src:url('{font_medium_url}') format('woff2')}}
            @font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:600;font-display:swap;src:url('{font_semibold_url}') format('woff2')}}
            @font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:700;font-display:swap;src:url('{font_bold_url}') format('woff2')}}"
        }
        document::Script { src: dock_url, r#type: "module", defer: true }
        document::Script { src: dicebear_url, r#type: "module", defer: true }
        document::Script { src: monaco_url, r#type: "module", defer: true }
        document::Script { src: marked_url, r#type: "module", defer: true }
        document::Script { src: sheetjs_url, r#type: "module", defer: true }
        document::Script { src: docxjs_url, r#type: "module", defer: true }
        document::Meta { name: "omni-pdfjs-worker", content: pdfjs_worker_url }
        document::Script { src: pdfjs_url, r#type: "module", defer: true }
        document::Script { src: pptx_renderer_url, r#type: "module", defer: true }
        document::Script { src: plyr_url, r#type: "module", defer: true }
        document::Script { src: pretext_url, r#type: "module", defer: true }
        document::Meta { name: "omni-inference-url", content: inference_url }
        document::Script { src: inference_register_url, r#type: "module", defer: true }
        document::Meta { name: "omni-sw-url", content: sw_url }
        document::Script { src: sw_register_url, r#type: "module", defer: true }

        Router::<Route> {}
    }
}

#[component]
fn AgentTooltipOverlay(panel: lib::FloatingPanel, label: String) -> Element {
    let floating_state = use_context::<Signal<lib::FloatingDockState>>();
    let (ox, oy) = floating_state.read().dock_origin;
    let left = panel.x + ox;
    let top = panel.y + oy;
    rsx! {
        div {
            class: "pointer-events-none fixed z-[200] flex items-center px-2 py-1 rounded-sm border border-border bg-background-elevated text-[10px] text-foreground shadow-xl whitespace-nowrap",
            style: "left: {left}px; top: {top}px;",
            "{label}"
        }
    }
}

#[component]
fn AgentCloseBadgeOverlay(panel: lib::FloatingPanel, agent_id: String) -> Element {
    let mut floating_dock = use_context::<Signal<lib::FloatingDockState>>();
    let mut agent_endpoint_state = use_context::<Signal<lib::AgentEndpointState>>();
    let floating_state = use_context::<Signal<lib::FloatingDockState>>();
    let (ox, oy) = floating_state.read().dock_origin;
    let left = panel.x + ox;
    let top = panel.y + oy;
    let panel_id = panel.id.clone();
    rsx! {
        div {
            class: "fixed z-[200] flex items-center justify-center",
            style: "left: {left}px; top: {top}px; width: {panel.width}px; height: {panel.height}px;",
            button {
                class: "flex h-5 w-5 items-center justify-center rounded-full border border-status-critical/60 bg-status-critical text-white shadow-lg",
                onclick: move |_| {
                    agent_endpoint_state.write().remove(&agent_id);
                    floating_dock.write().close(&panel_id);
                    #[cfg(target_arch = "wasm32")]
                    {
                        let id = agent_id.clone();
                        spawn(async move {
                            let _ = crate::lib::sw_api::delete_agent_endpoint(&id).await;
                        });
                    }
                },
                dioxus_free_icons::Icon { width: 10, height: 10, icon: dioxus_free_icons::icons::ld_icons::LdMinus }
            }
        }
    }
}

#[component]
fn FloatingPanelSlot(panel: lib::FloatingPanel) -> Element {
    let mut floating_dock = use_context::<Signal<lib::FloatingDockState>>();
    let mut agent_endpoint_state = use_context::<Signal<lib::AgentEndpointState>>();
    let mut add_agent_draft = use_context::<Signal<lib::AddAgentDraft>>();
    let panel_id = panel.id.clone();

    match panel.kind {
        lib::FloatingPanelKind::AgentTooltip { .. }
        | lib::FloatingPanelKind::AgentCloseBadge { .. } => {
            rsx! { Fragment {} }
        }
        lib::FloatingPanelKind::AddAgentPopover => {
            let panel_id_close = panel_id.clone();
            rsx! {
                div {
                    slot: "{panel_id}",
                    class: "space-y-3 p-2",
                    div { class: "space-y-1",
                        omni-text { "data-text": "Add Agent", "data-strategy": "none", "data-max-lines": "1", class: "text-xs font-semibold" }
                        omni-text { "data-text": "Connect a LangGraph endpoint for direct chat routing.", "data-strategy": "none", "data-max-lines": "2", class: "text-[10px] text-muted-foreground" }
                    }
                    div { class: "space-y-2",
                        Input {
                            value: add_agent_draft.read().name.clone(),
                            placeholder: "Agent name".to_string(),
                            oninput: move |evt: Event<FormData>| add_agent_draft.write().name = evt.value(),
                        }
                        Input {
                            value: add_agent_draft.read().url.clone(),
                            placeholder: "https://agent.example.com/api".to_string(),
                            oninput: move |evt: Event<FormData>| add_agent_draft.write().url = evt.value(),
                        }
                        Input {
                            value: add_agent_draft.read().token.clone(),
                            placeholder: "Bearer token".to_string(),
                            oninput: move |evt: Event<FormData>| add_agent_draft.write().token = evt.value(),
                        }
                    }
                    div { class: "flex justify-end",
                        Button {
                            onclick: move |_| {
                                let draft = add_agent_draft.read();
                                let name = draft.name.trim().to_string();
                                let url = draft.url.trim().to_string();
                                let bearer_token = draft.token.trim().to_string();
                                drop(draft);
                                if url.is_empty() || bearer_token.is_empty() {
                                    return;
                                }
                                let endpoint = lib::AgentEndpoint {
                                    id: lib::agent_config_hash(&url, &bearer_token),
                                    name: if name.is_empty() { lib::derive_agent_name(&url) } else { name },
                                    url,
                                    bearer_token,
                                    removable: true,
                                };
                                agent_endpoint_state.write().upsert(endpoint.clone());
                                *add_agent_draft.write() = lib::AddAgentDraft::default();
                                floating_dock.write().close(&panel_id_close);
                                #[cfg(target_arch = "wasm32")]
                                spawn(async move {
                                    let _ = crate::lib::sw_api::set_agent_endpoint(&endpoint).await;
                                });
                            },
                            omni-text { "data-text": "Add", "data-strategy": "none", "data-max-lines": "1" }
                        }
                    }
                }
            }
        }
        lib::FloatingPanelKind::ModelPicker { .. }
        | lib::FloatingPanelKind::WorkspacePicker { .. } => rsx! {
            div { slot: "{panel_id}" }
        },
    }
}

#[component]
pub fn AppLayout() -> Element {
    let thread_state = use_context::<Signal<ThreadState>>();
    let mut ui_state = use_context::<Signal<UiState>>();
    let mut workspace_state = use_context::<Signal<WorkspaceState>>();
    let model_state = use_context::<Signal<ModelState>>();
    let mut floating_dock = use_context::<Signal<lib::FloatingDockState>>();

    let thread_id = thread_state
        .read()
        .active_thread_id
        .clone()
        .unwrap_or_default();
    let active_panel = workspace_state.read().active_tab_for(&thread_id);

    let open_tabs = workspace_state.read().open_tabs_for(&thread_id);
    let mut panels = vec![
        json!({"id":"agent-rail","slot":"agent-rail","title":"Agents","position":{"direction":"left"},"hideHeader":true,"fixedWidth":48}),
        json!({"id":"sidebar","slot":"sidebar","title":"Threads","position":{"referencePanel":"agent-rail","direction":"right"}}),
        json!({"id":"chat","slot":"chat","title":"Chat","position":{"referencePanel":"sidebar","direction":"right"}}),
        json!({"id":"tasks","slot":"tasks","title":"Tasks","position":{"referencePanel":"chat","direction":"right"}}),
        json!({"id":"files","slot":"files","title":"Files","position":{"referencePanel":"tasks","direction":"below"}}),
        json!({"id":"bg-tasks","slot":"bg-tasks","title":"Background Tasks","position":{"referencePanel":"files","direction":"below"}}),
    ];
    for path in &open_tabs {
        if path.as_str() != "chat" {
            let title = path.rsplit('/').next().unwrap_or(path.as_str());
            panels.push(json!({
                "id": path,
                "slot": path,
                "title": title,
                "position": {"referencePanel": "chat", "direction": "within"},
            }));
        }
    }
    let panel_config = serde_json::to_string(&panels).unwrap_or_default();

    let floating_state = floating_dock.read();
    let mut floating_panels: Vec<serde_json::Value> = Vec::new();
    for panel in &floating_state.panels {
        floating_panels.push(json!({
            "id": panel.id,
            "slot": panel.id,
            "x": panel.x,
            "y": panel.y,
            "width": panel.width,
            "height": panel.height,
        }));
    }
    let floating_config = serde_json::to_string(&floating_panels).unwrap_or_default();

    let active_panels: Vec<lib::FloatingPanel> = floating_state.panels.clone();
    drop(floating_state);

    let theme = if ui_state.read().theme == Theme::Light {
        "light"
    } else {
        "dark"
    };

    rsx! {
        div {
            class: "h-screen w-screen overflow-hidden bg-background text-foreground",
            "data-theme": theme,
            omni-dock {
                class: "w-full h-screen",
                "data-panels": panel_config,
                "data-active-panel": active_panel.clone(),
                "data-proportions": "48px,20,60,20",
                "data-floating-panels": floating_config,
                onmounted: move |evt| async move {
                    if let Ok(cr) = evt.get_client_rect().await {
                        floating_dock.write().dock_origin = (cr.min_x(), cr.min_y());
                    }
                },
                input {
                    r#type: "hidden",
                    "data-dock-relay": "true",
                    oninput: move |evt: Event<FormData>| {
                        let panel_id = evt.value();
                        if panel_id.starts_with("floating:") {
                            let id = &panel_id["floating:".len()..];
                            floating_dock.write().close(id);
                        } else if !panel_id.is_empty() {
                            let tid = thread_state.read().active_thread_id.clone().unwrap_or_default();
                            workspace_state.write().open_tabs.entry(tid).or_default().retain(|x| x != &panel_id);
                        }
                    },
                },
                div { slot: "agent-rail", class: "h-full w-full overflow-hidden", AgentRail {} }
                div { slot: "sidebar", class: "h-full w-full overflow-hidden", ThreadSidebar {} }
                div { slot: "chat", class: "h-full w-full overflow-hidden",
                    if thread_state.read().show_kanban {
                        KanbanView {}
                    } else {
                        ChatContainer { thread_id: thread_id.clone() }
                    }
                }
                div { slot: "tasks", class: "h-full w-full overflow-auto", TasksSection {} }
                div { slot: "files", class: "h-full w-full overflow-auto", FilesSection {} }
                div { slot: "bg-tasks", class: "h-full w-full overflow-auto", BackgroundTasksSection {} }
                for panel in active_panels.iter() {
                    FloatingPanelSlot {
                        key: "{panel.id}",
                        panel: panel.clone(),
                    }
                }
                for path in open_tabs.iter().filter(|p| p.as_str() != "chat") {
                    {
                        let gen = workspace_state.read().tab_generation.get(path).copied().unwrap_or(0);
                        rsx! {
                            div {
                                slot: path.clone(),
                                key: "{path}-{gen}",
                                class: "h-full w-full overflow-hidden",
                                components::FileViewer { path: path.clone(), thread_id: thread_id.clone() }
                            }
                        }
                    }
                }
            }

            for panel in active_panels.iter() {
                match &panel.kind {
                    lib::FloatingPanelKind::AgentTooltip { label } => rsx! {
                        AgentTooltipOverlay {
                            key: "{panel.id}",
                            panel: panel.clone(),
                            label: label.clone(),
                        }
                    },
                    lib::FloatingPanelKind::AgentCloseBadge { agent_id } => rsx! {
                        AgentCloseBadgeOverlay {
                            key: "{panel.id}",
                            panel: panel.clone(),
                            agent_id: agent_id.clone(),
                        }
                    },
                    _ => rsx! { Fragment {} },
                }
            }

            div { class: "hidden", Outlet::<Route> {} }

            Dialog {
                open: ui_state.read().settings_open,
                on_close: move |_| ui_state.write().settings_open = false,
                h3 { class: "text-sm font-semibold", "Settings" }
                p { class: "mt-2 text-xs text-muted-foreground", "Workspace defaults, model preferences, and visual options." }
                div { class: "mt-3 grid gap-2",
                    div { class: "rounded-sm border border-border bg-background p-2 text-xs", "Theme: Tactical Dark" }
                    div { class: "rounded-sm border border-border bg-background p-2 text-xs", "Font Size: 12px" }
                    div { class: "rounded-sm border border-border bg-background p-2 text-xs", "Current Model: {model_state.read().selected_model_for(&thread_id)}" }
                }
                div { class: "mt-3",
                    Button {
                        onclick: move |_| ui_state.write().settings_open = false,
                        "Close"
                    }
                }
            }

            Dialog {
                open: ui_state.read().api_key_dialog_open,
                on_close: move |_| ui_state.write().api_key_dialog_open = false,
                h3 { class: "text-sm font-semibold", "Provider API Key" }
                p { class: "mt-2 text-xs text-muted-foreground", "API keys are stored in /home/config/.env." }
                p { class: "mt-2 text-[11px]", "Provider: {ui_state.read().api_key_provider:?}" }
                Input {
                    value: ui_state.read().api_key_draft.clone(),
                    placeholder: "sk-...".to_string(),
                    oninput: move |evt: Event<FormData>| ui_state.write().api_key_draft = evt.value(),
                }
                div { class: "mt-3 flex justify-end gap-2",
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            ui_state.write().api_key_draft.clear();
                            ui_state.write().api_key_dialog_open = false;
                        },
                        "Cancel"
                    }
                    Button {
                        onclick: move |_| {
                            #[cfg(target_arch = "wasm32")]
                            {
                                let provider = ui_state.read().api_key_provider.clone();
                                let value = ui_state.read().api_key_draft.trim().to_string();
                                let mut model_state_for_save = model_state;
                                spawn(async move {
                                    let prefix = provider_prefix(&provider);
                                    if value.is_empty() {
                                        let _ = lib::sw_api::delete_api_key(prefix).await;
                                    } else {
                                        let _ = lib::sw_api::set_api_key(prefix, &value).await;
                                    }

                                    if let Ok(providers) = lib::sw_api::list_providers_with_keys().await {
                                        model_state_for_save.write().providers = providers;
                                    }
                                });
                            }

                            ui_state.write().api_key_draft.clear();
                            ui_state.write().api_key_dialog_open = false;
                        },
                        "Save"
                    }
                }
            }
        }
    }
}

#[component]
pub fn Home() -> Element {
    let thread_state = use_context::<Signal<ThreadState>>();
    let navigator = use_navigator();

    use_effect(move || {
        let first = thread_state.read().threads.first().map(|t| t.id.clone());
        if let Some(id) = first {
            navigator.replace(Route::ThreadView { id });
        }
    });

    rsx! { div { class: "h-full w-full" } }
}

#[component]
pub fn ThreadView(id: String) -> Element {
    let mut thread_state = use_context::<Signal<ThreadState>>();
    let navigator = use_navigator();
    let id_clone = id.clone();
    use_effect(move || {
        let snapshot = thread_state.read();
        let exists = snapshot.threads.iter().any(|t| t.id == id_clone);
        let current = snapshot.active_thread_id.clone();
        let was_kanban = snapshot.show_kanban;
        let first = snapshot.threads.first().map(|t| t.id.clone());
        drop(snapshot);

        if exists {
            if current.as_deref() != Some(&id_clone) || was_kanban {
                let mut s = thread_state.write();
                s.active_thread_id = Some(id_clone.clone());
                s.show_kanban = false;
            }
            return;
        }

        if let Some(valid_id) = first {
            {
                let mut s = thread_state.write();
                s.active_thread_id = Some(valid_id.clone());
                s.show_kanban = false;
            }
            navigator.replace(Route::ThreadView { id: valid_id });
        }
    });
    rsx! { div {} }
}

#[component]
pub fn Board() -> Element {
    let mut thread_state = use_context::<Signal<ThreadState>>();
    use_effect(move || {
        if !thread_state.read().show_kanban {
            thread_state.write().show_kanban = true;
        }
    });
    rsx! { div {} }
}

#[component]
pub fn Settings() -> Element {
    let mut ui_state = use_context::<Signal<UiState>>();

    rsx! {
        div { class: "h-full overflow-auto p-4",
            div { class: "mx-auto max-w-2xl space-y-4",
                h2 { class: "text-lg font-semibold", "Settings" }
                div { class: "rounded-sm border border-border bg-background-elevated p-4",
                    p { class: "text-sm", "Configure providers and visual preferences." }
                }
                button {
                    class: "rounded-sm border border-border px-3 py-2 text-xs",
                    onclick: move |_| ui_state.write().settings_open = true,
                    "Open Settings Dialog"
                }
            }
        }
    }
}
