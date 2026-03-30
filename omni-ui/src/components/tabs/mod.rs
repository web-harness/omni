use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdBot, LdFile, LdFileCode, LdFileImage, LdFileText, LdFileVideo, LdX,
};
use dioxus_free_icons::Icon;

use crate::components::chat::ChatContainer;
use crate::lib::{
    file_types::{ext_to_mime_type, get_file_type, FileType},
    fixtures::{fixture_b64, fixture_text},
    ThreadState, WorkspaceState,
};

pub mod viewers;
use viewers::{
    BinaryViewer, CodeViewer, HtmlViewer, ImageViewer, MarkdownViewer, MediaViewer, PdfViewer,
};

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
    match get_file_type(&path) {
        FileType::Code => rsx! { Icon { width: 13, height: 13, icon: LdFileCode } },
        FileType::Markdown | FileType::Text => {
            rsx! { Icon { width: 13, height: 13, icon: LdFileText } }
        }
        FileType::Image => rsx! { Icon { width: 13, height: 13, icon: LdFileImage } },
        FileType::Video | FileType::Audio => {
            rsx! { Icon { width: 13, height: 13, icon: LdFileVideo } }
        }
        FileType::Html => rsx! { Icon { width: 13, height: 13, icon: LdFileCode } },
        _ => rsx! { Icon { width: 13, height: 13, icon: LdFile } },
    }
}

#[component]
pub fn TabbedPanel(thread_id: String) -> Element {
    rsx! {
        ChatContainer { thread_id }
    }
}

#[component]
pub fn FileViewer(path: String, thread_id: String) -> Element {
    let mut workspace_state = use_context::<Signal<WorkspaceState>>();
    let ext = path.rsplit('.').next().unwrap_or("").to_string();
    let mime = ext_to_mime_type(&ext).to_string();
    let path_for_close = path.clone();
    let tid_for_close = thread_id.clone();

    let viewer = if ext == "svg" {
        let svg = fixture_text(&path);
        rsx! {
            div {
                class: "h-full w-full overflow-auto flex items-center justify-center bg-background p-4",
                dangerous_inner_html: "{svg}",
            }
        }
    } else {
        match get_file_type(&path) {
            FileType::Code | FileType::Text => rsx! {
                CodeViewer { path: path.clone(), content: fixture_text(&path) }
            },
            FileType::Markdown => rsx! {
                MarkdownViewer { path: path.clone(), content: fixture_text(&path) }
            },
            FileType::Html => rsx! {
                HtmlViewer { path: path.clone(), content: fixture_text(&path) }
            },
            FileType::Image => rsx! {
                ImageViewer {
                    path: path.clone(),
                    base64_content: fixture_b64(&path),
                    mime_type: mime,
                }
            },
            FileType::Video => rsx! {
                MediaViewer {
                    path: path.clone(),
                    base64_content: fixture_b64(&path),
                    mime_type: mime,
                }
            },
            FileType::Audio => rsx! {
                MediaViewer {
                    path: path.clone(),
                    base64_content: fixture_b64(&path),
                    mime_type: mime,
                }
            },
            FileType::Pdf => rsx! {
                PdfViewer { path: path.clone(), base64_content: fixture_b64(&path) }
            },
            FileType::Binary => rsx! {
                BinaryViewer { path: path.clone(), size: None }
            },
        }
    };

    rsx! {
        div { class: "h-full w-full flex flex-col",
            div { class: "flex items-center justify-between px-3 py-1 bg-sidebar border-b border-border text-[11px] shrink-0",
                span { class: "text-muted-foreground truncate", "{path}" }
                button {
                    class: "ml-2 text-muted-foreground hover:text-foreground leading-none",
                    onclick: move |_| {
                        workspace_state
                            .write()
                            .open_tabs
                            .entry(tid_for_close.clone())
                            .or_default()
                            .retain(|x| x != &path_for_close);
                    },
                    "×"
                }
            }
            div { class: "flex-1 overflow-hidden relative", {viewer} }
        }
    }
}
