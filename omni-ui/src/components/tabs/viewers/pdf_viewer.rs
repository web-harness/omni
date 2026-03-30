use dioxus::prelude::*;

#[component]
pub fn PdfViewer(path: String, base64_content: String) -> Element {
    let filename = path.rsplit('/').next().unwrap_or(&path).to_string();
    let src = if base64_content.is_empty() {
        String::new()
    } else {
        format!("data:application/pdf;base64,{base64_content}")
    };

    rsx! {
        omni-pdfjs {
            class: "flex w-full h-full",
            "data-src": "{src}",
            "data-filename": "{filename}",
        }
    }
}
