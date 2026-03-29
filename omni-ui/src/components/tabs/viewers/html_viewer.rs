use dioxus::prelude::*;

#[component]
pub fn HtmlViewer(path: String, content: String) -> Element {
    rsx! {
        iframe {
            srcdoc: "{content}",
            "sandbox": "allow-scripts",
            style: "width:100%;height:100%;border:none;background:white;"
        }
    }
}
