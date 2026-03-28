use dioxus::prelude::*;
use omni_rt::protocol::{Message, Thread};

pub struct GlobalState {
    pub threads: Vec<Thread>,
    pub messages: Vec<Message>,
}

pub mod store {
    use super::*;

    pub struct Store {
        pub threads: Signal<Vec<Thread>>,
        pub messages: Signal<Vec<Message>>,
    }

    impl Store {
        pub fn new() -> Self {
            Self {
                threads: Signal::new(vec![]),
                messages: Signal::new(vec![]),
            }
        }
    }

    impl Default for Store {
        fn default() -> Self {
            Self::new()
        }
    }
}

pub mod thread_context;
pub mod utils;

pub use store::Store;
pub use thread_context::ThreadContext;
