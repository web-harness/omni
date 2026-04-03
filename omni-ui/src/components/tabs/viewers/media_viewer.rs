use dioxus::prelude::*;

#[component]
pub fn MediaViewer(source_url: String, mime_type: String) -> Element {
    let media_type = if mime_type.starts_with("video") {
        "video"
    } else {
        "audio"
    };

    rsx! {
        div { class: "flex h-full items-center justify-center bg-background p-4",
            omni-plyr {
                class: "w-full h-full",
                "data-source-url": "{source_url}",
                "data-mime": "{mime_type}",
                "data-type": "{media_type}",
            }
        }
    }
}
