use dioxus::prelude::*;

#[component]
pub fn MarkdownViewer(path: String, content: String) -> Element {
    rsx! {
        div { class: "h-full w-full overflow-auto",
            omni-mdx {
                class: "block w-full h-full",
                "data-value": "{content}",
                "data-readonly": "true",
            }
        }
    }
}
