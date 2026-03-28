use dioxus::prelude::*;

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(crate::components::Layout)]
    #[route("/")]
    Home {},
    #[route("/thread/:id")]
    ThreadView { id: String },
    #[route("/board")]
    Board {},
    #[route("/settings")]
    Settings {},
}
