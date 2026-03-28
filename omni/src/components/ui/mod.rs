use dioxus::prelude::*;

#[component]
pub fn Button(label: String, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition",
            onclick: move |evt| onclick.call(evt),
            "{label}"
        }
    }
}

#[component]
pub fn Input(placeholder: String, onchange: EventHandler<String>) -> Element {
    rsx! {
        input {
            class: "px-4 py-2 bg-zinc-800 text-white rounded-lg border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-blue-500",
            placeholder: "{placeholder}",
            onchange: move |evt| onchange.call(evt.value()),
        }
    }
}

#[component]
pub fn Badge(text: String) -> Element {
    rsx! {
        span {
            class: "inline-block px-2 py-1 text-xs font-semibold bg-blue-600 text-white rounded-full",
            "{text}"
        }
    }
}

#[component]
pub fn Spinner() -> Element {
    rsx! {
        div {
            class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600",
        }
    }
}

pub mod ui {
    pub use super::*;
}
