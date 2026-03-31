use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Default,
    Secondary,
    Ghost,
    Destructive,
    Outline,
    Nominal,
    Warning,
    Critical,
    Info,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Default,
    Sm,
    Lg,
    Icon,
}

fn button_variant_class(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Default => "bg-primary text-primary-foreground hover:bg-primary/90",
        ButtonVariant::Secondary => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ButtonVariant::Ghost => "bg-transparent text-foreground hover:bg-accent/20",
        ButtonVariant::Destructive => {
            "bg-destructive text-destructive-foreground hover:bg-destructive/90"
        }
        ButtonVariant::Outline => "border border-border bg-transparent hover:bg-accent/15",
        ButtonVariant::Nominal => {
            "bg-status-nominal/20 text-status-nominal hover:bg-status-nominal/30"
        }
        ButtonVariant::Warning => {
            "bg-status-warning/20 text-status-warning hover:bg-status-warning/30"
        }
        ButtonVariant::Critical => {
            "bg-status-critical/20 text-status-critical hover:bg-status-critical/30"
        }
        ButtonVariant::Info => "bg-status-info/20 text-status-info hover:bg-status-info/30",
    }
}

fn button_size_class(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::Default => "h-9 px-4 py-2 text-xs",
        ButtonSize::Sm => "h-8 px-3 text-[11px]",
        ButtonSize::Lg => "h-10 px-6 text-sm",
        ButtonSize::Icon => "h-8 w-8",
    }
}

#[component]
pub fn Button(
    children: Element,
    #[props(default = ButtonVariant::Default)] variant: ButtonVariant,
    #[props(default = ButtonSize::Default)] size: ButtonSize,
    #[props(default = false)] disabled: bool,
    #[props(default = None)] onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        button {
            class: "inline-flex items-center justify-center gap-2 rounded-sm font-medium transition-colors focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50 {button_variant_class(variant)} {button_size_class(size)}",
            disabled,
            onclick: move |evt| {
                if let Some(handler) = onclick {
                    handler.call(evt);
                }
            },
            {children}
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Destructive,
    Outline,
    Nominal,
    Warning,
    Critical,
    Info,
}

fn badge_variant_class(variant: BadgeVariant) -> &'static str {
    match variant {
        BadgeVariant::Default => "bg-primary/20 text-primary border-primary/30",
        BadgeVariant::Secondary => "bg-secondary/30 text-secondary-foreground border-border",
        BadgeVariant::Destructive => "bg-destructive/20 text-destructive border-destructive/30",
        BadgeVariant::Outline => "bg-transparent text-muted-foreground border-border",
        BadgeVariant::Nominal => {
            "bg-status-nominal/20 text-status-nominal border-status-nominal/30"
        }
        BadgeVariant::Warning => {
            "bg-status-warning/20 text-status-warning border-status-warning/30"
        }
        BadgeVariant::Critical => {
            "bg-status-critical/20 text-status-critical border-status-critical/30"
        }
        BadgeVariant::Info => "bg-status-info/20 text-status-info border-status-info/30",
    }
}

#[component]
pub fn Badge(
    children: Element,
    #[props(default = BadgeVariant::Default)] variant: BadgeVariant,
    #[props(default = String::new())] class: String,
) -> Element {
    rsx! {
        span {
            class: "inline-flex items-center rounded-sm border px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide {badge_variant_class(variant)} {class}",
            {children}
        }
    }
}

#[component]
pub fn Input(
    #[props(default = String::new())] value: String,
    #[props(default = String::new())] placeholder: String,
    #[props(default = None)] oninput: Option<EventHandler<FormEvent>>,
) -> Element {
    rsx! {
        input {
            class: "h-9 w-full rounded-sm border border-border bg-background px-3 text-xs text-foreground outline-none placeholder:text-muted-foreground focus:border-primary",
            value,
            placeholder,
            oninput: move |evt| {
                if let Some(handler) = oninput {
                    handler.call(evt);
                }
            }
        }
    }
}

#[component]
pub fn Card(children: Element) -> Element {
    rsx! {
        div {
            class: "rounded-sm border border-border bg-background-elevated text-foreground",
            {children}
        }
    }
}

#[component]
pub fn CardHeader(children: Element) -> Element {
    rsx! {
        div { class: "px-4 py-3 border-b border-border", {children} }
    }
}

#[component]
pub fn CardContent(children: Element) -> Element {
    rsx! {
        div { class: "p-4", {children} }
    }
}

#[component]
pub fn ScrollArea(children: Element) -> Element {
    rsx! {
        div { class: "overflow-auto scrollbar-thin", {children} }
    }
}

#[component]
pub fn Separator() -> Element {
    rsx! { div { class: "h-px w-full bg-border" } }
}

#[component]
pub fn Dialog(
    #[props(default = false)] open: bool,
    children: Element,
    #[props(default = None)] on_close: Option<EventHandler<MouseEvent>>,
) -> Element {
    if !open {
        return rsx! { Fragment {} };
    }

    rsx! {
        div {
            class: "fixed inset-0 z-[120] flex items-center justify-center bg-black/65",
            onclick: move |evt| {
                if let Some(handler) = on_close {
                    handler.call(evt);
                }
            },
            div {
                class: "w-[560px] max-w-[95vw] rounded-sm border border-border bg-background-elevated p-4",
                onclick: move |evt| evt.stop_propagation(),
                {children}
            }
        }
    }
}

#[component]
pub fn Popover(
    #[props(default = false)] open: bool,
    on_close: EventHandler<()>,
    trigger: Element,
    children: Element,
) -> Element {
    let open_attr = if open { "true" } else { "" };
    rsx! {
        if open {
            div {
                class: "fixed inset-0 z-[100]",
                onclick: move |_| on_close.call(()),
                onkeydown: move |e: Event<KeyboardData>| {
                    if e.key() == Key::Escape { on_close.call(()); }
                },
            }
        }
        omni-popper {
            placement: "bottom-start",
            offset: "0,8",
            strategy: "fixed",
            "open": "{open_attr}",
            div { slot: "trigger", {trigger} }
            div {
                slot: "content",
                class: "z-[110] w-[360px] rounded-sm border border-border bg-background-elevated p-2 shadow-xl",
                onclick: move |e: MouseEvent| e.stop_propagation(),
                {children}
            }
        }
    }
}
