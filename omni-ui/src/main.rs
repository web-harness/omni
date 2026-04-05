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

use components::{
    AgentRail, BackgroundTasksSection, Button, ButtonVariant, ChatContainer, Dialog, FilesSection,
    Input, KanbanView, TasksSection, ThreadSidebar,
};
use lib::{default_states, ModelState, Theme, ThreadState, UiState, WorkspaceState};
use routes::Route;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const FONT_REGULAR: Asset = asset!("/assets/fonts/JetBrainsMono-Regular.woff2");
const FONT_MEDIUM: Asset = asset!("/assets/fonts/JetBrainsMono-Medium.woff2");
const FONT_SEMIBOLD: Asset = asset!("/assets/fonts/JetBrainsMono-SemiBold.woff2");
const FONT_BOLD: Asset = asset!("/assets/fonts/JetBrainsMono-Bold.woff2");
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

    dioxus::launch(App);
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
    let dock_url = lib::utils::app_url("omni-dock.js");
    let dicebear_url = lib::utils::app_url("omni-dicebear.js");
    let popper_url = lib::utils::app_url("omni-popper.js");
    let monaco_url = lib::utils::app_url("omni-monaco.js");
    let marked_url = lib::utils::app_url("omni-marked.js");
    let sheetjs_url = lib::utils::app_url("omni-sheetjs.js");
    let docxjs_url = lib::utils::app_url("omni-docxjs.js");
    let pdfjs_worker_url = lib::utils::app_url("omni-pdfjs.worker.js");
    let pdfjs_url = lib::utils::app_url("omni-pdfjs.js");
    let pptx_renderer_url = lib::utils::app_url("omni-pptx-renderer.js");
    let plyr_url = lib::utils::app_url("omni-plyr.js");
    let pretext_url = lib::utils::app_url("omni-pretext.js");
    let inference_url = lib::utils::app_url("omni-inference.js");
    let inference_register_url = lib::utils::app_url("omni-inference-register.js");
    let sw_url = lib::utils::app_url("omni-sw.js");
    let sw_register_url = lib::utils::app_url("omni-sw-register.js");

    #[cfg(target_arch = "wasm32")]
    let thread_signal = use_context_provider(|| Signal::new(threads));
    #[cfg(not(target_arch = "wasm32"))]
    use_context_provider(|| Signal::new(threads));

    #[cfg(target_arch = "wasm32")]
    let chat_signal = use_context_provider(|| Signal::new(chat));
    #[cfg(not(target_arch = "wasm32"))]
    use_context_provider(|| Signal::new(chat));

    #[cfg(target_arch = "wasm32")]
    let tasks_signal = use_context_provider(|| Signal::new(tasks));
    #[cfg(not(target_arch = "wasm32"))]
    use_context_provider(|| Signal::new(tasks));

    #[cfg(target_arch = "wasm32")]
    let workspace_signal = use_context_provider(|| Signal::new(workspace));
    #[cfg(not(target_arch = "wasm32"))]
    use_context_provider(|| Signal::new(workspace));

    #[cfg(target_arch = "wasm32")]
    let model_signal = use_context_provider(|| Signal::new(model));
    #[cfg(not(target_arch = "wasm32"))]
    use_context_provider(|| Signal::new(model));

    #[cfg(target_arch = "wasm32")]
    let ui_signal = use_context_provider(|| Signal::new(ui));
    #[cfg(not(target_arch = "wasm32"))]
    use_context_provider(|| Signal::new(ui));
    #[cfg(target_arch = "wasm32")]
    let background_task_signal = use_context_provider(|| Signal::new(background_tasks));
    #[cfg(not(target_arch = "wasm32"))]
    use_context_provider(|| Signal::new(background_tasks));

    #[cfg(target_arch = "wasm32")]
    let agent_endpoint_signal = use_context_provider(|| Signal::new(agent_endpoints));
    #[cfg(not(target_arch = "wasm32"))]
    use_context_provider(|| Signal::new(agent_endpoints));

    #[cfg(target_arch = "wasm32")]
    install_iframe_runtime_listener(ui_signal, agent_endpoint_signal);

    #[cfg(target_arch = "wasm32")]
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
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Style {
            "@font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:400;font-display:swap;src:url('{FONT_REGULAR}') format('woff2')}}
            @font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:500;font-display:swap;src:url('{FONT_MEDIUM}') format('woff2')}}
            @font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:600;font-display:swap;src:url('{FONT_SEMIBOLD}') format('woff2')}}
            @font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:700;font-display:swap;src:url('{FONT_BOLD}') format('woff2')}}"
        }
        document::Script { src: dock_url, r#type: "module", defer: true }
        document::Script { src: dicebear_url, r#type: "module", defer: true }
        document::Script { src: popper_url, r#type: "module", defer: true }
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
pub fn AppLayout() -> Element {
    let thread_state = use_context::<Signal<ThreadState>>();
    let mut ui_state = use_context::<Signal<UiState>>();
    let mut workspace_state = use_context::<Signal<WorkspaceState>>();
    let model_state = use_context::<Signal<ModelState>>();

    let thread_id = thread_state
        .read()
        .active_thread_id
        .clone()
        .unwrap_or_default();
    let active_panel = workspace_state.read().active_tab_for(&thread_id);

    let open_tabs = workspace_state.read().open_tabs_for(&thread_id);
    let mut panels = vec![
        json!({"id":"sidebar","slot":"sidebar","title":"Threads","position":{"direction":"left"}}),
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

    let theme = if ui_state.read().theme == Theme::Light {
        "light"
    } else {
        "dark"
    };

    rsx! {
        div {
            class: "h-screen w-screen overflow-hidden bg-background text-foreground flex",
            "data-theme": theme,
            AgentRail {}
            omni-dock {
                class: "min-w-0 flex-1 h-screen",
                "data-panels": panel_config,
                "data-active-panel": active_panel.clone(),
                "data-proportions": "20,60,20",
                input {
                    r#type: "hidden",
                    "data-dock-relay": "true",
                    oninput: move |evt: Event<FormData>| {
                        let panel_id = evt.value();
                        if !panel_id.is_empty() {
                            let tid = thread_state.read().active_thread_id.clone().unwrap_or_default();
                            workspace_state.write().open_tabs.entry(tid).or_default().retain(|x| x != &panel_id);
                        }
                    },
                },
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
    let first = thread_state.read().threads.first().map(|t| t.id.clone());

    use_effect(move || {
        if let Some(id) = first.clone() {
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
