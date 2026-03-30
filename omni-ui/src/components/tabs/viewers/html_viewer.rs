use dioxus::prelude::*;

#[component]
pub fn HtmlViewer(path: String, content: String) -> Element {
    rsx! {
        iframe {
            class: "w-full h-full border-none bg-white",
            srcdoc: "{content}",
            "sandbox": "allow-scripts",
        }
    }
}
