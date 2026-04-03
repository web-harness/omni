#![cfg(target_arch = "wasm32")]

use crate::lib::{ChatState, Role, StreamEvent, TasksState, UiMessage};

pub fn apply_stream_event(
    active_thread_id: Option<&str>,
    chat: &mut ChatState,
    tasks: &mut TasksState,
    event: StreamEvent,
) {
    match event {
        StreamEvent::Token(token) => {
            chat.stream_buffer.push_str(&token);
        }
        StreamEvent::ToolCall(call) => {
            if let Some(thread_id) = active_thread_id {
                tasks
                    .tool_calls
                    .entry(thread_id.to_owned())
                    .or_default()
                    .push(call);
            }
        }
        StreamEvent::ToolResult(result) => {
            if let Some(thread_id) = active_thread_id {
                tasks
                    .tool_results
                    .entry(thread_id.to_owned())
                    .or_default()
                    .push(result);
            }
        }
        StreamEvent::Todos(todos) => {
            if let Some(thread_id) = active_thread_id {
                tasks.todos.insert(thread_id.to_owned(), todos);
            }
        }
        StreamEvent::Done => {
            if let Some(thread_id) = active_thread_id {
                if !chat.stream_buffer.is_empty() {
                    let message = UiMessage {
                        id: format!("asst-{}", chat.messages_for(thread_id).len()),
                        role: Role::Assistant,
                        content: chat.stream_buffer.clone(),
                    };
                    chat.messages
                        .entry(thread_id.to_owned())
                        .or_default()
                        .push(message);
                }
            }
            chat.stream_buffer.clear();
            chat.is_streaming = false;
        }
        StreamEvent::Error(err) => {
            chat.error = Some(err);
            chat.is_streaming = false;
        }
    }
}
