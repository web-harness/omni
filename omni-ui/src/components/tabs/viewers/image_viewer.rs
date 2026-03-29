use dioxus::prelude::*;

#[component]
pub fn ImageViewer(path: String, base64_content: String, mime_type: String) -> Element {
    let mut zoom = use_signal(|| 100u32);
    let mut rotation = use_signal(|| 0u32);

    let data_url = format!("data:{mime_type};base64,{base64_content}");

    let scale = *zoom.read() as f32 / 100.0;
    let rot = *rotation.read();

    rsx! {
        div { class: "flex h-full flex-col",
            div { class: "flex items-center gap-2 border-b border-border bg-sidebar px-3 py-1.5",
                button {
                    class: "rounded px-2 py-1 text-[11px] text-muted-foreground hover:bg-background-interactive",
                    onclick: move |_| { let z = (*zoom.read()).saturating_sub(25).max(25); zoom.set(z); },
                    "−"
                }
                span { class: "text-[11px] text-muted-foreground", "{zoom}%" }
                button {
                    class: "rounded px-2 py-1 text-[11px] text-muted-foreground hover:bg-background-interactive",
                    onclick: move |_| { let z = (*zoom.read() + 25).min(400); zoom.set(z); },
                    "+"
                }
                button {
                    class: "rounded px-2 py-1 text-[11px] text-muted-foreground hover:bg-background-interactive",
                    onclick: move |_| zoom.set(100),
                    "Reset"
                }
                div { class: "mx-2 h-4 w-px bg-border" }
                button {
                    class: "rounded px-2 py-1 text-[11px] text-muted-foreground hover:bg-background-interactive",
                    onclick: move |_| { let r = (*rotation.read() + 90) % 360; rotation.set(r); },
                    "↻"
                }
            }
            div { class: "flex flex-1 items-center justify-center overflow-auto bg-background p-4",
                img {
                    src: "{data_url}",
                    alt: "{path}",
                    style: "transform: rotate({rot}deg) scale({scale}); transition: transform 0.2s; max-width: 100%; max-height: 100%; object-fit: contain;"
                }
            }
        }
    }
}
