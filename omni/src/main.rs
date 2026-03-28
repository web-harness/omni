use std::rc::Rc;

use dioxus::prelude::*;

mod components;
mod lib;
mod routes;

use components::{
    AgentsSection, Button, ButtonVariant, ChatContainer, Dialog, FilesSection, Input, KanbanView,
    TasksSection, ThreadSidebar,
};
use lib::{AppState, DataProvider, MockDataProvider, Theme};
use routes::Route;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const OMNI_DOCK_JS: Asset = asset!("/public/omni-dock.js");
const OMNI_POPPER_JS: Asset = asset!("/public/omni-popper.js");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let provider: Rc<dyn DataProvider> = Rc::new(MockDataProvider::new());
    let state = AppState::bootstrap(provider.clone());

    use_context_provider(|| provider);
    use_context_provider(|| Signal::new(state));

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link { rel: "preconnect", href: "https://fonts.gstatic.com", crossorigin: "" }
        document::Link { rel: "stylesheet", href: "https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&display=swap" }
        document::Script { src: OMNI_DOCK_JS }
        document::Script { src: OMNI_POPPER_JS }

        Router::<Route> {}
    }
}

#[component]
pub fn AppLayout() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    let thread_id = state.read().active_thread_id.clone().unwrap_or_default();
    let active_panel = "chat";

    let panel_config = r#"[
      {"id":"sidebar","slot":"sidebar","title":"Threads","position":{"direction":"left"}},
      {"id":"chat","slot":"chat","title":"Chat","position":{"referencePanel":"sidebar","direction":"right"}},
      {"id":"tasks","slot":"tasks","title":"Tasks","position":{"referencePanel":"chat","direction":"right"}},
      {"id":"files","slot":"files","title":"Files","position":{"referencePanel":"tasks","direction":"below"}},
      {"id":"agents","slot":"agents","title":"Agents","position":{"referencePanel":"files","direction":"below"}}
    ]"#;

    let theme = if state.read().theme == Theme::Light {
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
                div { slot: "sidebar", class: "h-full w-full overflow-hidden", ThreadSidebar {} }
                div { slot: "chat", class: "h-full w-full overflow-hidden",
                    if state.read().show_kanban {
                        KanbanView {}
                    } else {
                        ChatContainer { thread_id }
                    }
                }
                div { slot: "tasks", class: "h-full w-full overflow-auto", TasksSection {} }
                div { slot: "files", class: "h-full w-full overflow-auto", FilesSection {} }
                div { slot: "agents", class: "h-full w-full overflow-auto", AgentsSection {} }
                for path in state.read().open_tabs.clone().into_iter().filter(|p| p != "chat") {
                    div {
                        slot: path.clone(),
                        key: "{path}",
                        class: "h-full w-full overflow-hidden",
                        components::FileViewer { path: path.clone() }
                    }
                }
            }

            // Hidden outlet keeps URL routing alive for state sync
            div { class: "hidden", Outlet::<Route> {} }

            Dialog {
                open: state.read().settings_open,
                on_close: move |_| state.write().settings_open = false,
                h3 { class: "text-sm font-semibold", "Settings" }
                p { class: "mt-2 text-xs text-muted-foreground", "Workspace defaults, model preferences, and visual options." }
                div { class: "mt-3 grid gap-2",
                    div { class: "rounded-sm border border-border bg-background p-2 text-xs", "Theme: Tactical Dark" }
                    div { class: "rounded-sm border border-border bg-background p-2 text-xs", "Font Size: 12px" }
                    div { class: "rounded-sm border border-border bg-background p-2 text-xs", "Current Model: {state.read().selected_model}" }
                }
                div { class: "mt-3",
                    Button {
                        onclick: move |_| state.write().settings_open = false,
                        "Close"
                    }
                }
            }

            Dialog {
                open: state.read().api_key_dialog_open,
                on_close: move |_| state.write().api_key_dialog_open = false,
                h3 { class: "text-sm font-semibold", "Provider API Key" }
                p { class: "mt-2 text-xs text-muted-foreground", "Mocked dialog for API key setup." }
                p { class: "mt-2 text-[11px]", "Provider: {state.read().api_key_provider:?}" }
                Input {
                    value: state.read().api_key_draft.clone(),
                    placeholder: "sk-...".to_string(),
                    oninput: move |evt: Event<FormData>| state.write().api_key_draft = evt.value(),
                }
                div { class: "mt-3 flex justify-end gap-2",
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            state.write().api_key_draft.clear();
                            state.write().api_key_dialog_open = false;
                        },
                        "Cancel"
                    }
                    Button {
                        onclick: move |_| {
                            state.write().api_key_dialog_open = false;
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
    let state = use_context::<Signal<AppState>>();
    let navigator = use_navigator();
    let first = state.read().threads.first().map(|t| t.id.clone());

    use_effect(move || {
        if let Some(id) = first.clone() {
            navigator.replace(Route::ThreadView { id });
        }
    });

    rsx! { div { class: "h-full w-full" } }
}

#[component]
pub fn ThreadView(id: String) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let id_clone = id.clone();
    use_effect(move || {
        let mut s = state.write();
        s.active_thread_id = Some(id_clone.clone());
        s.show_kanban = false;
    });
    rsx! { div {} }
}

#[component]
pub fn Board() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    use_effect(move || {
        state.write().show_kanban = true;
    });
    rsx! { div {} }
}

#[component]
pub fn Settings() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    rsx! {
        div { class: "h-full overflow-auto p-4",
            div { class: "mx-auto max-w-2xl space-y-4",
                h2 { class: "text-lg font-semibold", "Settings" }
                div { class: "rounded-sm border border-border bg-background-elevated p-4",
                    p { class: "text-sm", "Configure providers and visual preferences." }
                }
                button {
                    class: "rounded-sm border border-border px-3 py-2 text-xs",
                    onclick: move |_| state.write().settings_open = true,
                    "Open Settings Dialog"
                }
            }
        }
    }
}
