use dioxus::prelude::*;

#[component]
pub fn MarkdownViewer(path: String, content: String) -> Element {
    rsx! {
        div { class: "h-full w-full overflow-auto",
            omni-mdx {
                "data-value": "{content}",
                "data-readonly": "true",
                style: "display:block;width:100%;height:100%;"
            }
        }
    }
}
