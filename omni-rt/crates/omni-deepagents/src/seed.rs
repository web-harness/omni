use crate::{message_store, subagent_store, thread_store, todo_store, workspace_seed};
use chrono::Utc;
use message_store::StoredMessage;
use omni_protocol::ThreadStatus;
use omni_zenfs as zenfs;
use subagent_store::{StoredSubagent, SubagentStatus};
use todo_store::{StoredTodo, TodoStatus};
use uuid::Uuid;

const THREADS_DIR: &str = "/home/db/threads";

/// Fixed thread IDs matching web store-mocks.ts MOCK_THREAD_IDS
const THREAD_ID_GTD: &str = "11111111-1111-4111-8111-111111111111";
const THREAD_ID_AUTH: &str = "22222222-2222-4222-8222-222222222222";
const THREAD_ID_DB: &str = "33333333-3333-4333-8333-333333333333";
const THREAD_ID_CI: &str = "44444444-4444-4444-8444-444444444444";
const THREAD_ID_IDEA: &str = "55555555-5555-4555-8555-555555555555";

async fn backfill_seeded_workspaces(
    threads: Vec<omni_protocol::Thread>,
) -> Result<(), std::io::Error> {
    for (index, mut thread) in threads.into_iter().enumerate() {
        if thread.metadata.contains_key("workspace") {
            continue;
        }
        thread.metadata.insert(
            "workspace".to_string(),
            serde_json::Value::String(
                workspace_seed::default_workspace_for_index(index).to_string(),
            ),
        );
        thread_store::save_thread(&thread).await?;
    }
    Ok(())
}

fn stored_message(
    id: &str,
    thread_id: String,
    role: &str,
    content: &str,
    created_at: &str,
) -> StoredMessage {
    StoredMessage {
        id: id.into(),
        thread_id,
        role: role.into(),
        content: serde_json::Value::String(content.into()),
        created_at: created_at.into(),
        metadata: None,
        extra: Default::default(),
    }
}

fn parse_uuid(s: &str) -> Uuid {
    Uuid::parse_str(s).expect("invalid hardcoded UUID")
}

pub async fn seed_if_empty() -> Result<(), std::io::Error> {
    match zenfs::mkdir(THREADS_DIR, true).await {
        Ok(_) => {}
        Err(e) if e.to_string().contains("EEXIST") => {}
        Err(e) => return Err(e),
    }
    let existing = thread_store::list_threads().await?;
    if !existing.is_empty() {
        backfill_seeded_workspaces(existing).await?;
        workspace_seed::ensure_workspace_scaffold().await?;
        return Ok(());
    }

    let now = Utc::now();
    let four_hours_ago = (now - chrono::Duration::hours(4)).to_rfc3339();

    // --- Thread 1: GTD/Kanban/Chaos Mode (matches web MOCK_THREAD_IDS.gtd) ---
    let t1 = thread_store::create_thread_with_id_and_status(
        Some(parse_uuid(THREAD_ID_GTD)),
        "New Thread",
        workspace_seed::default_workspace_for_index(0),
        ThreadStatus::Busy,
        four_hours_ago.clone(),
    )
    .await?;
    let t1id = t1.thread_id.to_string();

    for msg in [
        stored_message(
            "m1",
            t1id.clone(),
            "user",
            "Build a todo management system with three modes: GTD (Getting Things Done), Kanban, and Chaos Mode (random prioritization). Research and implement all three.",
            "2025-03-30T10:00:00Z",
        ),
    ] {
        message_store::save_message(&msg).await?;
    }

    for todo in [
        StoredTodo {
            id: "t1".into(),
            thread_id: t1id.clone(),
            content: "Research GTD (Getting Things Done) methodology using subagent".into(),
            status: TodoStatus::InProgress,
        },
        StoredTodo {
            id: "t2".into(),
            thread_id: t1id.clone(),
            content: "Research Kanban methodology using subagent".into(),
            status: TodoStatus::InProgress,
        },
        StoredTodo {
            id: "t3".into(),
            thread_id: t1id.clone(),
            content: "Research Chaos Mode (random prioritization) approach using subagent".into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "t4".into(),
            thread_id: t1id.clone(),
            content: "Design data structure and API endpoints for three todo management systems"
                .into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "t5".into(),
            thread_id: t1id.clone(),
            content: "Implement GTD backend endpoints and logic in server.js".into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "t6".into(),
            thread_id: t1id.clone(),
            content: "Implement Kanban backend endpoints and logic in server.js".into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "t7".into(),
            thread_id: t1id.clone(),
            content: "Implement Chaos Mode backend endpoints and logic in server.js".into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "t8".into(),
            thread_id: t1id.clone(),
            content: "Create frontend UI for GTD system".into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "t9".into(),
            thread_id: t1id.clone(),
            content: "Create frontend UI for Kanban system".into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "t10".into(),
            thread_id: t1id.clone(),
            content: "Create frontend UI for Chaos Mode system".into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "t11".into(),
            thread_id: t1id.clone(),
            content: "Update README.md with documentation for new systems".into(),
            status: TodoStatus::Pending,
        },
    ] {
        todo_store::save_todo(&todo).await?;
    }

    for sa in [
        StoredSubagent {
            id: "sa-gtd-1".into(),
            thread_id: t1id.clone(),
            name: "General Purpose Agent".into(),
            description: "Research the GTD (Getting Things Done) methodology by David Allen. I need you to provide a comprehensive report covering: 1. Core principles and philosophy of GTD 2. The key components: Inbox, Next Actions, Projects, Waiting For, Someday/Maybe, Contexts 3. The 5 stages of workflow: Capture, Clarify, Organize, Reflect, Engage".into(),
            status: SubagentStatus::Running,
        },
        StoredSubagent {
            id: "sa-gtd-2".into(),
            thread_id: t1id.clone(),
            name: "General Purpose Agent".into(),
            description: "Research the Kanban methodology for task management. I need you to provide a comprehensive report covering the core principles, board setup, WIP limits, and flow metrics.".into(),
            status: SubagentStatus::Running,
        },
        StoredSubagent {
            id: "sa-gtd-3".into(),
            thread_id: t1id.clone(),
            name: "General Purpose Agent".into(),
            description: "Research and design a Chaos Mode todo management system based on random prioritization and unpredictable task ordering. I need you to provide a creative report...".into(),
            status: SubagentStatus::Pending,
        },
    ] {
        subagent_store::save_subagent(&sa).await?;
    }

    // --- Thread 2: Auth (matches web MOCK_THREAD_IDS.auth) ---
    let t2 = thread_store::create_thread_with_id_and_status(
        Some(parse_uuid(THREAD_ID_AUTH)),
        "New Thread",
        workspace_seed::default_workspace_for_index(1),
        ThreadStatus::Interrupted,
        four_hours_ago.clone(),
    )
    .await?;
    let t2id = t2.thread_id.to_string();

    for msg in [
        stored_message(
            "m1",
            t2id.clone(),
            "user",
            "Ship OAuth login with refresh rotation and audit trail.",
            "2025-03-29T18:00:00Z",
        ),
        stored_message(
            "m2",
            t2id.clone(),
            "assistant",
            "Copy. I will update auth middleware, add token persistence, and run smoke tests.",
            "2025-03-29T18:00:05Z",
        ),
    ] {
        message_store::save_message(&msg).await?;
    }

    for todo in generic_todos(&t2id) {
        todo_store::save_todo(&todo).await?;
    }

    for sa in generic_subagents(&t2id) {
        subagent_store::save_subagent(&sa).await?;
    }

    // --- Thread 3: DB Migration (matches web MOCK_THREAD_IDS.db) ---
    let t3 = thread_store::create_thread_with_id_and_status(
        Some(parse_uuid(THREAD_ID_DB)),
        "New Thread",
        workspace_seed::default_workspace_for_index(2),
        ThreadStatus::Idle,
        four_hours_ago.clone(),
    )
    .await?;
    let t3id = t3.thread_id.to_string();

    for msg in [
        stored_message(
            "m1",
            t3id.clone(),
            "user",
            "Need migration plan for v3 schema.",
            "2025-03-28T12:00:00Z",
        ),
        stored_message(
            "m2",
            t3id.clone(),
            "assistant",
            "Migration paused pending approval to write production scripts.",
            "2025-03-28T12:00:05Z",
        ),
    ] {
        message_store::save_message(&msg).await?;
    }

    for todo in generic_todos(&t3id) {
        todo_store::save_todo(&todo).await?;
    }

    for sa in generic_subagents(&t3id) {
        subagent_store::save_subagent(&sa).await?;
    }

    // --- Thread 4: CI Pipeline (matches web MOCK_THREAD_IDS.ci) ---
    let t4 = thread_store::create_thread_with_id_and_status(
        Some(parse_uuid(THREAD_ID_CI)),
        "New Thread",
        workspace_seed::default_workspace_for_index(0),
        ThreadStatus::Idle,
        four_hours_ago.clone(),
    )
    .await?;
    let t4id = t4.thread_id.to_string();

    // No messages (matches web)

    for todo in generic_todos(&t4id) {
        todo_store::save_todo(&todo).await?;
    }

    for sa in generic_subagents(&t4id) {
        subagent_store::save_subagent(&sa).await?;
    }

    // --- Thread 5: Idea (matches web MOCK_THREAD_IDS.idea) ---
    let t5 = thread_store::create_thread_with_id_and_status(
        Some(parse_uuid(THREAD_ID_IDEA)),
        "New Thread",
        workspace_seed::default_workspace_for_index(0),
        ThreadStatus::Idle,
        four_hours_ago.clone(),
    )
    .await?;
    let t5id = t5.thread_id.to_string();

    // No messages (matches web)

    for todo in generic_todos(&t5id) {
        todo_store::save_todo(&todo).await?;
    }

    for sa in generic_subagents(&t5id) {
        subagent_store::save_subagent(&sa).await?;
    }

    workspace_seed::ensure_workspace_scaffold().await?;
    Ok(())
}

fn generic_todos(thread_id: &str) -> Vec<StoredTodo> {
    vec![
        StoredTodo {
            id: "t1".into(),
            thread_id: thread_id.into(),
            content: "Wire dock slots".into(),
            status: TodoStatus::Completed,
        },
        StoredTodo {
            id: "t2".into(),
            thread_id: thread_id.into(),
            content: "Port sidebar interactions".into(),
            status: TodoStatus::InProgress,
        },
        StoredTodo {
            id: "t3".into(),
            thread_id: thread_id.into(),
            content: "Implement model switcher".into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "t4".into(),
            thread_id: thread_id.into(),
            content: "File viewer tabs".into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "t5".into(),
            thread_id: thread_id.into(),
            content: "Settings dialog".into(),
            status: TodoStatus::Pending,
        },
    ]
}

fn generic_subagents(thread_id: &str) -> Vec<StoredSubagent> {
    vec![
        StoredSubagent {
            id: "sa-1".into(),
            thread_id: thread_id.into(),
            name: "Spec Auditor".into(),
            description: "Cross-checks implementation against the phase plan".into(),
            status: SubagentStatus::Running,
        },
        StoredSubagent {
            id: "sa-2".into(),
            thread_id: thread_id.into(),
            name: "Test Synth".into(),
            description: "Generates verification tests for touched modules".into(),
            status: SubagentStatus::Completed,
        },
    ]
}
