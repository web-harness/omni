use dioxus::prelude::*;

use crate::lib::file_types::ext_to_monaco_language;

#[component]
pub fn CodeViewer(path: String, content: String) -> Element {
    let ext = path.rsplit('.').next().unwrap_or("").to_string();
    let lang = ext_to_monaco_language(&ext).to_string();

    rsx! {
        div { class: "h-full w-full",
            omni-monaco {
                "data-value": "{content}",
                "data-language": "{lang}",
                "data-readonly": "true",
                "data-theme": "vs-dark",
                style: "display:block;width:100%;height:100%;"
            }
        }
    }
}
