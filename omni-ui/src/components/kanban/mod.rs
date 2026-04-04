use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdCircleDot, LdGitBranch};
use dioxus_free_icons::Icon;

use crate::lib::utils::relative_time;
use crate::lib::{
    BackgroundTask, BackgroundTaskState, BackgroundTaskStatus, ThreadState, ThreadStatus, UiThread,
};
use crate::routes::Route;

#[component]
pub fn KanbanView() -> Element {
    let thread_state = use_context::<Signal<ThreadState>>();
    let background_task_state = use_context::<Signal<BackgroundTaskState>>();
    let show_background_tasks = use_signal(|| true);

    let mut pending = vec![];
    let mut progress = vec![];
    let mut blocked = vec![];
    let mut done = vec![];

    for thread in thread_state.read().threads.clone() {
        match thread.status {
            ThreadStatus::Idle => pending.push(thread),
            ThreadStatus::Busy => progress.push(thread),
            ThreadStatus::Interrupted | ThreadStatus::Error => blocked.push(thread),
            ThreadStatus::Done => done.push(thread),
        }
    }

    let tid = thread_state
        .read()
        .active_thread_id
        .clone()
        .unwrap_or_default();
    let tasks = background_task_state.read().tasks_for(&tid);

    rsx! {
        div { class: "flex h-full min-h-0 flex-col",
            KanbanHeader { show_background_tasks }
            div { class: "grid min-h-0 flex-1 grid-cols-4 gap-3 overflow-auto p-3",
                KanbanColumn { title: "PENDING".to_string(), tone: "border-border".to_string(), threads: pending }
                KanbanColumn { title: "IN PROGRESS".to_string(), tone: "border-status-info".to_string(), threads: progress }
                KanbanColumn { title: "BLOCKED".to_string(), tone: "border-status-warning".to_string(), threads: blocked }
                KanbanColumn { title: "DONE".to_string(), tone: "border-status-nominal".to_string(), threads: done }
            }
            if show_background_tasks() {
                div { class: "border-t border-border px-3 py-2",
                    div { class: "mb-2 text-[10px] font-semibold text-muted-foreground", "BACKGROUND TASKS" }
                    div { class: "grid grid-cols-2 gap-2",
                        for task in tasks {
                            BackgroundTaskKanbanCard { key: "{task.id}", task }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn KanbanHeader(show_background_tasks: Signal<bool>) -> Element {
    let thread_state = use_context::<Signal<ThreadState>>();
    let active = thread_state
        .read()
        .threads
        .iter()
        .filter(|t| matches!(t.status, ThreadStatus::Busy))
        .count();
    let toggle_label = if show_background_tasks() {
        "Hide Background Tasks"
    } else {
        "Show Background Tasks"
    };

    rsx! {
        div { class: "flex items-center justify-between border-b border-border px-3 py-2",
            div { class: "inline-flex items-center gap-2 text-[11px] text-muted-foreground",
                Icon { width: 14, height: 14, icon: LdCircleDot }
                span { "KANBAN OVERVIEW • {active} ACTIVE" }
            }
            button {
                class: "rounded-sm border border-border px-2 py-1 text-[11px]",
                onclick: move |_| show_background_tasks.set(!show_background_tasks()),
                "{toggle_label}"
            }
        }
    }
}

#[component]
pub fn KanbanColumn(title: String, tone: String, threads: Vec<UiThread>) -> Element {
    rsx! {
        div { class: "min-w-0 rounded-sm border {tone} bg-muted/30",
            div { class: "flex items-center justify-between border-b border-border px-2 py-2 text-[10px] font-semibold",
                span { "{title}" }
                span { class: "text-muted-foreground", "{threads.len()}" }
            }
            div { class: "space-y-2 p-2",
                for thread in threads {
                    KanbanCard { key: "{thread.id}", thread }
                }
            }
        }
    }
}

#[component]
pub fn KanbanCard(thread: UiThread) -> Element {
    let navigator = use_navigator();

    rsx! {
        button {
            class: "w-full rounded-sm border border-border bg-background px-2 py-2 text-left hover:bg-background-elevated",
            onclick: move |_| {
                navigator.push(Route::ThreadView { id: thread.id.clone() });
            },
            omni-text {
                "data-text": "{thread.title}",
                "data-strategy": "shrink",
                "data-max-lines": "1",
                "data-min-size": "9",
                class: "text-[11px] font-semibold",
            }
            div { class: "mt-1 text-[10px] text-muted-foreground", "{relative_time(&thread.updated_at)}" }
        }
    }
}

#[component]
pub fn BackgroundTaskKanbanCard(task: BackgroundTask) -> Element {
    let tone = match task.status {
        BackgroundTaskStatus::Running => "text-status-info",
        BackgroundTaskStatus::Completed => "text-status-nominal",
        BackgroundTaskStatus::Failed => "text-status-critical",
        BackgroundTaskStatus::Pending => "text-status-warning",
    };

    rsx! {
        div { class: "rounded-sm border border-dashed border-border px-2 py-2",
            div { class: "inline-flex items-center gap-2 text-[11px] {tone}",
                Icon { width: 12, height: 12, icon: LdGitBranch }
                omni-text { "data-text": "{task.name}", "data-strategy": "truncate", "data-max-lines": "1", class: "text-[11px]" }
            }
            omni-text {
                "data-text": "{task.description}",
                "data-strategy": "truncate",
                "data-max-lines": "2",
                class: "text-[10px] text-muted-foreground",
            }
        }
    }
}
