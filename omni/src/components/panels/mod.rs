use dioxus::prelude::*;

#[component]
pub fn FilePanel(thread_id: String) -> Element {
    rsx! {
        aside {
            class: "w-80 bg-zinc-800 border-l border-zinc-700 flex flex-col overflow-hidden",
            div {
                class: "p-4 border-b border-zinc-700",
                h3 {
                    class: "text-lg font-semibold text-white",
                    "Files"
                }
            }
            FileViewer { thread_id }
        }
    }
}

#[component]
pub fn FileViewer(thread_id: String) -> Element {
    rsx! {
        div {
            class: "flex-1 overflow-y-auto p-4 space-y-2",
            div {
                class: "p-2 bg-zinc-700 rounded text-sm text-gray-300",
                "file1.txt"
            }
            div {
                class: "p-2 bg-zinc-700 rounded text-sm text-gray-300",
                "data.json"
            }
        }
    }
}

pub mod panels {
    pub use super::*;
}
