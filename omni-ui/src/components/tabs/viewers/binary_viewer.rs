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
                div { class: "text-4xl text-muted-foreground", "⬛" }
                div { class: "text-sm font-medium text-foreground", "{filename}" }
                if !ext.is_empty() {
                    div { class: "rounded bg-background px-2 py-0.5 text-[11px] text-muted-foreground", "{ext} file" }
                }
                if let Some(bytes) = size {
                    div { class: "text-[11px] text-muted-foreground", "{bytes} bytes" }
                }
                div { class: "text-[11px] text-muted-foreground", "Cannot preview this file type." }
            }
        }
    }
}
