use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdBrain;
use dioxus_free_icons::Icon;

use crate::lib::{AgentEndpoint, AgentEndpointState};

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
pub fn AgentRail() -> Element {
    let mut agent_state = use_context::<Signal<AgentEndpointState>>();
    let mut hovered_item = use_signal(|| None::<String>);

    let (ordered_endpoints, active_agent_id, dicebear_style) = {
        let s = agent_state.read();
        (
            s.ordered()
                .into_iter()
                .cloned()
                .collect::<Vec<AgentEndpoint>>(),
            s.active_agent_id.clone(),
            s.dicebear_style.clone(),
        )
    };

    let hovered = hovered_item();

    rsx! {
        dock-wrapper {
            class: "agent-dock",
            direction: "vertical",
            position: "left",
            size: "36",
            gap: "4",
            padding: "6",
            "max-range": "150",
            "max-scale": "1.8",

            for endpoint in ordered_endpoints.iter() {
                {
                    let id = endpoint.id.clone();
                    let removable = endpoint.removable;
                    let is_active = if removable {
                        active_agent_id.as_deref() == Some(id.as_str())
                    } else {
                        active_agent_id.is_none()
                    };
                    let is_dimmed = hovered.as_ref().map(|h| h != &id).unwrap_or(false);
                    let opacity = if is_dimmed { "opacity-50" } else { "opacity-100" };
                    let btn_cls = if is_active {
                        "relative flex h-full w-full items-center justify-center overflow-hidden rounded-lg border border-primary bg-background-elevated ring-2 ring-primary/80 transition-opacity duration-150"
                    } else {
                        "relative flex h-full w-full items-center justify-center overflow-hidden rounded-lg border border-border bg-background-elevated hover:border-primary/50 hover:bg-background-interactive transition-opacity duration-150"
                    };
                    let initials = agent_initials(&endpoint.name);
                    let style = dicebear_style.clone();
                    let ep_id = endpoint.id.clone();
                    let id_enter = id.clone();
                    let id_leave = id.clone();
                    let id_activate = id.clone();

                    rsx! {
                        dock-item {
                            key: "{ep_id}",
                            div {
                                class: "dock-btn-wrap",
                                onmouseenter: move |_| {
                                    hovered_item.set(Some(id_enter.clone()));
                                },
                                onmouseleave: move |_| {
                                    if hovered_item.read().as_deref() == Some(id_leave.as_str()) {
                                        hovered_item.set(None);
                                    }
                                },
                                button {
                                    class: "{btn_cls} {opacity}",
                                    onclick: move |_| {
                                        if removable {
                                            agent_state.write().active_agent_id = Some(id_activate.clone());
                                        } else {
                                            agent_state.write().active_agent_id = None;
                                        }
                                    },
                                    if removable {
                                        div {
                                            class: "absolute inset-0 flex items-center justify-center bg-gradient-to-br from-status-info/25 via-primary/15 to-background-elevated text-[11px] font-semibold text-foreground",
                                            "{initials}"
                                        }
                                        omni-dicebear {
                                            class: "relative z-10 block h-full w-full",
                                            seed: "{ep_id}",
                                            "avatar-style": "{style}",
                                            size: "36",
                                        }
                                    } else {
                                        Icon { width: 20, height: 20, icon: LdBrain, class: "text-primary" }
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
