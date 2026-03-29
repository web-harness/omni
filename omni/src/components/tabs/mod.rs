use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdBot, LdFileCode, LdFileJson, LdFileText, LdX};
use dioxus_free_icons::Icon;

use crate::components::chat::ChatContainer;
use crate::lib::{ThreadState, WorkspaceState};

#[component]
pub fn TabBar() -> Element {
    let workspace_state = use_context::<Signal<WorkspaceState>>();
    let thread_state = use_context::<Signal<ThreadState>>();
    let tid = thread_state
        .read()
        .active_thread_id
        .clone()
        .unwrap_or_default();
    let open_tabs = workspace_state.read().open_tabs_for(&tid);
    let active_tab = workspace_state.read().active_tab_for(&tid);

    rsx! {
        div { class: "flex items-center gap-1 border-b border-border bg-sidebar px-2 py-1",
            for tab in open_tabs {
                TabChip {
                    key: "{tab}",
                    tab: tab.clone(),
                    active: tab == active_tab,
                }
            }
        }
    }
}

#[component]
fn TabChip(tab: String, active: bool) -> Element {
    let mut workspace_state = use_context::<Signal<WorkspaceState>>();
    let thread_state = use_context::<Signal<ThreadState>>();
    let tid = thread_state
        .read()
        .active_thread_id
        .clone()
        .unwrap_or_default();
    let tab_for_select = tab.clone();
    let tab_for_close = tab.clone();
    let class = if active {
        "inline-flex items-center gap-2 rounded-t-sm px-3 py-1.5 text-[11px] transition bg-primary/15 text-primary border-b border-primary"
    } else {
        "inline-flex items-center gap-2 rounded-t-sm px-3 py-1.5 text-[11px] transition text-muted-foreground hover:bg-background-interactive"
    };

    rsx! {
        button {
            class: "{class}",
            onclick: move |_| { workspace_state.write().active_tab.insert(tid.clone(), tab_for_select.clone()); },
            if tab == "chat" {
                Icon { width: 13, height: 13, icon: LdBot }
                span { "Agent" }
            } else {
                FileIcon { path: tab.clone() }
                span { class: "max-w-[180px] truncate", "{tab}" }
                button {
                    class: "rounded p-0.5 hover:bg-background",
                    onclick: {
                        let tid2 = tid.clone();
                        move |evt: Event<MouseData>| {
                            evt.stop_propagation();
                            workspace_state.write().open_tabs.entry(tid2.clone()).or_default().retain(|x| x != &tab_for_close);
                            if workspace_state.read().active_tab_for(&tid2) == tab_for_close {
                                workspace_state.write().active_tab.insert(tid2.clone(), "chat".to_string());
                            }
                        }
                    },
                    Icon { width: 11, height: 11, icon: LdX }
                }
            }
        }
    }
}

#[component]
fn FileIcon(path: String) -> Element {
    if path.ends_with(".rs") || path.ends_with(".ts") || path.ends_with(".tsx") {
        return rsx! { Icon { width: 13, height: 13, icon: LdFileCode } };
    }
    if path.ends_with(".json") {
        return rsx! { Icon { width: 13, height: 13, icon: LdFileJson } };
    }
    rsx! { Icon { width: 13, height: 13, icon: LdFileText } }
}

#[component]
pub fn TabbedPanel(thread_id: String) -> Element {
    rsx! {
        ChatContainer { thread_id }
    }
}

#[component]
pub fn FileViewer(path: String) -> Element {
    rsx! {
        div { class: "h-full overflow-auto p-4",
            div { class: "mb-3 text-[11px] text-muted-foreground", "{path}" }
            pre {
                class: "rounded-sm border border-border bg-background p-3 font-mono text-[11px] leading-5 text-foreground",
                "// Mocked file preview\n",
                "fn main() -> ()\n",
                "    println!(\"hello\");\n",
                "end\n",
                "\n",
                "// path: {path}"
            }
        }
    }
}
