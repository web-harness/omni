use dioxus::prelude::*;

use crate::lib::utils::file_name;

#[component]
pub fn PdfViewer(path: String, source_url: String) -> Element {
    let filename = file_name(&path);

    rsx! {
        omni-pdfjs {
            class: "flex w-full h-full",
            "data-source-url": "{source_url}",
            "data-filename": "{filename}",
        }
    }
}
