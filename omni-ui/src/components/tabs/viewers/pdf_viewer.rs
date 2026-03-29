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
            "data-src": "{src}",
            "data-filename": "{filename}",
            style: "display:flex;width:100%;height:100%;"
        }
    }
}
