use dioxus::prelude::*;

#[component]
pub fn TabBar() -> Element {
    rsx! {
        div {
            class: "flex gap-2 border-b border-zinc-700 bg-zinc-900 px-4 py-3",
            Tab { title: "Chat".to_string(), active: true, href: "/" }
            Tab { title: "Board".to_string(), active: false, href: "/board" }
            Tab { title: "Settings".to_string(), active: false, href: "/settings" }
        }
    }
}

#[component]
pub fn Tab(title: String, active: bool, href: String) -> Element {
    let tab_class = if active {
        "text-white border-b-2 border-blue-600"
    } else {
        "text-gray-400 hover:text-gray-200"
    };

    rsx! {
        Link {
            to: href,
            class: format!("px-4 py-2 {} transition", tab_class),
            "{title}"
        }
    }
}

pub mod tabs {
    pub use super::*;
}
