use dioxus::prelude::*;

#[component]
pub fn MediaViewer(
    path: String,
    base64_content: String,
    mime_type: String,
    media_type: String,
) -> Element {
    let data_url = format!("data:{mime_type};base64,{base64_content}");

    rsx! {
        div { class: "flex h-full items-center justify-center bg-background p-4",
            if media_type == "video" {
                video {
                    src: "{data_url}",
                    controls: true,
                    style: "max-width:100%;max-height:100%;"
                }
            } else {
                audio {
                    src: "{data_url}",
                    controls: true,
                    style: "width:100%;max-width:480px;"
                }
            }
        }
    }
}
