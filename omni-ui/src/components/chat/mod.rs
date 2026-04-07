use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdBot, LdChevronDown, LdChevronRight, LdFolder, LdListTodo, LdSend, LdSquare, LdTrash2, LdUser,
};
use dioxus_free_icons::Icon;
use futures_util::StreamExt;
use omni_rt::deepagents::model_registry::browser_model_spec;

use crate::components::ui::{Badge, BadgeVariant, Popover};
use crate::lib::thread_context::apply_stream_event;
use crate::lib::utils::api_url;
use crate::lib::{
    AgentEndpoint, AgentEndpointState, ChatState, ModelState, Role, TasksState, ThreadState,
    ToolCall, ToolResult, UiState, WorkspaceState,
};

#[derive(Clone)]
struct StreamRequest {
    thread_id: String,
    input: String,
    model_id: String,
    endpoint: Option<AgentEndpoint>,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[derive(serde::Serialize)]
struct DesktopAgentModuleRequest {
    body: serde_json::Value,
    #[serde(rename = "baseUrl")]
    base_url: String,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[derive(serde::Deserialize)]
struct DesktopAgentModuleEvent {
    event: String,
    data: serde_json::Value,
}

fn send_stream_request(
    stream: &Coroutine<StreamRequest>,
    thread_id: String,
    input: String,
    model_id: String,
    endpoint: Option<AgentEndpoint>,
) {
    stream.send(StreamRequest {
        thread_id,
        input,
        model_id,
        endpoint,
    });
}

fn browser_download_bytes(
    model_id: &str,
    status: &crate::lib::BrowserInferenceStatus,
) -> Option<(u64, u64)> {
    if status.download.phase != crate::lib::BrowserDownloadPhase::Downloading {
        return None;
    }

    if status.download.model_id.as_deref() != Some(model_id) {
        return None;
    }

    let spec = browser_model_spec(model_id)?;
    let loaded_bytes = status.download.loaded_bytes.unwrap_or(0).min(spec.size);
    Some((loaded_bytes, spec.size))
}

fn browser_download_segment_label(
    model_id: &str,
    status: &crate::lib::BrowserInferenceStatus,
) -> Option<String> {
    let (loaded_bytes, total_bytes) = browser_download_bytes(model_id, status)?;
    let spec = browser_model_spec(model_id)?;
    let completed_segments = if loaded_bytes >= total_bytes {
        spec.mirror_parts
    } else {
        ((loaded_bytes as u128 * spec.mirror_parts as u128) / total_bytes as u128) as u16
    };

    Some(format!(
        "{}/{}",
        completed_segments.min(spec.mirror_parts),
        spec.mirror_parts
    ))
}

fn browser_download_progress_percent(
    model_id: &str,
    status: &crate::lib::BrowserInferenceStatus,
) -> Option<u8> {
    let (loaded_bytes, total_bytes) = browser_download_bytes(model_id, status)?;
    if total_bytes == 0 {
        return Some(0);
    }

    Some(((loaded_bytes as u128 * 100) / total_bytes as u128).min(100) as u8)
}

fn stream_request_body(req: &StreamRequest) -> serde_json::Value {
    serde_json::json!({
        "thread_id": req.thread_id,
        "messages": [
            {
                "role": "user",
                "content": req.input,
            }
        ],
        "stream_mode": ["messages", "values"],
        "metadata": {
            "model_id": req.model_id,
            "agent_id": req.endpoint.as_ref().map(|endpoint| endpoint.id.clone()).unwrap_or_else(|| "main".to_string()),
            "agent_name": req.endpoint.as_ref().map(|endpoint| endpoint.name.clone()).unwrap_or_else(|| "Main Agent".to_string()),
            "agent_url": req.endpoint.as_ref().map(|endpoint| endpoint.url.clone()).unwrap_or_default(),
            "agent_bearer_token": req.endpoint.as_ref().map(|endpoint| endpoint.bearer_token.clone()).unwrap_or_default(),
            "agent_mode": if req.endpoint.as_ref().map(|endpoint| endpoint.removable).unwrap_or(false) { "direct" } else { "main" },
        },
    })
}

fn apply_sse_event(
    req: &StreamRequest,
    thread_state: &Signal<ThreadState>,
    chat_state: &mut Signal<ChatState>,
    tasks_state: &mut Signal<TasksState>,
    event: omni_rt::deepagents::sse::SseEvent,
) -> bool {
    match event {
        omni_rt::deepagents::sse::SseEvent::Message(value) => {
            let content = value
                .get("content")
                .and_then(|content| content.as_str())
                .map(str::to_string)
                .unwrap_or_else(|| value.to_string());
            if !content.is_empty() {
                let active_tid = thread_state.read().active_thread_id.clone();
                apply_stream_event(
                    active_tid.as_deref(),
                    &mut chat_state.write(),
                    &mut tasks_state.write(),
                    crate::lib::StreamEvent::Token(content),
                );
            }
            false
        }
        omni_rt::deepagents::sse::SseEvent::MessageComplete(_) => false,
        omni_rt::deepagents::sse::SseEvent::Values(_) => false,
        omni_rt::deepagents::sse::SseEvent::Done => {
            apply_stream_event(
                Some(&req.thread_id),
                &mut chat_state.write(),
                &mut tasks_state.write(),
                crate::lib::StreamEvent::Done,
            );
            true
        }
        omni_rt::deepagents::sse::SseEvent::Error(error) => {
            chat_state.write().error = Some(error);
            chat_state.write().is_streaming = false;
            true
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
async fn run_desktop_stream_via_eval(
    req: &StreamRequest,
    thread_state: &Signal<ThreadState>,
    chat_state: &mut Signal<ChatState>,
    tasks_state: &mut Signal<TasksState>,
) {
    let mut eval = document::eval(
        r#"
        const payload = await dioxus.recv();
        const module = await import(`${payload.baseUrl}/omni-agent-module.js`);
        const events = await module.executeRunStream(payload.body, payload.baseUrl);
        for await (const event of events) {
            dioxus.send(event);
        }
        dioxus.send({ event: "end", data: null });
        "#,
    );

    if let Err(error) = eval.send(DesktopAgentModuleRequest {
        body: stream_request_body(req),
        base_url: api_url("").trim_end_matches('/').to_string(),
    }) {
        chat_state.write().error = Some(error.to_string());
        chat_state.write().is_streaming = false;
        return;
    }

    loop {
        match eval.recv::<DesktopAgentModuleEvent>().await {
            Ok(frame) => {
                let event = match frame.event.as_str() {
                    "message" | "messages/partial" => {
                        omni_rt::deepagents::sse::SseEvent::Message(frame.data)
                    }
                    "messages/complete" => {
                        omni_rt::deepagents::sse::SseEvent::MessageComplete(frame.data)
                    }
                    "values" => omni_rt::deepagents::sse::SseEvent::Values(frame.data),
                    "error" => omni_rt::deepagents::sse::SseEvent::Error(
                        frame
                            .data
                            .get("message")
                            .and_then(|value| value.as_str())
                            .map(str::to_string)
                            .unwrap_or_else(|| frame.data.to_string()),
                    ),
                    "end" => omni_rt::deepagents::sse::SseEvent::Done,
                    _ => continue,
                };

                if apply_sse_event(req, thread_state, chat_state, tasks_state, event) {
                    break;
                }
            }
            Err(error) => {
                chat_state.write().error = Some(error.to_string());
                chat_state.write().is_streaming = false;
                break;
            }
        }
    }
}

#[component]
pub fn ChatContainer(thread_id: String) -> Element {
    let stream = {
        let thread_state = use_context::<Signal<ThreadState>>();
        let mut chat_state = use_context::<Signal<ChatState>>();
        let mut tasks_state = use_context::<Signal<TasksState>>();

        use_coroutine(move |mut rx: UnboundedReceiver<StreamRequest>| async move {
            while let Some(req) = rx.next().await {
                chat_state.write().is_streaming = true;
                chat_state.write().error = None;

                #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
                {
                    run_desktop_stream_via_eval(
                        &req,
                        &thread_state,
                        &mut chat_state,
                        &mut tasks_state,
                    )
                    .await;
                    continue;
                }

                #[cfg(any(target_arch = "wasm32", not(feature = "desktop")))]
                match omni_rt::deepagents::sse::SseStream::connect(
                    &api_url("runs/stream"),
                    &stream_request_body(&req).to_string(),
                )
                .await
                {
                    Ok(mut stream) => loop {
                        match stream.next_event().await {
                            Ok(Some(event)) => {
                                if apply_sse_event(
                                    &req,
                                    &thread_state,
                                    &mut chat_state,
                                    &mut tasks_state,
                                    event,
                                ) {
                                    break;
                                }
                            }
                            Err(e) => {
                                chat_state.write().error = Some(e.to_string());
                                chat_state.write().is_streaming = false;
                                break;
                            }
                            Ok(None) => {
                                chat_state.write().is_streaming = false;
                                break;
                            }
                        }
                    },
                    Err(e) => {
                        chat_state.write().error = Some(e.to_string());
                        chat_state.write().is_streaming = false;
                    }
                }
            }
        })
    };

    let chat_state = use_context::<Signal<ChatState>>();
    let tasks_state = use_context::<Signal<TasksState>>();
    let thread_state = use_context::<Signal<ThreadState>>();
    let tid = thread_state
        .read()
        .active_thread_id
        .clone()
        .unwrap_or_default();
    let messages = chat_state.read().messages_for(&tid);
    let tool_calls = tasks_state.read().tool_calls_for(&tid);
    let tool_results = tasks_state.read().tool_results_for(&tid);

    rsx! {
        div { class: "flex h-full flex-col",
            div { class: "min-h-0 flex-1 overflow-auto px-4 py-4",
                div { class: "mx-auto flex w-full max-w-3xl flex-col gap-3",
                    if messages.is_empty() && tool_calls.is_empty() {
                        div { class: "rounded-sm border border-border bg-background-elevated p-4 text-center",
                            omni-text { "data-text": "NEW THREAD", "data-strategy": "none", "data-max-lines": "1", class: "text-xs font-semibold text-muted-foreground" }
                            omni-text { "data-text": "Pick workspace, choose model, and issue your first task.", "data-strategy": "none", "data-max-lines": "2", class: "mt-2 text-sm" }
                        }
                    }
                    for msg in &messages {
                        MessageBubble { key: "{msg.id}", message: msg.clone() }
                    }
                    for call in tool_calls {
                        {
                            let result = tool_results.iter().find(|r| r.tool_call_id == call.id).cloned();
                            rsx! { ToolCallRenderer { key: "{call.id}", call, result } }
                        }
                    }
                    if chat_state.read().is_streaming {
                        div { class: "rounded-sm border border-border bg-background p-3 text-[11px]",
                            omni-text { "data-text": "Agent is working...", "data-strategy": "none", "data-max-lines": "1", class: "mb-1 text-muted-foreground" }
                            pre { class: "whitespace-pre-wrap", "{chat_state.read().stream_buffer}" }
                        }
                    }
                    if let Some(err) = chat_state.read().error.clone() {
                        div { class: "rounded-sm border border-status-critical bg-status-critical/10 p-2 text-[11px] text-status-critical",
                            omni-text { "data-text": "{err}", "data-strategy": "none", "data-max-lines": "4" }
                        }
                    }
                }
            }
            ChatInput { thread_id, stream }
        }
    }
}

#[component]
pub fn MessageBubble(message: crate::lib::UiMessage) -> Element {
    let user = message.role == Role::User;
    let bubble_class = if user {
        "rounded-sm p-3 text-[12px] leading-5 bg-primary/10"
    } else {
        "rounded-sm p-3 text-[12px] leading-5 bg-card"
    };
    let label = if user { "YOU" } else { "AGENT" };

    rsx! {
        div { class: "flex gap-3 overflow-hidden",
            if !user {
                div { class: "mt-1 h-7 w-7 shrink-0 rounded-full bg-status-info/15 flex items-center justify-center",
                    Icon { class: "text-status-info", width: 14, height: 14, icon: LdBot }
                }
            } else {
                div { class: "w-7" }
            }
            div { class: "min-w-0 flex-1",
                omni-text { "data-text": "{label}", "data-strategy": "none", "data-max-lines": "1", class: "mb-1 text-[10px] font-semibold text-muted-foreground" }
                div { class: "{bubble_class}",
                    if user {
                        pre { class: "whitespace-pre-wrap font-sans text-[12px]", "{message.content}" }
                    } else {
                        omni-marked {
                            class: "block w-full text-[12px] [&_.markdown-body]:min-h-0 [&_.markdown-body]:bg-transparent [&_.markdown-body]:p-0 [&_.markdown-body_pre]:mb-0 [&_.markdown-body_pre]:mt-2 [&_.markdown-body_p:last-child]:mb-0",
                            "data-value": "{message.content}",
                            "data-readonly": "true",
                        }
                    }
                }
            }
            if user {
                div { class: "mt-1 h-7 w-7 shrink-0 rounded-full bg-primary/15 flex items-center justify-center",
                    Icon { class: "text-primary", width: 14, height: 14, icon: LdUser }
                }
            } else {
                div { class: "w-7" }
            }
        }
    }
}

#[component]
pub fn ToolCallRenderer(call: ToolCall, result: Option<ToolResult>) -> Element {
    match call.name.as_str() {
        "update_todos" => rsx! { UpdateTodosRenderer { call, result } },
        "dispatch_subagent" => rsx! { BackgroundTaskRenderer { call } },
        _ => rsx! { GenericToolCallRenderer { call, result } },
    }
}

#[component]
fn UpdateTodosRenderer(call: ToolCall, result: Option<ToolResult>) -> Element {
    let mut open = use_signal(|| true);

    let todos: Vec<(String, String)> = call
        .args
        .get("todos")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|item| {
                    let content = item
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let status = item
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("pending")
                        .to_string();
                    (content, status)
                })
                .collect()
        })
        .unwrap_or_default();

    let has_in_progress = todos.iter().any(|(_, status)| status == "in_progress");
    let has_pending = todos.iter().any(|(_, status)| status == "pending");
    let is_done = !todos.is_empty() && todos.iter().all(|(_, status)| status == "completed");
    let is_synced = result.as_ref().is_some_and(|item| !item.is_error);
    let is_error = result.as_ref().is_some_and(|item| item.is_error);

    rsx! {
        div { class: "rounded-sm border border-border bg-background-elevated text-[11px] overflow-hidden",
            button {
                class: "flex w-full items-center gap-2 px-3 py-2 text-left hover:bg-background-interactive",
                onclick: move |_| open.set(!open()),
                if open() {
                    Icon { width: 10, height: 10, icon: LdChevronDown, class: "text-muted-foreground shrink-0" }
                } else {
                    Icon { width: 10, height: 10, icon: LdChevronRight, class: "text-muted-foreground shrink-0" }
                }
                Icon { width: 12, height: 12, icon: LdListTodo, class: "text-muted-foreground shrink-0" }
                omni-text { "data-text": "Update Tasks", "data-strategy": "none", "data-max-lines": "1", class: "font-semibold" }
                div { class: "ml-auto flex items-center gap-1",
                    if is_error {
                        Badge { variant: BadgeVariant::Critical, "ERROR" }
                    } else if has_in_progress {
                        Badge { variant: BadgeVariant::Info, "IN PROGRESS" }
                    } else if is_done {
                        Badge { variant: BadgeVariant::Nominal, "DONE" }
                        if is_synced {
                            Badge { variant: BadgeVariant::Info, "SYNCED" }
                        }
                    } else if has_pending {
                        Badge { variant: BadgeVariant::Warning, "PENDING" }
                        if is_synced {
                            Badge { variant: BadgeVariant::Info, "SYNCED" }
                        }
                    } else if is_synced {
                        Badge { variant: BadgeVariant::Info, "SYNCED" }
                    } else {
                        Badge { variant: BadgeVariant::Info, "RUNNING" }
                    }
                }
            }
            if open() {
                div { class: "px-3 pb-3 space-y-1",
                    for (content, status) in todos {
                        div { class: "flex items-start gap-2 py-0.5",
                            if status == "in_progress" {
                                div { class: "mt-0.5 h-3.5 w-3.5 shrink-0 rounded-full border-2 border-status-info bg-status-info/30 flex items-center justify-center",
                                    div { class: "h-1.5 w-1.5 rounded-full bg-status-info" }
                                }
                            } else if status == "completed" {
                                div { class: "mt-0.5 h-3.5 w-3.5 shrink-0 rounded-full bg-status-nominal flex items-center justify-center",
                                    div { class: "h-1 w-2 border-b border-r border-white rotate-45 mb-0.5" }
                                }
                            } else {
                                div { class: "mt-0.5 h-3.5 w-3.5 shrink-0 rounded-full border border-border" }
                            }
                            omni-text {
                                "data-text": "{content}",
                                "data-strategy": "truncate",
                                "data-max-lines": "2",
                                class: "text-[11px] text-foreground leading-5",
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BackgroundTaskRenderer(call: ToolCall) -> Element {
    let mut open = use_signal(|| false);
    let task = call
        .args
        .get("task")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    rsx! {
        div { class: "rounded-sm border border-border bg-background-elevated text-[11px] overflow-hidden",
            button {
                class: "flex w-full items-center gap-2 px-3 py-2 text-left hover:bg-background-interactive",
                onclick: move |_| open.set(!open()),
                if open() {
                    Icon { width: 10, height: 10, icon: LdChevronDown, class: "text-muted-foreground shrink-0" }
                } else {
                    Icon { width: 10, height: 10, icon: LdChevronRight, class: "text-muted-foreground shrink-0" }
                }
                Icon { width: 12, height: 12, icon: LdBot, class: "text-status-info shrink-0" }
                omni-text { "data-text": "Background Task", "data-strategy": "none", "data-max-lines": "1", class: "font-semibold" }
            }
            div { class: "px-3 pb-3 pt-1 text-muted-foreground",
                if open() {
                    omni-text {
                        "data-text": "{task}",
                        "data-strategy": "none",
                        "data-max-lines": "20",
                        class: "text-[11px]",
                    }
                } else {
                    omni-text {
                        "data-text": "{task}",
                        "data-strategy": "truncate",
                        "data-max-lines": "2",
                        class: "text-[11px]",
                    }
                }
            }
        }
    }
}

#[component]
fn GenericToolCallRenderer(call: ToolCall, result: Option<ToolResult>) -> Element {
    let mut open = use_signal(|| false);
    let is_done = result.as_ref().map(|r| !r.is_error).unwrap_or(false);
    let is_err = result.as_ref().map(|r| r.is_error).unwrap_or(false);

    rsx! {
        div { class: "rounded-sm border border-border bg-background-elevated text-[11px] overflow-hidden",
            button {
                class: "flex w-full items-center gap-2 px-3 py-2 text-left hover:bg-background-interactive",
                onclick: move |_| open.set(!open()),
                if open() {
                    Icon { width: 10, height: 10, icon: LdChevronDown, class: "text-muted-foreground shrink-0" }
                } else {
                    Icon { width: 10, height: 10, icon: LdChevronRight, class: "text-muted-foreground shrink-0" }
                }
                omni-text { "data-text": "{call.name}", "data-strategy": "truncate", "data-max-lines": "1", class: "font-semibold font-mono" }
                div { class: "ml-auto flex items-center gap-1",
                    if is_done { Badge { variant: BadgeVariant::Nominal, "OK" } }
                    if is_err { Badge { variant: BadgeVariant::Critical, "ERROR" } }
                    if result.is_none() { Badge { variant: BadgeVariant::Info, "RUNNING" } }
                }
            }
            if open() {
                pre { class: "px-3 pb-3 whitespace-pre-wrap text-muted-foreground", "{call.args}" }
            }
        }
    }
}

#[component]
fn ChatInput(thread_id: String, stream: Coroutine<StreamRequest>) -> Element {
    let mut chat_state = use_context::<Signal<ChatState>>();
    let thread_state = use_context::<Signal<ThreadState>>();
    let model_state = use_context::<Signal<ModelState>>();
    let agent_state = use_context::<Signal<AgentEndpointState>>();

    #[cfg(target_arch = "wasm32")]
    let sw_ready = {
        let global = js_sys::global();
        js_sys::Reflect::get(&global, &"__omni_sw_ready".into())
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    };
    #[cfg(not(target_arch = "wasm32"))]
    let sw_ready = true;

    rsx! {
        div { class: "border-t border-border px-4 py-3",
            div { class: "mx-auto w-full max-w-3xl",
                div { class: "flex items-center gap-2 rounded-sm border border-border bg-background px-3 py-2",
                    input {
                        class: "flex-1 bg-transparent text-[12px] outline-none placeholder:text-muted-foreground",
                        placeholder: "Message...",
                        value: "{chat_state.read().input_draft}",
                        oninput: move |evt: Event<FormData>| chat_state.write().input_draft = evt.value(),
                        onkeydown: {
                            let thread_id = thread_id.clone();
                            move |evt: Event<KeyboardData>| {
                                if evt.key() == Key::Enter && !evt.modifiers().contains(Modifiers::SHIFT) {
                                    let input = chat_state.read().input_draft.trim().to_string();
                                    if input.is_empty() { return; }
                                    if !sw_ready {
                                        chat_state.write().error = Some("Service worker is not ready yet. Please wait a moment and retry.".to_string());
                                        return;
                                    }
                                    let active_id = thread_state.read().active_thread_id.clone();
                                    if let Some(active_id) = active_id {
                                        {
                                            let mut write = chat_state.write();
                                            let msg_count = write.messages.get(&active_id).map(|v| v.len()).unwrap_or(0);
                                            write.messages.entry(active_id.clone()).or_default().push(crate::lib::UiMessage {
                                                id: format!("u-{}", msg_count + 1),
                                                role: Role::User,
                                                content: input.clone(),
                                            });
                                            write.input_draft.clear();
                                            write.stream_buffer.clear();
                                        }
                                        send_stream_request(
                                            &stream,
                                            thread_id.clone(),
                                            input,
                                            model_state.read().selected_model_for(&active_id),
                                            agent_state.read().active_endpoint().cloned(),
                                        );
                                    }
                                }
                            }
                        },
                    }
                    button {
                        class: "shrink-0 rounded bg-primary p-1.5 text-primary-foreground hover:opacity-90 disabled:opacity-50",
                        disabled: chat_state.read().input_draft.trim().is_empty() || chat_state.read().is_streaming,
                        onclick: move |_| {
                            let input = chat_state.read().input_draft.trim().to_string();
                            if input.is_empty() { return; }
                            if !sw_ready {
                                chat_state.write().error = Some("Service worker is not ready yet. Please wait a moment and retry.".to_string());
                                return;
                            }
                            let active_id = thread_state.read().active_thread_id.clone();
                            if let Some(active_id) = active_id {
                                {
                                    let mut write = chat_state.write();
                                    let msg_count = write.messages.get(&active_id).map(|v| v.len()).unwrap_or(0);
                                    write.messages.entry(active_id.clone()).or_default().push(crate::lib::UiMessage {
                                        id: format!("u-{}", msg_count + 1),
                                        role: Role::User,
                                        content: input.clone(),
                                    });
                                    write.input_draft.clear();
                                    write.stream_buffer.clear();
                                }
                                send_stream_request(
                                    &stream,
                                    thread_id.clone(),
                                    input,
                                    model_state.read().selected_model_for(&active_id),
                                    agent_state.read().active_endpoint().cloned(),
                                );
                            }
                        },
                        Icon { width: 13, height: 13, icon: LdSend }
                    }
                }
                div { class: "mt-2 flex items-center gap-2",
                    ModelSwitcher {}
                    WorkspacePicker {}
                    omni-text { "data-text": "~2.4k input · ~580 output · $0.012", "data-strategy": "none", "data-max-lines": "1", class: "ml-auto text-[10px] text-muted-foreground whitespace-nowrap" }
                }
            }
        }
    }
}

#[component]
pub fn ModelSwitcher() -> Element {
    let mut model_state = use_context::<Signal<ModelState>>();
    let mut ui_state = use_context::<Signal<UiState>>();
    let agent_state = use_context::<Signal<crate::lib::AgentEndpointState>>();
    let thread_state = use_context::<Signal<ThreadState>>();
    let mut open = use_signal(|| false);

    let tid = thread_state
        .read()
        .active_thread_id
        .clone()
        .unwrap_or_default();
    let providers = model_state.read().providers.clone();
    let models = model_state.read().models.clone();
    let selected_model = model_state.read().selected_model_for(&tid);
    let browser_status = model_state.read().browser_inference.clone();
    let selected_model_config = models
        .iter()
        .find(|model| model.id == selected_model)
        .cloned();
    let locked_agent = agent_state
        .read()
        .active_endpoint()
        .filter(|endpoint| endpoint.removable)
        .cloned();
    let locked_agent_for_close_effect = locked_agent.clone();
    let locked_agent_for_render = locked_agent.clone();
    let initial_provider = selected_model_config
        .as_ref()
        .map(|model| model.provider.clone())
        .unwrap_or(crate::lib::ProviderId::Anthropic);
    let selected_label = selected_model_config
        .as_ref()
        .map(|model| model.name.clone())
        .unwrap_or_else(|| selected_model.clone());
    let mut selected_provider = use_signal(|| initial_provider.clone());
    let mut pending_download = use_signal(|| None::<crate::lib::ModelConfig>);
    let mut pending_delete = use_signal(|| None::<crate::lib::ModelConfig>);
    let mut active_download_model_id = use_signal(|| None::<String>);
    let browser_status_for_download_effect = browser_status.clone();
    let mut _browser_status_subscription =
        use_signal(|| None::<crate::lib::sw_api::BrowserInferenceStatusSubscription>);

    let filtered_models: Vec<_> = models
        .iter()
        .filter(|m| m.provider == selected_provider())
        .cloned()
        .collect();

    let selected_model_for_effect = selected_model_config.clone();
    let selected_model_for_trigger = selected_model_config.clone();
    let tid_for_download_effect = tid.clone();

    use_effect(move || {
        if locked_agent_for_close_effect.is_none() {
            return;
        }

        pending_download.set(None);
        pending_delete.set(None);
        if open() {
            open.set(false);
        }
    });

    use_effect(move || {
        if let Some(model) = selected_model_for_effect.clone() {
            if selected_provider() != model.provider {
                selected_provider.set(model.provider);
            }
        }
    });

    use_effect(move || {
        let mut model_state_for_status = model_state;
        spawn(async move {
            if let Ok(status) = crate::lib::sw_api::get_browser_inference_status().await {
                model_state_for_status.write().browser_inference = status;
            }
        });

        #[cfg(target_arch = "wasm32")]
        let subscription = crate::lib::sw_api::subscribe_browser_inference_status({
            let mut model_state_for_status = model_state;
            move |status| {
                model_state_for_status.write().browser_inference = status;
            }
        })
        .ok();

        #[cfg(target_arch = "wasm32")]
        _browser_status_subscription.set(subscription);
    });

    use_effect(move || {
        let Some(model_id) = active_download_model_id() else {
            return;
        };

        let download_model_id = browser_status_for_download_effect.download.model_id.clone();
        let download_phase = browser_status_for_download_effect.download.phase.clone();

        if download_model_id.as_deref() != Some(model_id.as_str()) {
            if download_phase != crate::lib::BrowserDownloadPhase::Downloading {
                active_download_model_id.set(None);
            }
            return;
        }

        match download_phase {
            crate::lib::BrowserDownloadPhase::Completed => {
                model_state
                    .write()
                    .selected_model
                    .insert(tid_for_download_effect.clone(), model_id.clone());
                active_download_model_id.set(None);
                open.set(false);

                #[cfg(target_arch = "wasm32")]
                spawn(async move {
                    let _ = crate::lib::sw_api::set_default_model(&model_id).await;
                });
            }
            crate::lib::BrowserDownloadPhase::Error | crate::lib::BrowserDownloadPhase::Idle => {
                active_download_model_id.set(None);
            }
            crate::lib::BrowserDownloadPhase::Downloading => {}
        }
    });

    #[cfg(target_arch = "wasm32")]
    fn provider_prefix(provider: &crate::lib::ProviderId) -> &'static str {
        match provider {
            crate::lib::ProviderId::Anthropic => "anthropic",
            crate::lib::ProviderId::OpenAI => "openai",
            crate::lib::ProviderId::Google => "google",
            crate::lib::ProviderId::Ollama => "ollama",
            crate::lib::ProviderId::Browser => "browser",
        }
    }

    rsx! {
        if let Some(agent) = locked_agent_for_render {
            button {
                disabled: true,
                class: "flex w-[clamp(120px,18vw,180px)] max-w-full cursor-default items-center gap-1 rounded-sm border border-border px-2 py-1 text-[11px] text-muted-foreground opacity-70",
                omni-text { "data-text": "{agent.name}", "data-strategy": "shrink-truncate", "data-max-lines": "1", "data-min-size": "9", class: "min-w-0 flex-1 overflow-hidden whitespace-nowrap" }
                span { class: "h-[10px] w-[10px] shrink-0" }
            }
        } else {
            Popover {
                open: open(),
                on_close: move |_| {
                    pending_download.set(None);
                    pending_delete.set(None);
                    open.set(false);
                },
                trigger: rsx! {
                    button {
                        class: "flex w-[clamp(120px,18vw,180px)] max-w-full cursor-pointer items-center gap-1 rounded-sm border border-border px-2 py-1 text-[11px] text-muted-foreground hover:bg-background-interactive",
                        onclick: move |_| {
                            if let Some(model) = selected_model_for_trigger.clone() {
                                selected_provider.set(model.provider);
                            }
                            open.set(!open());
                        },
                        omni-text { "data-text": "{selected_label}", "data-strategy": "shrink-truncate", "data-max-lines": "1", "data-min-size": "9", class: "min-w-0 flex-1 overflow-hidden whitespace-nowrap" }
                        Icon { width: 10, height: 10, icon: LdChevronDown }
                    }
                },
                if let Some(model) = pending_download() {
                {
                    let download_url = browser_model_spec(&model.id)
                        .map(|details| details.download_url())
                        .unwrap_or_default();
                    let size_label = browser_model_spec(&model.id)
                        .map(|details| crate::lib::utils::fmt_size(details.size))
                        .unwrap_or_default();
                    let source_label = browser_model_spec(&model.id)
                        .map(|details| details.source_label())
                        .unwrap_or_default();
                    let model_for_download = model.clone();
                    let download_phase = browser_status.download.phase.clone();
                    rsx! {
                        div { class: "space-y-3",
                            div {
                                omni-text { "data-text": "Download {model.name}", "data-strategy": "none", "data-max-lines": "1", class: "text-[11px] font-semibold text-foreground" }
                                omni-text { "data-text": "Browser inference caches the model locally before use.", "data-strategy": "none", "data-max-lines": "2", class: "mt-1 text-[10px] text-muted-foreground" }
                            }
                            div { class: "space-y-1 rounded-sm border border-border bg-background px-3 py-2 text-[10px]",
                                div { class: "flex items-center justify-between gap-2",
                                    omni-text { "data-text": "Size", "data-strategy": "none", "data-max-lines": "1", class: "text-muted-foreground" }
                                    omni-text { "data-text": "{size_label}", "data-strategy": "none", "data-max-lines": "1" }
                                }
                                div { class: "space-y-1",
                                    omni-text { "data-text": "Source", "data-strategy": "none", "data-max-lines": "1", class: "text-muted-foreground" }
                                    omni-text { "data-text": "{source_label}", "data-strategy": "none", "data-max-lines": "2", class: "text-muted-foreground/80" }
                                    omni-text {
                                        "data-text": "{download_url}",
                                        "data-strategy": "none",
                                        "data-max-lines": "4",
                                        class: "break-all text-foreground",
                                    }
                                }
                            }
                            if let Some(error) = browser_status.last_error.clone() {
                                div { class: "rounded-sm border border-status-critical bg-status-critical/10 px-3 py-2 text-[10px] text-status-critical",
                                    omni-text { "data-text": "{error}", "data-strategy": "none", "data-max-lines": "4" }
                                }
                            }
                            div { class: "flex items-center justify-end gap-2",
                                button {
                                    class: "rounded-sm border border-border px-3 py-1.5 text-[11px] text-muted-foreground hover:bg-background-interactive",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        pending_download.set(None);
                                    },
                                    omni-text { "data-text": "Back", "data-strategy": "none", "data-max-lines": "1" }
                                }
                                button {
                                    class: "rounded-sm bg-primary px-3 py-1.5 text-[11px] text-primary-foreground hover:opacity-90 disabled:opacity-50",
                                    disabled: download_phase == crate::lib::BrowserDownloadPhase::Downloading,
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        pending_download.set(None);
                                        open.set(true);
                                        if let Some(spec) = browser_model_spec(&model_for_download.id) {
                                            let mut state = model_state.write();
                                            state.browser_inference.last_error = None;
                                            state.browser_inference.download.phase = crate::lib::BrowserDownloadPhase::Downloading;
                                            state.browser_inference.download.model_id = Some(model_for_download.id.clone());
                                            state.browser_inference.download.loaded_bytes = Some(0);
                                            state.browser_inference.download.total_bytes = Some(spec.size);
                                            state.browser_inference.download.progress_percent = Some(0);
                                        }
                                        active_download_model_id.set(Some(model_for_download.id.clone()));
                                        let mut model_state_for_download = model_state;
                                        let model_id = model_for_download.id.clone();
                                        spawn(async move {
                                            match crate::lib::sw_api::start_browser_model_download(&model_id).await {
                                                Ok(()) => {}
                                                Err(error) => {
                                                    let mut state = model_state_for_download.write();
                                                    state.browser_inference.last_error = Some(error.to_string());
                                                    state.browser_inference.download.phase = crate::lib::BrowserDownloadPhase::Error;
                                                }
                                            }
                                        });
                                    },
                                    omni-text { "data-text": "Download", "data-strategy": "none", "data-max-lines": "1" }
                                }
                            }
                        }
                    }
                }
            } else if let Some(model) = pending_delete() {
                {
                    let model_for_delete = model.clone();
                    let tid_for_delete = tid.clone();
                    rsx! {
                        div { class: "space-y-3",
                            div {
                                omni-text { "data-text": "Delete {model.name}?", "data-strategy": "none", "data-max-lines": "1", class: "text-[11px] font-semibold text-foreground" }
                                omni-text { "data-text": "This removes the model from your browser's local storage.", "data-strategy": "none", "data-max-lines": "2", class: "mt-1 text-[10px] text-muted-foreground" }
                            }
                            div { class: "rounded-sm border border-status-warning/40 bg-status-warning/10 px-3 py-2 text-[10px] text-muted-foreground",
                                omni-text { "data-text": "You can download it again later if you need it.", "data-strategy": "none", "data-max-lines": "2" }
                            }
                            if let Some(error) = browser_status.last_error.clone() {
                                div { class: "rounded-sm border border-status-critical bg-status-critical/10 px-3 py-2 text-[10px] text-status-critical",
                                    omni-text { "data-text": "{error}", "data-strategy": "none", "data-max-lines": "4" }
                                }
                            }
                            div { class: "flex items-center justify-end gap-2",
                                button {
                                    class: "rounded-sm border border-border px-3 py-1.5 text-[11px] text-muted-foreground hover:bg-background-interactive",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        pending_delete.set(None);
                                    },
                                    omni-text { "data-text": "Cancel", "data-strategy": "none", "data-max-lines": "1" }
                                }
                                button {
                                    class: "rounded-sm bg-status-critical px-3 py-1.5 text-[11px] text-white hover:opacity-90",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        pending_delete.set(None);
                                        {
                                            let mut state = model_state.write();
                                            state.browser_inference.cached_model_ids.retain(|cached| cached != &model_for_delete.id);
                                            if state.browser_inference.loaded_model_id.as_deref() == Some(model_for_delete.id.as_str()) {
                                                state.browser_inference.loaded_model_id = None;
                                            }
                                            if state.browser_inference.download.model_id.as_deref() == Some(model_for_delete.id.as_str()) {
                                                state.browser_inference.download.phase = crate::lib::BrowserDownloadPhase::Idle;
                                                state.browser_inference.download.model_id = None;
                                                state.browser_inference.download.loaded_bytes = None;
                                                state.browser_inference.download.total_bytes = None;
                                                state.browser_inference.download.progress_percent = None;
                                            }
                                        }
                                        let mut model_state_for_delete = model_state;
                                        let model_id = model_for_delete.id.clone();
                                        let tid_for_async = tid_for_delete.clone();
                                        spawn(async move {
                                            match crate::lib::sw_api::delete_browser_model(&model_id).await {
                                                Ok(()) => {
                                                    match crate::lib::sw_api::get_browser_inference_status().await {
                                                        Ok(status) => {
                                                            let fallback_model = status
                                                                .cached_model_ids
                                                                .first()
                                                                .cloned()
                                                                .unwrap_or_else(|| "claude-3-7-sonnet".to_string());
                                                            let mut state = model_state_for_delete.write();
                                                            state.browser_inference = status;
                                                            if state.selected_model_for(&tid_for_async) == model_id {
                                                                state.selected_model.insert(tid_for_async.clone(), fallback_model);
                                                            }
                                                        }
                                                        Err(error) => {
                                                            let mut state = model_state_for_delete.write();
                                                            state.browser_inference.last_error = Some(error.to_string());
                                                        }
                                                    }
                                                }
                                                Err(error) => {
                                                    match crate::lib::sw_api::get_browser_inference_status().await {
                                                        Ok(status) => {
                                                            let mut state = model_state_for_delete.write();
                                                            state.browser_inference = status;
                                                            state.browser_inference.last_error = Some(error.to_string());
                                                        }
                                                        Err(_) => {
                                                            let mut state = model_state_for_delete.write();
                                                            state.browser_inference.last_error = Some(error.to_string());
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    },
                                    omni-text { "data-text": "Delete", "data-strategy": "none", "data-max-lines": "1" }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "flex gap-0",
                    div { class: "w-[140px] shrink-0 space-y-0.5 border-r border-border pr-2 mr-2",
                        for p in providers {
                            {
                                let dot_class = if p.has_api_key { "bg-status-nominal" } else { "bg-status-warning" };
                                let btn_class = if p.id == selected_provider() {
                                    "flex w-full cursor-pointer items-center gap-2 rounded-sm px-2 py-1.5 text-left text-[11px] bg-background-interactive"
                                } else {
                                    "flex w-full cursor-pointer items-center gap-2 rounded-sm px-2 py-1.5 text-left text-[11px] hover:bg-background-interactive text-muted-foreground"
                                };
                                let pid = p.id.clone();
                                rsx! {
                                    button {
                                        key: "{p.name}",
                                        class: "{btn_class}",
                                        onclick: move |_| selected_provider.set(pid.clone()),
                                        div { class: "h-1.5 w-1.5 rounded-full {dot_class} shrink-0" }
                                        omni-text { "data-text": "{p.name}", "data-strategy": "truncate", "data-max-lines": "1" }
                                    }
                                }
                            }
                        }
                        if selected_provider() != crate::lib::ProviderId::Browser {
                            button {
                                class: "mt-1 w-full rounded-sm border border-border px-2 py-1 text-left text-[10px] text-muted-foreground hover:bg-background-interactive",
                                onclick: move |_| {
                                    let provider = selected_provider();
                                    ui_state.write().api_key_provider = provider.clone();

                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        let mut ui_for_load = ui_state;
                                        let prefix = provider_prefix(&provider).to_string();
                                        spawn(async move {
                                            let key = crate::lib::sw_api::get_api_key(&prefix)
                                                .await
                                                .unwrap_or_default();
                                            ui_for_load.write().api_key_draft = key;
                                        });
                                    }

                                    ui_state.write().api_key_dialog_open = true;
                                    open.set(false);
                                },
                                omni-text { "data-text": "API Keys", "data-strategy": "none", "data-max-lines": "1" }
                            }
                        }
                    }
                    div { class: "flex-1 space-y-0.5",
                        for model in filtered_models {
                            {
                                let is_browser = model.provider == crate::lib::ProviderId::Browser;
                                let is_cached = browser_status.cached_model_ids.iter().any(|cached| cached == &model.id);
                                let is_loaded = browser_status.loaded_model_id.as_deref() == Some(model.id.as_str());
                                let is_downloading = browser_status.download.phase == crate::lib::BrowserDownloadPhase::Downloading
                                    && browser_status.download.model_id.as_deref() == Some(model.id.as_str());
                                let download_progress = browser_download_progress_percent(&model.id, &browser_status);
                                let download_segments = browser_download_segment_label(&model.id, &browser_status);
                                let browser_badge = if is_browser {
                                    if is_downloading {
                                        Some((
                                            BadgeVariant::Info,
                                            download_segments.unwrap_or_else(|| "0/0".to_string()),
                                        ))
                                    } else if is_loaded {
                                        Some((BadgeVariant::Nominal, "LOADED".to_string()))
                                    } else if is_cached {
                                        Some((BadgeVariant::Nominal, "READY".to_string()))
                                    } else {
                                        Some((BadgeVariant::Warning, "DOWNLOAD".to_string()))
                                    }
                                } else {
                                    None
                                };
                                let btn_class = if model.id == selected_model {
                                    "relative w-full cursor-pointer overflow-hidden rounded-sm px-2 py-1.5 text-left text-[11px] bg-primary/10 text-primary"
                                } else {
                                    "relative w-full cursor-pointer overflow-hidden rounded-sm px-2 py-1.5 text-left text-[11px] hover:bg-background-interactive text-muted-foreground"
                                };
                                let mid = model.id.clone();
                                let model_name = model.name.clone();
                                let tid_for_click = tid.clone();
                                let model_for_confirm = model.clone();
                                let model_for_delete = model.clone();
                                rsx! {
                                    div {
                                        key: "{model.id}",
                                        class: "relative",
                                        div {
                                            class: "{btn_class}",
                                            onclick: move |_| {
                                                if is_browser && !is_cached {
                                                    if !is_downloading {
                                                        pending_download.set(Some(model_for_confirm.clone()));
                                                    }
                                                    return;
                                                }
                                                model_state.write().selected_model.insert(tid_for_click.clone(), mid.clone());
                                                #[cfg(target_arch = "wasm32")]
                                                {
                                                    let model_id = mid.clone();
                                                    spawn(async move {
                                                        let _ = crate::lib::sw_api::set_default_model(&model_id).await;
                                                    });
                                                }
                                                open.set(false);
                                            },
                                            if let Some(progress_percent) = download_progress {
                                                div {
                                                    class: "pointer-events-none absolute inset-y-0 left-0 rounded-sm bg-status-info/15",
                                                    style: "width: {progress_percent}%;",
                                                }
                                            }
                                            if is_downloading {
                                                div { class: "pointer-events-none absolute inset-y-0 left-0 z-10 w-[42%] browser-download-sweep" }
                                            }
                                            div { class: "relative z-20 flex items-center gap-2",
                                                div { class: "min-w-0 flex-1",
                                                    omni-text {
                                                        "data-text": "{model_name}",
                                                        "data-strategy": "truncate",
                                                        "data-max-lines": "1",
                                                        class: "min-w-0",
                                                    }
                                                    if is_downloading {
                                                        omni-text { "data-text": "{download_progress.unwrap_or(0)}% of total download", "data-strategy": "none", "data-max-lines": "1", class: "mt-0.5 text-[10px] text-muted-foreground" }
                                                    }
                                                }
                                                if let Some((variant, badge_text)) = browser_badge.clone() {
                                                    if is_browser && is_cached && !is_downloading {
                                                        div { class: "group relative shrink-0",
                                                            Badge { variant: variant, class: "shrink-0 transition-opacity group-hover:opacity-0", "{badge_text}" }
                                                            button {
                                                                class: "absolute inset-0 inline-flex items-center justify-center rounded-sm border border-status-critical/30 bg-status-critical/10 text-status-critical opacity-0 transition-opacity group-hover:opacity-100",
                                                                onclick: move |evt| {
                                                                    evt.stop_propagation();
                                                                    pending_delete.set(Some(model_for_delete.clone()));
                                                                },
                                                                Icon { width: 11, height: 11, icon: LdTrash2 }
                                                            }
                                                        }
                                                    } else if is_downloading {
                                                        div { class: "group relative shrink-0",
                                                            Badge { variant: variant, class: "shrink-0 transition-opacity group-hover:opacity-0", "{badge_text}" }
                                                            button {
                                                                class: "absolute inset-0 inline-flex items-center justify-center rounded-sm border border-status-critical/30 bg-status-critical/10 text-status-critical opacity-0 transition-opacity group-hover:opacity-100",
                                                                onclick: move |evt| {
                                                                    evt.stop_propagation();
                                                                    {
                                                                        let mut state = model_state.write();
                                                                        state.browser_inference.last_error = None;
                                                                        state.browser_inference.download.phase = crate::lib::BrowserDownloadPhase::Idle;
                                                                        state.browser_inference.download.model_id = None;
                                                                        state.browser_inference.download.loaded_bytes = None;
                                                                        state.browser_inference.download.total_bytes = None;
                                                                        state.browser_inference.download.progress_percent = None;
                                                                        state.browser_inference.cached_model_ids.retain(|cached| cached != &model.id);
                                                                        if state.browser_inference.loaded_model_id.as_deref() == Some(model.id.as_str()) {
                                                                            state.browser_inference.loaded_model_id = None;
                                                                        }
                                                                    }
                                                                    let mut model_state_for_stop = model_state;
                                                                    let model_id = model.id.clone();
                                                                    spawn(async move {
                                                                        match crate::lib::sw_api::stop_browser_model_download(&model_id).await {
                                                                            Ok(()) => {
                                                                                if let Ok(status) = crate::lib::sw_api::get_browser_inference_status().await {
                                                                                    model_state_for_stop.write().browser_inference = status;
                                                                                }
                                                                            }
                                                                            Err(error) => {
                                                                                let refreshed_status = crate::lib::sw_api::get_browser_inference_status().await.ok();
                                                                                let mut state = model_state_for_stop.write();
                                                                                if let Some(status) = refreshed_status {
                                                                                    state.browser_inference = status;
                                                                                }
                                                                                state.browser_inference.last_error = Some(error.to_string());
                                                                            }
                                                                        }
                                                                    });
                                                                },
                                                                Icon { width: 10, height: 10, icon: LdSquare }
                                                            }
                                                        }
                                                    } else {
                                                        Badge { variant: variant, class: "shrink-0", "{badge_text}" }
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
                }
            }
        }
    }
}

#[component]
pub fn WorkspacePicker() -> Element {
    let workspace_state = use_context::<Signal<WorkspaceState>>();
    let thread_state = use_context::<Signal<ThreadState>>();
    let mut open = use_signal(|| false);
    let presets = vec![
        ("test", "/home/user/projects/test"),
        ("omni", "/home/user/projects/omni"),
        ("omni-rt", "/home/user/projects/omni-rt"),
    ];
    let tid = thread_state
        .read()
        .active_thread_id
        .clone()
        .unwrap_or_default();

    rsx! {
        Popover {
            open: open(),
            on_close: move |_| open.set(false),
            trigger: rsx! {
                button {
                    class: "flex items-center gap-1 rounded-sm border border-border px-2 py-1 text-[11px] text-muted-foreground hover:bg-background-interactive",
                    onclick: move |_| open.set(!open()),
                    Icon { width: 10, height: 10, icon: LdFolder }
                    omni-text { "data-text": "{workspace_state.read().workspace_for(&tid)}", "data-strategy": "truncate", "data-max-lines": "1", class: "max-w-[160px]" }
                    Icon { width: 10, height: 10, icon: LdChevronDown }
                }
            },
            div { class: "space-y-1",
                omni-text { "data-text": "Select Workspace", "data-strategy": "none", "data-max-lines": "1", class: "px-2 pb-1 text-[9px] font-semibold uppercase tracking-widest text-muted-foreground" }
                for (name, path) in presets {
                    {
                        let active = workspace_state.read().workspace_for(&tid) == path;
                        let btn_class = if active {
                            "flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-left bg-primary/10 text-primary"
                        } else {
                            "flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-left hover:bg-background-interactive text-muted-foreground"
                        };
                        let tid_for_click = tid.clone();
                        let mut ws_state = workspace_state;
                        rsx! {
                            button {
                                key: "{name}",
                                class: "{btn_class}",
                                onclick: move |_| {
                                    let workspace_path = path.to_string();
                                    ws_state
                                        .write()
                                        .workspace_path
                                        .insert(tid_for_click.clone(), workspace_path.clone());
                                    spawn(async move {
                                        if let Ok(files) = crate::lib::sw_api::list_workspace_files(&workspace_path).await {
                                            ws_state.write().workspace_files.insert(workspace_path, files);
                                        }
                                    });
                                    open.set(false);
                                },
                                Icon { width: 12, height: 12, icon: LdFolder, class: "shrink-0" }
                                div {
                                    omni-text { "data-text": "{name}", "data-strategy": "truncate", "data-max-lines": "1", class: "text-[11px] font-semibold" }
                                    omni-text { "data-text": "{path}", "data-strategy": "truncate", "data-max-lines": "1", class: "text-[10px] text-muted-foreground" }
                                }
                            }
                        }
                    }
                }
                div { class: "h-px w-full bg-border my-1" }
                button {
                    class: "flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-left text-[11px] text-muted-foreground hover:bg-background-interactive",
                    Icon { width: 12, height: 12, icon: LdFolder, class: "shrink-0" }
                    omni-text { "data-text": "Browse...", "data-strategy": "none", "data-max-lines": "1" }
                }
            }
        }
    }
}
