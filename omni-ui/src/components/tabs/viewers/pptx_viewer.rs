use dioxus::prelude::*;

use crate::lib::utils::file_name;

#[component]
pub fn PptxViewer(path: String, source_url: String) -> Element {
    let filename = file_name(&path);

    rsx! {
        omni-pptx-renderer {
            class: "block h-full w-full",
            "data-source-url": "{source_url}",
            "data-filename": "{filename}",
        }
    }
}
