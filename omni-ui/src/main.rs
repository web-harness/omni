use std::rc::Rc;

use dioxus::prelude::*;
use serde_json::json;

mod components;
mod lib;
mod routes;

use components::{
    AgentsSection, Button, ButtonVariant, ChatContainer, Dialog, FilesSection, Input, KanbanView,
    TasksSection, ThreadSidebar,
};
use lib::{
    bootstrap, DataProvider, MockDataProvider, ModelState, Theme, ThreadState, UiState,
    WorkspaceState,
};
use routes::Route;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const FONT_REGULAR: Asset = asset!("/assets/fonts/JetBrainsMono-Regular.woff2");
const FONT_MEDIUM: Asset = asset!("/assets/fonts/JetBrainsMono-Medium.woff2");
const FONT_SEMIBOLD: Asset = asset!("/assets/fonts/JetBrainsMono-SemiBold.woff2");
const FONT_BOLD: Asset = asset!("/assets/fonts/JetBrainsMono-Bold.woff2");
const OMNI_DOCK_JS: Asset = asset!("/public/omni-dock.js");
const OMNI_POPPER_JS: Asset = asset!("/public/omni-popper.js");
const OMNI_MONACO_JS: Asset = asset!("/public/omni-monaco.js");
const OMNI_MDX_JS: Asset = asset!("/public/omni-mdx.js");
const OMNI_PDFJS_JS: Asset = asset!("/public/omni-pdfjs.js");
const OMNI_PDFJS_WORKER_JS: Asset = asset!("/public/omni-pdfjs.worker.js");
const OMNI_PLYR_JS: Asset = asset!("/public/omni-plyr.js");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let provider: Rc<dyn DataProvider> = Rc::new(MockDataProvider::new());
    let (threads, chat, tasks, workspace, model, ui, subagents) = bootstrap(provider.clone());

    use_context_provider(|| provider);
    use_context_provider(|| Signal::new(threads));
    use_context_provider(|| Signal::new(chat));
    use_context_provider(|| Signal::new(tasks));
    use_context_provider(|| Signal::new(workspace));
    use_context_provider(|| Signal::new(model));
    use_context_provider(|| Signal::new(ui));
    use_context_provider(|| Signal::new(subagents));

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
        document::Script { src: OMNI_DOCK_JS, r#type: "module", defer: true }
        document::Script { src: OMNI_POPPER_JS, r#type: "module", defer: true }
        document::Script { src: OMNI_MONACO_JS, r#type: "module", defer: true }
        document::Script { src: OMNI_MDX_JS, r#type: "module", defer: true }
        document::Meta { name: "omni-pdfjs-worker", content: "{OMNI_PDFJS_WORKER_JS}" }
        document::Script { src: OMNI_PDFJS_JS, r#type: "module", defer: true }
        document::Script { src: OMNI_PLYR_JS, r#type: "module", defer: true }

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
    let active_panel = "chat";

    let open_tabs = workspace_state.read().open_tabs_for(&thread_id);
    let mut panels = vec![
        json!({"id":"sidebar","slot":"sidebar","title":"Threads","position":{"direction":"left"}}),
        json!({"id":"chat","slot":"chat","title":"Chat","position":{"referencePanel":"sidebar","direction":"right"}}),
        json!({"id":"tasks","slot":"tasks","title":"Tasks","position":{"referencePanel":"chat","direction":"right"}}),
        json!({"id":"files","slot":"files","title":"Files","position":{"referencePanel":"tasks","direction":"below"}}),
        json!({"id":"agents","slot":"agents","title":"Agents","position":{"referencePanel":"files","direction":"below"}}),
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
            class: "h-screen w-screen overflow-hidden bg-background text-foreground",
            "data-theme": theme,
            omni-dock {
                class: "h-screen w-screen",
                "data-panels": panel_config,
                "data-active-panel": active_panel,
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
                div { slot: "agents", class: "h-full w-full overflow-auto", AgentsSection {} }
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
                p { class: "mt-2 text-xs text-muted-foreground", "Mocked dialog for API key setup." }
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
    let id_clone = id.clone();
    use_effect(move || {
        let current = thread_state.read().active_thread_id.clone();
        if current.as_deref() != Some(&id_clone) || thread_state.read().show_kanban {
            let mut s = thread_state.write();
            s.active_thread_id = Some(id_clone.clone());
            s.show_kanban = false;
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
