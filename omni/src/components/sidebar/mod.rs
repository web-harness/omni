use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdLayoutGrid, LdLoader, LdMessageSquare, LdPlus, LdTrash2, LdTriangleAlert,
};
use dioxus_free_icons::Icon;

use crate::lib::{AppState, ThreadStatus};
use crate::routes::Route;

#[component]
pub fn ThreadSidebar() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let navigator = use_navigator();

    rsx! {
        aside {
            class: "h-full w-full border-r border-border bg-sidebar flex flex-col overflow-hidden",
            div { class: "px-3 py-2 border-b border-border",
                button {
                    class: "inline-flex w-full items-center justify-center gap-2 rounded-sm bg-primary px-3 py-2 text-xs font-semibold text-primary-foreground",
                    onclick: move |_| {
                        let new_id = format!("thread-{}", state.read().threads.len() + 1);
                        state.write().threads.insert(0, crate::lib::UiThread {
                            id: new_id.clone(),
                            title: "New Thread".to_string(),
                            status: ThreadStatus::Idle,
                            updated_at: "now".to_string(),
                        });
                        state.write().active_thread_id = Some(new_id.clone());
                        navigator.push(Route::ThreadView { id: new_id });
                    },
                    Icon { width: 14, height: 14, icon: LdPlus }
                    span { "New Thread" }
                }
            }

            div { class: "flex-1 overflow-auto p-2 space-y-1",
                for thread in state.read().threads.clone() {
                    ThreadRow { thread }
                }
            }

            div {
                class: "p-2 border-t border-border",
                button {
                    class: "inline-flex w-full items-center justify-center gap-2 rounded-sm border border-border bg-background px-3 py-2 text-xs font-medium",
                    onclick: move |_| {
                        state.write().show_kanban = true;
                        navigator.push(Route::Board {});
                    },
                    Icon { width: 14, height: 14, icon: LdLayoutGrid }
                    span { "Overview" }
                }
            }
        }
    }
}

#[component]
fn ThreadRow(thread: crate::lib::UiThread) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let navigator = use_navigator();
    let thread_id_for_open = thread.id.clone();
    let thread_id_for_delete = thread.id.clone();
    let active = state
        .read()
        .active_thread_id
        .as_ref()
        .map(|id| id == &thread.id)
        .unwrap_or(false);
    let class = if active {
        "group w-full rounded-sm px-2 py-2 text-left transition bg-sidebar-accent text-sidebar-accent-foreground"
    } else {
        "group w-full rounded-sm px-2 py-2 text-left transition hover:bg-background-interactive text-muted-foreground"
    };

    rsx! {
        button {
            class: "{class}",
            onclick: move |_| {
                state.write().active_thread_id = Some(thread_id_for_open.clone());
                state.write().show_kanban = false;
                navigator.push(Route::ThreadView { id: thread_id_for_open.clone() });
            },
            div { class: "flex items-center gap-2",
                StatusIcon { status: thread.status.clone() }
                div { class: "min-w-0 flex-1",
                    div { class: "truncate text-xs font-semibold", "{thread.title}" }
                    div { class: "text-[10px] text-muted-foreground", "{thread.updated_at}" }
                }
                button {
                    class: "rounded px-1 text-muted-foreground opacity-0 group-hover:opacity-100 hover:text-status-critical transition-opacity",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        state.write().threads.retain(|t| t.id != thread_id_for_delete);
                        if state.read().threads.is_empty() {
                            state.write().active_thread_id = None;
                        }
                    },
                    Icon { width: 12, height: 12, icon: LdTrash2 }
                }
            }
        }
    }
}

#[component]
fn StatusIcon(status: ThreadStatus) -> Element {
    match status {
        ThreadStatus::Busy => {
            rsx! { Icon { class: "animate-spin text-status-info", width: 12, height: 12, icon: LdLoader } }
        }
        ThreadStatus::Interrupted | ThreadStatus::Error => {
            rsx! { Icon { class: "text-status-warning", width: 12, height: 12, icon: LdTriangleAlert } }
        }
        _ => {
            rsx! { Icon { class: "text-muted-foreground", width: 12, height: 12, icon: LdMessageSquare } }
        }
    }
}
