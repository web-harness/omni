use dioxus::prelude::*;
use omni_rt::protocol::Thread;

pub struct ThreadContext {
    pub thread: Thread,
}

#[derive(Clone, Copy)]
pub struct ThreadContextProvider {
    pub thread: Signal<Option<Thread>>,
}

impl ThreadContextProvider {
    pub fn new(thread: Thread) -> Self {
        Self {
            thread: Signal::new(Some(thread)),
        }
    }
}
