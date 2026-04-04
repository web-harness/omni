use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdBrain, LdMinus, LdPlus};
use dioxus_free_icons::Icon;
use std::rc::Rc;

use crate::components::ui::{Button, Input, Popover};
use crate::lib::{agent_config_hash, derive_agent_name, AgentEndpoint, AgentEndpointState};

#[component]
fn RailTooltip(
    open: bool,
    label: String,
    trigger: Element,
    #[props(default = None)] on_open_change: Option<EventHandler<bool>>,
) -> Element {
    let open_attr = if open { "true" } else { "" };

    rsx! {
        omni-popper {
            placement: "right",
            offset: "4,0",
            strategy: "fixed",
            "open": "{open_attr}",
            div { slot: "trigger", {trigger} }
            div {
                slot: "content",
                class: "z-[120] rounded-sm border border-border bg-background-elevated px-2 py-1 text-[10px] text-foreground shadow-xl",
                onmouseenter: move |_| {
                    if let Some(handler) = on_open_change {
                        handler.call(true);
                    }
                },
                onmouseleave: move |_| {
                    if let Some(handler) = on_open_change {
                        handler.call(false);
                    }
                },
                span { class: "whitespace-nowrap", "{label}" }
            }
        }
    }
}

#[component]
fn RailClosePopover(
    open: bool,
    trigger: Element,
    on_open_change: EventHandler<bool>,
    on_delete: EventHandler<MouseEvent>,
) -> Element {
    let open_attr = if open { "true" } else { "" };

    rsx! {
        omni-popper {
            placement: "top-end",
            offset: "0,0",
            strategy: "fixed",
            "open": "{open_attr}",
            div { slot: "trigger", {trigger} }
            div {
                slot: "content",
                class: "z-[121] rounded-full bg-background/0 p-0 translate-x-1/2 translate-y-1/2",
                onmouseenter: move |_| on_open_change.call(true),
                onmouseleave: move |_| on_open_change.call(false),
                button {
                    class: "flex h-5 w-5 items-center justify-center rounded-full border border-status-critical/60 bg-status-critical text-white shadow-lg",
                    onmousedown: move |evt| evt.stop_propagation(),
                    onclick: move |evt| {
                        evt.stop_propagation();
                        on_delete.call(evt);
                    },
                    Icon { width: 10, height: 10, icon: LdMinus }
                }
            }
        }
    }
}

#[component]
fn AgentRailButton(
    endpoint: AgentEndpoint,
    active: bool,
    hovered: bool,
    dimmed: bool,
    dicebear_style: String,
    on_hover_change: EventHandler<bool>,
    on_activate: EventHandler<MouseEvent>,
    on_delete: EventHandler<MouseEvent>,
) -> Element {
    let mut trigger_hovered = use_signal(|| false);
    let mut tooltip_hovered = use_signal(|| false);
    let mut delete_hovered = use_signal(|| false);
    let show_overlays = trigger_hovered() || tooltip_hovered() || delete_hovered() || hovered;
    let opacity_class = if dimmed { "opacity-50" } else { "opacity-100" };
    let trigger_class = if endpoint.removable {
        "relative inline-flex"
    } else {
        "relative inline-flex"
    };
    let button_class = if active {
        "relative flex h-9 w-9 items-center justify-center overflow-hidden rounded-sm border border-primary bg-background-elevated ring-2 ring-primary/80 transition-opacity duration-150"
    } else {
        "relative flex h-9 w-9 items-center justify-center overflow-hidden rounded-sm border border-border bg-background-elevated hover:border-primary/50 hover:bg-background-interactive transition-opacity duration-150"
    };

    rsx! {
        div {
            class: "relative pr-3 pt-3 pb-1 pl-1 -mr-3 -mt-3 -mb-1 -ml-1",
            onmouseenter: move |_| {
                trigger_hovered.set(true);
                on_hover_change.call(true);
            },
            onmouseleave: move |_| {
                trigger_hovered.set(false);
                on_hover_change.call(false);
            },
            RailTooltip {
                open: show_overlays,
                label: endpoint.name.clone(),
                on_open_change: move |is_open| tooltip_hovered.set(is_open),
                trigger: rsx! {
                    div { class: "{trigger_class}",
                        button {
                            class: "{button_class} {opacity_class}",
                            onclick: move |evt| on_activate.call(evt),
                            if endpoint.removable {
                                omni-dicebear {
                                    class: "block h-full w-full",
                                    seed: "{endpoint.id}",
                                    "avatar-style": "{dicebear_style}",
                                    size: "36",
                                }
                            } else {
                                Icon { width: 20, height: 20, icon: LdBrain, class: "text-primary" }
                            }
                        }
                        if endpoint.removable {
                            RailClosePopover {
                                open: show_overlays,
                                on_open_change: move |is_open| delete_hovered.set(is_open),
                                on_delete: on_delete,
                                trigger: rsx! {
                                    div {
                                        class: "pointer-events-none absolute right-0 top-0 h-0 w-0",
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AgentRail() -> Element {
    let mut agent_state = use_context::<Signal<AgentEndpointState>>();
    let mut add_open = use_signal(|| false);
    let mut add_hovered = use_signal(|| false);
    let mut hovered_agent_id = use_signal(|| None::<String>);
    let mut scroll_container = use_signal(|| None::<Rc<MountedData>>);
    let mut scroll_requested = use_signal(|| false);
    let mut url_draft = use_signal(String::new);
    let mut token_draft = use_signal(String::new);

    let (ordered_endpoints, active_agent_id, dicebear_style) = {
        let snapshot = agent_state.read();
        (
            snapshot
                .ordered()
                .into_iter()
                .cloned()
                .collect::<Vec<AgentEndpoint>>(),
            snapshot.active_agent_id.clone(),
            snapshot.dicebear_style.clone(),
        )
    };
    let (pinned_endpoint, scrollable_endpoints) = match ordered_endpoints.split_first() {
        Some((first, rest)) => (Some(first.clone()), rest.to_vec()),
        None => (None, Vec::new()),
    };
    let current_hovered_agent_id = hovered_agent_id();
    let add_is_hovered = add_hovered();
    let add_button_opacity = if current_hovered_agent_id.is_some() && !add_is_hovered {
        "opacity-50"
    } else {
        "opacity-100"
    };

    use_effect(move || {
        if !scroll_requested() {
            return;
        }
        let container = scroll_container.read().as_ref().cloned();
        let Some(container) = container else {
            return;
        };
        scroll_requested.set(false);
        spawn(async move {
            if let Ok(size) = container.get_scroll_size().await {
                let _ = container
                    .scroll(
                        dioxus::html::geometry::PixelsVector2D::new(0.0, size.height),
                        ScrollBehavior::Smooth,
                    )
                    .await;
            }
        });
    });

    let mut add_endpoint = {
        let mut agent_state = agent_state;
        move || {
            let url = url_draft.read().trim().to_string();
            let bearer_token = token_draft.read().trim().to_string();
            if url.is_empty() || bearer_token.is_empty() {
                return;
            }

            let endpoint = AgentEndpoint {
                id: agent_config_hash(&url, &bearer_token),
                name: derive_agent_name(&url),
                url,
                bearer_token,
                removable: true,
            };

            agent_state.write().upsert(endpoint.clone());
            scroll_requested.set(true);
            url_draft.set(String::new());
            token_draft.set(String::new());
            add_open.set(false);

            #[cfg(target_arch = "wasm32")]
            spawn(async move {
                let _ = crate::lib::sw_api::set_agent_endpoint(&endpoint).await;
            });
        }
    };

    rsx! {
        div { class: "relative flex h-full w-12 shrink-0 flex-col border-r border-border bg-background",
            if let Some(endpoint) = pinned_endpoint {
                {
                    let endpoint_id = endpoint.id.clone();
                    let hover_id = endpoint_id.clone();
                    let is_active = active_agent_id.is_none();
                    let is_hovered = current_hovered_agent_id.as_deref() == Some(endpoint_id.as_str());
                    let is_dimmed = add_is_hovered || current_hovered_agent_id
                        .as_ref()
                        .map(|hovered| hovered != &endpoint_id)
                        .unwrap_or(false);

                    rsx! {
                        div { class: "relative z-10 flex shrink-0 justify-center px-1 pt-2 pb-1 bg-background",
                            AgentRailButton {
                                key: "{endpoint.id}",
                                endpoint: endpoint.clone(),
                                active: is_active,
                                hovered: is_hovered,
                                dimmed: is_dimmed,
                                dicebear_style: dicebear_style.clone(),
                                on_hover_change: move |is_hovering| {
                                    if is_hovering {
                                        hovered_agent_id.set(Some(hover_id.clone()));
                                    } else if hovered_agent_id.read().as_deref() == Some(hover_id.as_str()) {
                                        hovered_agent_id.set(None);
                                    }
                                },
                                on_activate: move |_| {
                                    agent_state.write().active_agent_id = None;
                                },
                                on_delete: move |_| {},
                            }
                        }
                    }
                }
            }

            div { class: "pointer-events-none absolute top-[3.25rem] left-0 right-0 z-20 h-5 bg-gradient-to-b from-background via-background/85 to-background/0" }

            div {
                class: "relative min-h-0 flex-1 overflow-y-auto overflow-x-hidden px-1 pt-1 pb-2 [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden",
                onmounted: move |event| scroll_container.set(Some(event.data())),
                for endpoint in scrollable_endpoints {
                    {
                        let endpoint_id = endpoint.id.clone();
                        let delete_id = endpoint_id.clone();
                        let hover_id = endpoint_id.clone();
                        let is_active = active_agent_id.as_deref() == Some(endpoint_id.as_str());
                        let is_hovered = current_hovered_agent_id.as_deref() == Some(endpoint_id.as_str());
                        let is_dimmed = add_is_hovered || current_hovered_agent_id
                            .as_ref()
                            .map(|hovered| hovered != &endpoint_id)
                            .unwrap_or(false);

                        rsx! {
                            div { class: "flex justify-center",
                                AgentRailButton {
                                    key: "{endpoint.id}",
                                    endpoint: endpoint.clone(),
                                    active: is_active,
                                    hovered: is_hovered,
                                    dimmed: is_dimmed,
                                    dicebear_style: dicebear_style.clone(),
                                    on_hover_change: move |is_hovering| {
                                        if is_hovering {
                                            hovered_agent_id.set(Some(hover_id.clone()));
                                        } else if hovered_agent_id.read().as_deref() == Some(hover_id.as_str()) {
                                            hovered_agent_id.set(None);
                                        }
                                    },
                                    on_activate: move |_| {
                                        agent_state.write().active_agent_id = Some(endpoint_id.clone());
                                    },
                                    on_delete: move |_| {
                                        agent_state.write().remove(&delete_id);
                                        #[cfg(target_arch = "wasm32")]
                                        {
                                            let endpoint_id_for_delete = delete_id.clone();
                                            spawn(async move {
                                                let _ = crate::lib::sw_api::delete_agent_endpoint(&endpoint_id_for_delete).await;
                                            });
                                        }
                                    },
                                }
                            }
                        }
                    }
                }

            }

            div { class: "pointer-events-none absolute bottom-[3.25rem] left-0 right-0 z-20 h-5 bg-gradient-to-t from-background via-background/85 to-background/0" }

            div { class: "relative z-10 flex shrink-0 justify-center px-1 pt-2 pb-2 bg-background",
                Popover {
                    open: add_open(),
                    on_close: move |_| add_open.set(false),
                    trigger: rsx! {
                        RailTooltip {
                            open: add_hovered() && !add_open(),
                            label: "Add New Agent".to_string(),
                            trigger: rsx! {
                                button {
                                    class: "flex h-9 w-9 items-center justify-center rounded-sm border border-dashed border-border bg-background-elevated text-muted-foreground hover:border-primary hover:text-foreground transition-opacity duration-150 {add_button_opacity}",
                                    onmouseenter: move |_| add_hovered.set(true),
                                    onmouseleave: move |_| add_hovered.set(false),
                                    onclick: move |_| add_open.set(true),
                                    Icon { width: 20, height: 20, icon: LdPlus }
                                }
                            }
                        }
                    },
                    div { class: "space-y-3",
                        div {
                            class: "space-y-1",
                            h3 { class: "text-xs font-semibold", "Add Agent" }
                            p { class: "text-[10px] text-muted-foreground", "Connect a LangGraph endpoint for direct chat routing." }
                        }
                        div { class: "space-y-2",
                            Input {
                                value: url_draft(),
                                placeholder: "https://agent.example.com/api".to_string(),
                                oninput: move |evt: Event<FormData>| url_draft.set(evt.value()),
                            }
                            Input {
                                value: token_draft(),
                                placeholder: "Bearer token".to_string(),
                                oninput: move |evt: Event<FormData>| token_draft.set(evt.value()),
                            }
                        }
                        div { class: "flex justify-end",
                            Button {
                                onclick: move |_| add_endpoint(),
                                "Add"
                            }
                        }
                    }
                }
            }
        }
    }
}
