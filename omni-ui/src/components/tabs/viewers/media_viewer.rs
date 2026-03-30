use dioxus::prelude::*;

#[component]
pub fn MediaViewer(path: String, base64_content: String, mime_type: String) -> Element {
    let media_type = if mime_type.starts_with("video") {
        "video"
    } else {
        "audio"
    };

    rsx! {
        div { class: "flex h-full items-center justify-center bg-background p-4",
            omni-plyr {
                "data-base64": "{base64_content}",
                "data-mime": "{mime_type}",
                "data-type": "{media_type}",
                style: "width:100%;height:100%;"
            }
        }
    }
}
