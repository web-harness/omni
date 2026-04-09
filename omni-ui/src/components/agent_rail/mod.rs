use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdBrain, LdPlus};
use dioxus_free_icons::Icon;
use std::rc::Rc;

use crate::lib::{
    AgentEndpoint, AgentEndpointState, FloatingDockState, FloatingPanel, FloatingPanelKind,
};

fn agent_initials(name: &str) -> String {
    let mut initials = name
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();

    if initials.is_empty() {
        initials = name
            .chars()
            .filter(|ch| ch.is_alphanumeric())
            .take(2)
            .collect::<String>()
            .to_uppercase();
    }

    if initials.is_empty() {
        "AG".to_string()
    } else {
        initials
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
) -> Element {
    let mut floating_dock = use_context::<Signal<FloatingDockState>>();
    let mut trigger_rect = use_signal(|| (0.0f64, 0.0f64, 0.0f64, 0.0f64));
    let opacity_class = if dimmed { "opacity-50" } else { "opacity-100" };
    let button_class = if active {
        "relative flex h-9 w-9 items-center justify-center overflow-hidden rounded-sm border border-primary bg-background-elevated ring-2 ring-primary/80 transition-opacity duration-150"
    } else {
        "relative flex h-9 w-9 items-center justify-center overflow-hidden rounded-sm border border-border bg-background-elevated hover:border-primary/50 hover:bg-background-interactive transition-opacity duration-150"
    };
    let fallback_initials = agent_initials(&endpoint.name);
    let endpoint_id = endpoint.id.clone();
    let endpoint_name = endpoint.name.clone();
    let show_overlays = hovered;
    let endpoint_id_leave = endpoint_id.clone();

    rsx! {
        div {
            class: "relative pr-3 pt-3 pb-1 pl-1 -mr-3 -mt-3 -mb-1 -ml-1",
            onmounted: move |evt| async move {
                if let Ok(cr) = evt.get_client_rect().await {
                    trigger_rect.set((cr.min_x(), cr.min_y(), cr.max_x(), cr.max_y()));
                }
            },
            onmouseenter: move |_| {
                on_hover_change.call(true);
                let (x0, y0, x1, y1) = trigger_rect();
                let tooltip_x = x1 + 4.0;
                let tooltip_y = (y0 + y1) / 2.0 - 12.0;
                floating_dock.write().open(FloatingPanel {
                    id: format!("tooltip-{}", endpoint_id),
                    kind: FloatingPanelKind::AgentTooltip { label: endpoint_name.clone() },
                    x: tooltip_x,
                    y: tooltip_y,
                    width: 0.0,
                    height: 24.0,
                });
                if endpoint.removable {
                    floating_dock.write().open(FloatingPanel {
                        id: format!("badge-{}", endpoint_id),
                        kind: FloatingPanelKind::AgentCloseBadge { agent_id: endpoint_id.clone() },
                        x: x1 - 10.0,
                        y: y0 - 10.0,
                        width: 20.0,
                        height: 20.0,
                    });
                }
            },
            onmouseleave: move |_| {
                on_hover_change.call(false);
                floating_dock.write().close(&format!("tooltip-{}", endpoint_id_leave));
                if endpoint.removable {
                    floating_dock.write().close(&format!("badge-{}", endpoint_id_leave));
                }
            },
            button {
                class: "{button_class} {opacity_class}",
                onclick: move |evt| on_activate.call(evt),
                if endpoint.removable {
                    div { class: "absolute inset-0 flex items-center justify-center bg-gradient-to-br from-status-info/25 via-primary/15 to-background-elevated text-[11px] font-semibold text-foreground",
                        "{fallback_initials.clone()}"
                    }
                    omni-dicebear {
                        class: "relative z-10 block h-full w-full",
                        seed: "{endpoint.id}",
                        "avatar-style": "{dicebear_style}",
                        size: "36",
                    }
                } else {
                    Icon { width: 20, height: 20, icon: LdBrain, class: "text-primary" }
                }
            }
        }
    }
}

#[component]
pub fn AgentRail() -> Element {
    let mut agent_state = use_context::<Signal<AgentEndpointState>>();
    let mut floating_dock = use_context::<Signal<FloatingDockState>>();
    let mut add_hovered = use_signal(|| false);
    let mut hovered_agent_id = use_signal(|| None::<String>);
    let mut scroll_container = use_signal(|| None::<Rc<MountedData>>);
    let mut scroll_requested = use_signal(|| false);
    let mut add_button_rect = use_signal(|| (0.0f64, 0.0f64, 0.0f64, 0.0f64));

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
                                }
                            }
                        }
                    }
                }

            }

            div { class: "pointer-events-none absolute bottom-[3.25rem] left-0 right-0 z-20 h-5 bg-gradient-to-t from-background via-background/85 to-background/0" }

            div { class: "relative z-10 flex shrink-0 justify-center px-1 pt-2 pb-2 bg-background",
                div {
                    onmounted: move |evt| async move {
                        if let Ok(cr) = evt.get_client_rect().await {
                            add_button_rect.set((cr.min_x(), cr.min_y(), cr.max_x(), cr.max_y()));
                        }
                    },
                    onmouseenter: move |_| {
                        add_hovered.set(true);
                        floating_dock.write().open(FloatingPanel {
                            id: "add-agent-tooltip".to_string(),
                            kind: FloatingPanelKind::AgentTooltip { label: "Add New Agent".to_string() },
                            x: 52.0,
                            y: (add_button_rect().1 + add_button_rect().3) / 2.0 - 12.0,
                            width: 0.0,
                            height: 24.0,
                        });
                    },
                    onmouseleave: move |_| {
                        add_hovered.set(false);
                        if !floating_dock.read().is_open("add-agent-popover") {
                            floating_dock.write().close("add-agent-tooltip");
                        }
                    },
                    button {
                        class: "flex h-9 w-9 items-center justify-center rounded-sm border border-dashed border-border bg-background-elevated text-muted-foreground hover:border-primary hover:text-foreground transition-opacity duration-150 {add_button_opacity}",
                        onclick: move |_| {
                            floating_dock.write().close("add-agent-tooltip");
                            let (_, y0, _, _) = add_button_rect();
                            floating_dock.write().open(FloatingPanel {
                                id: "add-agent-popover".to_string(),
                                kind: FloatingPanelKind::AddAgentPopover,
                                x: 52.0,
                                y: y0,
                                width: 240.0,
                                height: 180.0,
                            });
                        },
                        Icon { width: 20, height: 20, icon: LdPlus }
                    }
                }
            }
        }
    }
}
