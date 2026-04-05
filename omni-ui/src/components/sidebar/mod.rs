use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdLayoutGrid, LdLoader, LdMessageSquare, LdMoon, LdPlus, LdSun, LdTrash2, LdTriangleAlert,
};
use dioxus_free_icons::Icon;

use crate::lib::utils::relative_time;
use crate::lib::{Theme, ThreadState, ThreadStatus, UiState};
use crate::routes::Route;

#[component]
pub fn ThreadSidebar() -> Element {
    let mut thread_state = use_context::<Signal<ThreadState>>();
    let mut ui_state = use_context::<Signal<UiState>>();
    let navigator = use_navigator();

    rsx! {
        aside {
            class: "h-full w-full border-r border-border bg-sidebar flex flex-col overflow-hidden",
            div { class: "px-3 py-2 border-b border-border",
                button {
                    class: "inline-flex w-full items-center justify-center gap-2 rounded-sm bg-primary px-3 py-2 text-xs font-semibold text-primary-foreground",
                    onclick: move |_| {
                        // Optimistic UI: insert a placeholder immediately, then persist async
                        let temp_id = format!("thread-{}", thread_state.read().threads.len() + 1);
                        thread_state.write().threads.insert(0, crate::lib::UiThread {
                            id: temp_id.clone(),
                            title: "New Thread".to_string(),
                            status: crate::lib::ThreadStatus::Idle,
                            updated_at: "now".to_string(),
                        });
                        thread_state.write().active_thread_id = Some(temp_id.clone());
                        navigator.push(Route::ThreadView { id: temp_id.clone() });

                        #[cfg(target_arch = "wasm32")]
                        {
                            let mut t_state = thread_state;
                            let tid = temp_id.clone();
                            let nav = navigator.clone();
                            spawn(async move {
                                if let Ok(thread) = crate::lib::sw_api::create_thread().await {
                                    let real_id = thread.id;
                                    // Replace the temporary thread with the persisted one
                                    if let Some(entry) = t_state.write().threads.iter_mut().find(|t| t.id == tid) {
                                        entry.id = real_id.clone();
                                        entry.updated_at = thread.updated_at;
                                    }
                                    if t_state.read().active_thread_id.as_deref() == Some(&tid) {
                                        t_state.write().active_thread_id = Some(real_id.clone());
                                        nav.push(Route::ThreadView { id: real_id });
                                    }
                                }
                            });
                        }
                    },
                    Icon { width: 14, height: 14, icon: LdPlus }
                    omni-text { "data-text": "New Thread", "data-strategy": "none", "data-max-lines": "1" }
                }
            }

            div { class: "flex-1 overflow-auto p-2 space-y-1",
                for thread in thread_state.read().threads.clone() {
                    ThreadRow { key: "{thread.id}", thread }
                }
            }

            div {
                class: "p-2 border-t border-border flex gap-1",
                button {
                    class: "inline-flex items-center justify-center rounded-sm border border-border bg-background p-2 text-muted-foreground hover:text-foreground",
                    title: "Toggle theme",
                    onclick: move |_| {
                        let next = match ui_state.read().theme {
                            Theme::Dark => Theme::Light,
                            Theme::Light => Theme::Dark,
                        };
                        ui_state.write().theme = next;
                    },
                    if ui_state.read().theme == Theme::Dark {
                        Icon { width: 14, height: 14, icon: LdSun }
                    } else {
                        Icon { width: 14, height: 14, icon: LdMoon }
                    }
                }
                button {
                    class: "inline-flex flex-1 items-center justify-center gap-2 rounded-sm border border-border bg-background px-3 py-2 text-xs font-medium",
                    onclick: move |_| {
                        thread_state.write().show_kanban = true;
                        navigator.push(Route::Board {});
                    },
                    Icon { width: 14, height: 14, icon: LdLayoutGrid }
                    omni-text { "data-text": "Overview", "data-strategy": "none", "data-max-lines": "1" }
                }
            }
        }
    }
}

#[component]
fn ThreadRow(thread: crate::lib::UiThread) -> Element {
    let mut thread_state = use_context::<Signal<ThreadState>>();
    let navigator = use_navigator();
    let thread_id_for_open = thread.id.clone();
    let thread_id_for_delete = thread.id.clone();
    let active = thread_state
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
                thread_state.write().active_thread_id = Some(thread_id_for_open.clone());
                thread_state.write().show_kanban = false;
                navigator.push(Route::ThreadView { id: thread_id_for_open.clone() });
            },
            div { class: "flex items-center gap-2",
                StatusIcon { status: thread.status.clone() }
                div { class: "min-w-0 flex-1",
                    omni-text {
                        "data-text": "{thread.title}",
                        "data-strategy": "truncate",
                        "data-max-lines": "1",
                        class: "text-xs font-semibold",
                    }
                    omni-text { "data-text": "{relative_time(&thread.updated_at)}", "data-strategy": "none", "data-max-lines": "1", class: "text-[10px] text-muted-foreground" }
                }
                button {
                    class: "rounded px-1 text-muted-foreground opacity-0 group-hover:opacity-100 hover:text-status-critical transition-opacity",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        let id = thread_id_for_delete.clone();
                        thread_state.write().threads.retain(|t| t.id != id);
                        if thread_state.read().threads.is_empty() {
                            thread_state.write().active_thread_id = None;
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            let id2 = id.clone();
                            spawn(async move {
                                let _ = crate::lib::sw_api::delete_thread(&id2).await;
                            });
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
