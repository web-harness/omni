use crate::lib::{AppState, Role, StreamEvent, UiMessage};

pub fn apply_stream_event(state: &mut AppState, event: StreamEvent) {
    match event {
        StreamEvent::Token(token) => {
            state.stream_buffer.push_str(&token);
        }
        StreamEvent::ToolCall(call) => {
            if let Some(thread_id) = state.current_thread_id().map(ToOwned::to_owned) {
                state.tool_calls.entry(thread_id).or_default().push(call);
            }
        }
        StreamEvent::ToolResult(result) => {
            if let Some(thread_id) = state.current_thread_id().map(ToOwned::to_owned) {
                state
                    .tool_results
                    .entry(thread_id)
                    .or_default()
                    .push(result);
            }
        }
        StreamEvent::Todos(todos) => {
            if let Some(thread_id) = state.current_thread_id().map(ToOwned::to_owned) {
                state.todos.insert(thread_id, todos);
            }
        }
        StreamEvent::Done => {
            if let Some(thread_id) = state.current_thread_id().map(ToOwned::to_owned) {
                if !state.stream_buffer.is_empty() {
                    let message = UiMessage {
                        id: format!("asst-{}", state.messages_for_active().len()),
                        role: Role::Assistant,
                        content: state.stream_buffer.clone(),
                    };
                    state.messages.entry(thread_id).or_default().push(message);
                }
            }
            state.stream_buffer.clear();
            state.is_streaming = false;
        }
        StreamEvent::Error(err) => {
            state.error = Some(err);
            state.is_streaming = false;
        }
    }
}
