use dioxus::prelude::*;

#[component]
pub fn KanbanColumn(title: String) -> Element {
    rsx! {
        div {
            class: "w-80 bg-zinc-800 rounded-lg p-4 flex flex-col",
            h3 {
                class: "text-lg font-semibold text-white mb-4",
                "{title}"
            }
            div {
                class: "flex-1 space-y-2 overflow-y-auto",
                KanbanCard { title: "Task 1".to_string() }
                KanbanCard { title: "Task 2".to_string() }
            }
        }
    }
}

#[component]
pub fn KanbanCard(title: String) -> Element {
    rsx! {
        div {
            class: "p-3 bg-zinc-700 rounded-lg border border-zinc-600 hover:border-blue-500 transition cursor-move",
            p {
                class: "text-sm text-gray-200",
                "{title}"
            }
        }
    }
}

pub mod kanban {
    pub use super::*;
}
