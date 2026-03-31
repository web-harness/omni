use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdBot, LdChevronDown, LdChevronRight, LdFolder, LdListTodo, LdSend, LdUser,
};
use dioxus_free_icons::Icon;
use futures_util::StreamExt;

use crate::components::ui::{Badge, BadgeVariant, Popover};
use crate::lib::thread_context::apply_stream_event;
use crate::lib::{
    ChatState, ModelState, Role, TasksState, ThreadState, ToolCall, ToolResult, UiState,
    WorkspaceState,
};

#[derive(Clone)]
struct StreamRequest {
    thread_id: String,
    input: String,
    model_id: String,
}

#[component]
pub fn ChatContainer(thread_id: String) -> Element {
    let thread_state = use_context::<Signal<ThreadState>>();
    let mut chat_state = use_context::<Signal<ChatState>>();
    let mut tasks_state = use_context::<Signal<TasksState>>();

    let stream = use_coroutine(move |mut rx: UnboundedReceiver<StreamRequest>| async move {
        while let Some(req) = rx.next().await {
            chat_state.write().is_streaming = true;
            chat_state.write().error = None;

            #[cfg(target_arch = "wasm32")]
            {
                use omni_rt::deepagents::sse::{SseEvent, SseStream};
                let body = serde_json::json!({
                    "thread_id": req.thread_id,
                    "input": req.input,
                    "model_id": req.model_id,
                })
                .to_string();

                match SseStream::connect("/api/runs/stream", &body).await {
                    Ok(mut stream) => loop {
                        match stream.next_event().await {
                            Ok(Some(SseEvent::Token(tok))) => {
                                let active_tid = thread_state.read().active_thread_id.clone();
                                apply_stream_event(
                                    active_tid.as_deref(),
                                    &mut chat_state.write(),
                                    &mut tasks_state.write(),
                                    crate::lib::StreamEvent::Token(tok),
                                );
                            }
                            Ok(Some(SseEvent::Done)) => {
                                apply_stream_event(
                                    Some(&req.thread_id),
                                    &mut chat_state.write(),
                                    &mut tasks_state.write(),
                                    crate::lib::StreamEvent::Done,
                                );
                                break;
                            }
                            Ok(Some(SseEvent::Error(e))) => {
                                chat_state.write().error = Some(e);
                                chat_state.write().is_streaming = false;
                                break;
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

            #[cfg(not(target_arch = "wasm32"))]
            {
                chat_state.write().is_streaming = false;
            }
        }
    });

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
                            div { class: "text-xs font-semibold text-muted-foreground", "NEW THREAD" }
                            p { class: "mt-2 text-sm", "Pick workspace, choose model, and issue your first task." }
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
                            div { class: "mb-1 text-muted-foreground", "Agent is working..." }
                            pre { class: "whitespace-pre-wrap", "{chat_state.read().stream_buffer}" }
                        }
                    }
                    if let Some(err) = chat_state.read().error.clone() {
                        div { class: "rounded-sm border border-status-critical bg-status-critical/10 p-2 text-[11px] text-status-critical", "{err}" }
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
                div { class: "mb-1 text-[10px] font-semibold text-muted-foreground", "{label}" }
                div { class: "{bubble_class}",
                    pre { class: "whitespace-pre-wrap font-sans text-[12px]", "{message.content}" }
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
        "dispatch_subagent" => rsx! { SubagentTaskRenderer { call } },
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

    let is_done = result.is_some();

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
                span { class: "font-semibold", "Update Tasks" }
                div { class: "ml-auto flex items-center gap-1",
                    if is_done {
                        Badge { variant: BadgeVariant::Nominal, "OK" }
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
fn SubagentTaskRenderer(call: ToolCall) -> Element {
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
                span { class: "font-semibold", "Subagent Task" }
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
                span { class: "font-semibold font-mono", "{call.name}" }
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
                                        stream.send(StreamRequest { thread_id: thread_id.clone(), input, model_id: model_state.read().selected_model_for(&active_id) });
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
                                stream.send(StreamRequest { thread_id: thread_id.clone(), input, model_id: model_state.read().selected_model_for(&active_id) });
                            }
                        },
                        Icon { width: 13, height: 13, icon: LdSend }
                    }
                }
                div { class: "mt-2 flex items-center gap-2",
                    ModelSwitcher {}
                    WorkspacePicker {}
                    div { class: "ml-auto text-[10px] text-muted-foreground whitespace-nowrap",
                        "~2.4k input · ~580 output · $0.012"
                    }
                }
            }
        }
    }
}

#[component]
pub fn ModelSwitcher() -> Element {
    let mut model_state = use_context::<Signal<ModelState>>();
    let mut ui_state = use_context::<Signal<UiState>>();
    let thread_state = use_context::<Signal<ThreadState>>();
    let mut open = use_signal(|| false);
    let mut selected_provider = use_signal(|| crate::lib::ProviderId::Anthropic);

    let tid = thread_state
        .read()
        .active_thread_id
        .clone()
        .unwrap_or_default();
    let providers = model_state.read().providers.clone();
    let models = model_state.read().models.clone();
    let selected_model = model_state.read().selected_model_for(&tid);

    let filtered_models: Vec<_> = models
        .iter()
        .filter(|m| m.provider == selected_provider())
        .cloned()
        .collect();

    #[cfg(target_arch = "wasm32")]
    fn provider_prefix(provider: &crate::lib::ProviderId) -> &'static str {
        match provider {
            crate::lib::ProviderId::Anthropic => "anthropic",
            crate::lib::ProviderId::OpenAI => "openai",
            crate::lib::ProviderId::Google => "google",
            crate::lib::ProviderId::Ollama => "ollama",
        }
    }

    rsx! {
        Popover {
            open: open(),
            on_close: move |_| open.set(false),
            trigger: rsx! {
                button {
                    class: "flex items-center gap-1 rounded-sm border border-border px-2 py-1 text-[11px] text-muted-foreground hover:bg-background-interactive",
                    onclick: move |_| open.set(!open()),
                    omni-text { "data-text": "{selected_model}", "data-strategy": "truncate", "data-max-lines": "1", class: "max-w-[180px]" }
                    Icon { width: 10, height: 10, icon: LdChevronDown }
                }
            },
            div { class: "flex gap-0",
                div { class: "w-[140px] shrink-0 space-y-0.5 border-r border-border pr-2 mr-2",
                    for p in providers {
                        {
                            let dot_class = if p.has_api_key { "bg-status-nominal" } else { "bg-status-warning" };
                            let btn_class = if p.id == selected_provider() {
                                "flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-left text-[11px] bg-background-interactive"
                            } else {
                                "flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-left text-[11px] hover:bg-background-interactive text-muted-foreground"
                            };
                            let pid = p.id.clone();
                            rsx! {
                                button {
                                    key: "{p.name}",
                                    class: "{btn_class}",
                                    onclick: move |_| selected_provider.set(pid.clone()),
                                    div { class: "h-1.5 w-1.5 rounded-full {dot_class} shrink-0" }
                                    span { "{p.name}" }
                                }
                            }
                        }
                    }
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
                        "API Keys"
                    }
                }
                div { class: "flex-1 space-y-0.5",
                    for model in filtered_models {
                        {
                            let btn_class = if model.id == selected_model {
                                "w-full rounded-sm px-2 py-1.5 text-left text-[11px] bg-primary/10 text-primary"
                            } else {
                                "w-full rounded-sm px-2 py-1.5 text-left text-[11px] hover:bg-background-interactive text-muted-foreground"
                            };
                            let mid = model.id.clone();
                            let tid_for_click = tid.clone();
                            rsx! {
                                button {
                                    key: "{model.id}",
                                    class: "{btn_class}",
                                    onclick: move |_| {
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
                                    "{model.name}"
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
    let mut workspace_state = use_context::<Signal<WorkspaceState>>();
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
                div { class: "px-2 pb-1 text-[9px] font-semibold uppercase tracking-widest text-muted-foreground", "Select Workspace" }
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
                                    div { class: "text-[11px] font-semibold", "{name}" }
                                    div { class: "text-[10px] text-muted-foreground", "{path}" }
                                }
                            }
                        }
                    }
                }
                div { class: "h-px w-full bg-border my-1" }
                button {
                    class: "flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-left text-[11px] text-muted-foreground hover:bg-background-interactive",
                    Icon { width: 12, height: 12, icon: LdFolder, class: "shrink-0" }
                    span { "Browse..." }
                }
            }
        }
    }
}
