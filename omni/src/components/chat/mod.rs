use dioxus::prelude::*;

#[component]
pub fn ChatView(thread_id: String) -> Element {
    rsx! {
        div {
            class: "flex flex-col flex-1 bg-zinc-900",
            div {
                class: "flex-1 overflow-y-auto p-4 space-y-4",
                MessageBubble {
                    role: "assistant".to_string(),
                    content: "Hello! How can I help you today?".to_string(),
                }
                MessageBubble {
                    role: "user".to_string(),
                    content: "I need some help".to_string(),
                }
            }
            ChatInput { thread_id }
        }
    }
}

#[component]
pub fn MessageBubble(role: String, content: String) -> Element {
    let is_user = role == "user";
    let bubble_class = if is_user {
        "bg-blue-600 text-white rounded-br-none ml-auto"
    } else {
        "bg-gray-800 text-gray-100 rounded-bl-none"
    };

    rsx! {
        div {
            class: format!("max-w-md p-3 rounded-lg {}", bubble_class),
            p { "{content}" }
        }
    }
}

#[component]
pub fn ChatInput(thread_id: String) -> Element {
    rsx! {
        div {
            class: "flex gap-2 p-4 border-t border-zinc-700",
            input {
                class: "flex-1 px-4 py-2 bg-zinc-800 text-white rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                placeholder: "Type a message...",
            }
            button {
                class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition",
                "Send"
            }
        }
    }
}

pub mod chat {
    pub use super::*;
}
