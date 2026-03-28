use dioxus::prelude::*;

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        aside {
            class: "w-64 bg-zinc-900 border-r border-zinc-700 flex flex-col overflow-hidden",
            div {
                class: "p-4 border-b border-zinc-700",
                button {
                    class: "w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition",
                    "+ New Chat"
                }
            }
            ThreadList {}
        }
    }
}

#[component]
pub fn ThreadList() -> Element {
    rsx! {
        nav {
            class: "flex-1 overflow-y-auto space-y-1 p-2",
            ThreadItem { id: "thread-1".to_string(), title: "Conversation 1".to_string(), active: true }
            ThreadItem { id: "thread-2".to_string(), title: "Conversation 2".to_string(), active: false }
            ThreadItem { id: "thread-3".to_string(), title: "Conversation 3".to_string(), active: false }
        }
    }
}

#[component]
pub fn ThreadItem(id: String, title: String, active: bool) -> Element {
    let item_class = if active {
        "bg-zinc-800 text-white"
    } else {
        "text-gray-400 hover:bg-zinc-800"
    };

    rsx! {
        Link {
            to: crate::Route::ThreadView { id: id.clone() },
            class: format!("block px-4 py-2 rounded-lg transition cursor-pointer {}", item_class),
            "{title}"
        }
    }
}

pub mod sidebar {
    pub use super::*;
}
