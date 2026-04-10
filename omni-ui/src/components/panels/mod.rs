use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdBot, LdFile, LdFileCode2, LdFileText, LdFolder, LdRefreshCw,
};
use dioxus_free_icons::Icon;

use crate::components::ui::{Badge, BadgeVariant};
use crate::lib::utils::fmt_size;
use crate::lib::{
    BackgroundTaskState, BackgroundTaskStatus, FileInfo, TasksState, ThreadState, TodoStatus,
    WorkspaceState,
};

#[component]
pub fn TasksSection() -> Element {
    let thread_state = use_context::<Signal<ThreadState>>();
    let tasks_state = use_context::<Signal<TasksState>>();
    let tid = thread_state
        .read()
        .active_thread_id
        .clone()
        .unwrap_or_default();
    let todos = tasks_state.read().todos_for(&tid);
    let total = todos.len();
    let done = todos
        .iter()
        .filter(|t| t.status == TodoStatus::Completed)
        .count();

    rsx! {
        div { class: "overflow-auto",
            div { class: "flex items-center justify-between px-3 py-1.5 text-[10px] text-muted-foreground border-b border-border",
                omni-text { "data-text": "PROGRESS", "data-strategy": "none", "data-max-lines": "1", class: "font-semibold tracking-wide" }
                omni-text { "data-text": "{done}/{total}", "data-strategy": "none", "data-max-lines": "1" }
            }
            div { class: "py-1",
                for todo in todos {
                    div { key: "{todo.id}", class: "flex items-start gap-2 px-3 py-2 border-b border-border/50 hover:bg-background-interactive",
                        {
                            let (dot_class, ring_class) = match todo.status {
                                TodoStatus::InProgress => ("bg-status-info", "border-status-info"),
                                TodoStatus::Completed => ("bg-status-nominal", "border-status-nominal"),
                                _ => ("bg-transparent", "border-border"),
                            };
                            rsx! {
                                div { class: "mt-1 h-3 w-3 shrink-0 rounded-full border-2 {ring_class} flex items-center justify-center",
                                    if todo.status == TodoStatus::InProgress || todo.status == TodoStatus::Completed {
                                        div { class: "h-1.5 w-1.5 rounded-full {dot_class}" }
                                    }
                                }
                            }
                        }
                        div { class: "min-w-0 flex-1",
                            omni-text {
                                "data-text": "{todo.content}",
                                "data-strategy": "shrink-truncate",
                                "data-max-lines": "2",
                                "data-min-size": "9",
                                class: "text-[11px] leading-4",
                            }
                        }
                        if todo.status == TodoStatus::InProgress {
                            Badge { variant: BadgeVariant::Info, class: "shrink-0", "IN PROGRESS" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn FilesSection() -> Element {
    let mut workspace_state = use_context::<Signal<WorkspaceState>>();
    let thread_state = use_context::<Signal<ThreadState>>();
    let tid = thread_state
        .read()
        .active_thread_id
        .clone()
        .unwrap_or_default();
    let files = workspace_state.read().files_for_thread(&tid);
    let workspace = workspace_state.read().workspace_for(&tid);
    let mut workspace_state_for_sync = workspace_state;

    {
        let workspace_path = workspace.clone();
        let mut ws_state = workspace_state;
        let should_fetch = files.is_empty();
        use_effect(move || {
            if should_fetch {
                let workspace_path_for_task = workspace_path.clone();
                spawn(async move {
                    if let Ok(fetched) =
                        crate::lib::list_workspace_files(&workspace_path_for_task).await
                    {
                        ws_state
                            .write()
                            .workspace_files
                            .insert(workspace_path_for_task, fetched);
                    }
                });
            }
        });
    }

    rsx! {
        div { class: "overflow-auto",
            div { class: "flex items-center justify-between px-3 py-1.5 border-b border-border",
                omni-text { "data-text": "{workspace}", "data-strategy": "truncate", "data-max-lines": "1", class: "text-[10px] font-semibold text-muted-foreground tracking-wide" }
                button {
                    class: "flex items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground",
                    onclick: move |_| {
                        let workspace_path = workspace.clone();
                        spawn(async move {
                            if let Ok(files) = crate::lib::list_workspace_files(&workspace_path).await {
                                workspace_state_for_sync
                                    .write()
                                    .workspace_files
                                    .insert(workspace_path, files);
                            }
                        });
                    },
                    Icon { width: 10, height: 10, icon: LdRefreshCw }
                    omni-text { "data-text": "Sync", "data-strategy": "none", "data-max-lines": "1" }
                }
            }
            div { class: "py-1",
                for file in files {
                    {
                        let tid = tid.clone();
                        rsx! { FileRow { key: "{file.path}", file: file.clone(), on_open: move |path: String| {
                            let mut ws = workspace_state.write();
                            if ws.open_tabs_for(&tid).contains(&path) {
                                *ws.tab_generation.entry(path.clone()).or_insert(0) += 1;
                            } else {
                                ws.open_tabs.entry(tid.clone()).or_default().push(path.clone());
                            }
                            ws.active_tab.insert(tid.clone(), path);
                        }, workspace_root: workspace.clone() }}
                    }
                }
            }
        }
    }
}

#[component]
fn FileRow(file: FileInfo, on_open: EventHandler<String>, workspace_root: String) -> Element {
    let root = workspace_root.trim_end_matches('/');
    let relative = file
        .path
        .strip_prefix(&format!("{root}/"))
        .unwrap_or(&file.path)
        .to_string();
    let depth = relative.matches('/').count();
    let name = file
        .path
        .split('/')
        .next_back()
        .unwrap_or(&file.path)
        .to_string();
    let ext = name.rsplit('.').next().unwrap_or("").to_string();
    let indent = depth * 14 + 12;

    let (icon_el, icon_color) = if file.is_dir {
        ("folder", "text-muted-foreground")
    } else {
        match ext.as_str() {
            "js" | "jsx" | "ts" | "tsx" => ("code", "text-yellow-400"),
            "json" => ("code", "text-green-400"),
            "html" => ("code", "text-orange-400"),
            "css" | "scss" => ("code", "text-purple-400"),
            "rs" => ("code", "text-orange-500"),
            "md" | "txt" => ("text", "text-muted-foreground"),
            _ => ("file", "text-muted-foreground"),
        }
    };

    let size_str = file.size.map(fmt_size).unwrap_or_default();

    rsx! {
        button {
            class: "flex w-full items-center gap-1.5 py-1.5 pr-3 hover:bg-background-interactive text-left",
            style: "padding-left: {indent}px",
            onclick: move |_| {
                if !file.is_dir {
                    on_open.call(file.path.clone());
                }
            },
            if file.is_dir {
                Icon { width: 12, height: 12, icon: LdFolder, class: "{icon_color} shrink-0" }
            } else if icon_el == "text" || ext == "md" || ext == "txt" {
                Icon { width: 12, height: 12, icon: LdFileText, class: "{icon_color} shrink-0" }
            } else if icon_el == "file" {
                Icon { width: 12, height: 12, icon: LdFile, class: "{icon_color} shrink-0" }
            } else {
                Icon { width: 12, height: 12, icon: LdFileCode2, class: "{icon_color} shrink-0" }
            }
            omni-text { "data-text": "{name}", "data-strategy": "truncate", "data-max-lines": "1", class: "flex-1 text-[11px]" }
            if !size_str.is_empty() {
                omni-text { "data-text": "{size_str}", "data-strategy": "none", "data-max-lines": "1", class: "shrink-0 text-[10px] text-muted-foreground" }
            }
        }
    }
}

#[component]
pub fn BackgroundTasksSection() -> Element {
    let thread_state = use_context::<Signal<ThreadState>>();
    let background_task_state = use_context::<Signal<BackgroundTaskState>>();
    let tid = thread_state
        .read()
        .active_thread_id
        .clone()
        .unwrap_or_default();
    let tasks = background_task_state.read().tasks_for(&tid);

    rsx! {
        div { class: "overflow-auto py-1",
            for agent in tasks {
                div { key: "{agent.id}", class: "px-3 py-2 border-b border-border/50",
                    div { class: "flex items-center gap-2 mb-1",
                        Icon { width: 12, height: 12, icon: LdBot, class: "text-status-info shrink-0" }
                        omni-text { "data-text": "{agent.name}", "data-strategy": "truncate", "data-max-lines": "1", class: "flex-1 text-[11px] font-semibold" }
                        {
                            let (variant, label) = match agent.status {
                                BackgroundTaskStatus::Running => (BadgeVariant::Info, "RUNNING"),
                                BackgroundTaskStatus::Completed => (BadgeVariant::Nominal, "DONE"),
                                BackgroundTaskStatus::Failed => (BadgeVariant::Critical, "FAILED"),
                                BackgroundTaskStatus::Pending => (BadgeVariant::Warning, "PENDING"),
                            };
                            rsx! { Badge { variant, "{label}" } }
                        }
                    }
                    omni-text { "data-text": "{agent.description}", "data-strategy": "truncate", "data-max-lines": "3", class: "text-[10px] text-muted-foreground leading-4" }
                }
            }
        }
    }
}
