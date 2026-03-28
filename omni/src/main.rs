use dioxus::prelude::*;

mod components;
mod lib;

use components::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
    #[route("/thread/:id")]
    ThreadView { id: String },
    #[route("/board")]
    Board {},
    #[route("/settings")]
    Settings {},
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Layout() -> Element {
    rsx! {
        div {
            class: "flex h-screen bg-zinc-950",
            Sidebar {}
            div {
                class: "flex flex-col flex-1",
                TabBar {}
                div {
                    class: "flex-1 overflow-hidden",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center h-full p-8",
            h1 {
                class: "text-4xl font-bold text-white mb-4",
                "Welcome to Omni"
            }
            p {
                class: "text-lg text-gray-400",
                "Start a new conversation or select a thread"
            }
        }
    }
}

#[component]
fn ThreadView(id: String) -> Element {
    rsx! {
        div {
            class: "flex h-full",
            ChatView { thread_id: id.clone() }
            FilePanel { thread_id: id }
        }
    }
}

#[component]
fn Board() -> Element {
    rsx! {
        div {
            class: "flex flex-col h-full p-6 bg-zinc-900",
            h2 {
                class: "text-2xl font-bold text-white mb-4",
                "Task Board"
            }
            div {
                class: "flex gap-6 flex-1 overflow-x-auto",
                KanbanColumn { title: "To Do".to_string() }
                KanbanColumn { title: "In Progress".to_string() }
                KanbanColumn { title: "Done".to_string() }
            }
        }
    }
}

#[component]
fn Settings() -> Element {
    rsx! {
        div {
            class: "flex flex-col h-full p-8 bg-zinc-900",
            h2 {
                class: "text-2xl font-bold text-white mb-6",
                "Settings"
            }
            div {
                class: "space-y-4",
                div {
                    class: "p-4 bg-zinc-800 rounded-lg",
                    p {
                        class: "text-gray-300",
                        "Configure your agent preferences and API keys here."
                    }
                }
            }
        }
    }
}
