use dioxus::prelude::*;

use crate::lib::utils::file_name;

#[component]
pub fn BinaryViewer(path: String, size: Option<u64>) -> Element {
    let filename = file_name(&path);
    let ext = path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_uppercase()
        .to_string();

    rsx! {
        div { class: "flex h-full items-center justify-center bg-background",
            div { class: "flex flex-col items-center gap-3 rounded-lg border border-border bg-background-elevated p-8 text-center",
                omni-text { "data-text": "⬛", "data-strategy": "none", "data-max-lines": "1", class: "text-4xl text-muted-foreground" }
                omni-text { "data-text": "{filename}", "data-strategy": "truncate", "data-max-lines": "1", class: "text-sm font-medium text-foreground" }
                if !ext.is_empty() {
                    omni-text { "data-text": "{ext} file", "data-strategy": "none", "data-max-lines": "1", class: "rounded bg-background px-2 py-0.5 text-[11px] text-muted-foreground" }
                }
                if let Some(bytes) = size {
                    omni-text { "data-text": "{bytes} bytes", "data-strategy": "none", "data-max-lines": "1", class: "text-[11px] text-muted-foreground" }
                }
                omni-text { "data-text": "Cannot preview this file type.", "data-strategy": "none", "data-max-lines": "1", class: "text-[11px] text-muted-foreground" }
            }
        }
    }
}
