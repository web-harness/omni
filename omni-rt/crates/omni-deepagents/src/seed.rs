use crate::{message_store, subagent_store, thread_store, todo_store};
use chrono::Utc;
use message_store::{Role, StoredMessage};
use omni_protocol::ThreadStatus;
use omni_zenfs as zenfs;
use subagent_store::{StoredSubagent, SubagentStatus};
use todo_store::{StoredTodo, TodoStatus};

const THREADS_DIR: &str = "/home/db/threads";

pub async fn seed_if_empty() -> Result<(), std::io::Error> {
    match zenfs::mkdir(THREADS_DIR, true).await {
        Ok(_) => {}
        Err(e) if e.to_string().contains("EEXIST") => {}
        Err(e) => return Err(e),
    }
    let existing = thread_store::list_threads().await?;
    if !existing.is_empty() {
        return Ok(());
    }

    let now = Utc::now();

    // --- Thread 1: Implement todo management system ---
    let t1 = thread_store::create_thread_with_status(
        "Implement todo management sys...",
        ThreadStatus::Busy,
        now.to_rfc3339(),
    )
    .await?;
    let t1id = t1.thread_id.simple().to_string();

    for msg in [
        StoredMessage { id: "m1".into(), thread_id: t1id.clone(), role: Role::User, content: "I need a full todo management system with CRUD operations, filtering by status, and persistence via ZenFS.".into(), created_at: "2025-03-30T10:00:00Z".into() },
        StoredMessage { id: "m2".into(), thread_id: t1id.clone(), role: Role::Assistant, content: "I'll implement a comprehensive todo management system. Let me start by setting up the data layer.".into(), created_at: "2025-03-30T10:00:05Z".into() },
        StoredMessage { id: "m3".into(), thread_id: t1id.clone(), role: Role::Tool, content: "```rust\n// todo_store.rs created\npub struct Todo { pub id: String, pub content: String, pub status: TodoStatus }\n```".into(), created_at: "2025-03-30T10:00:10Z".into() },
        StoredMessage { id: "m4".into(), thread_id: t1id.clone(), role: Role::Assistant, content: "Data layer is ready. Now building the UI components with filtering and sorting capabilities.".into(), created_at: "2025-03-30T10:00:15Z".into() },
    ] { message_store::save_message(&msg).await?; }

    for todo in [
        StoredTodo {
            id: "todo1".into(),
            thread_id: t1id.clone(),
            content: "Design TodoStore data structure".into(),
            status: TodoStatus::Completed,
        },
        StoredTodo {
            id: "todo2".into(),
            thread_id: t1id.clone(),
            content: "Implement CRUD operations".into(),
            status: TodoStatus::InProgress,
        },
        StoredTodo {
            id: "todo3".into(),
            thread_id: t1id.clone(),
            content: "Add ZenFS persistence".into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "todo4".into(),
            thread_id: t1id.clone(),
            content: "Build filter/sort UI".into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "todo5".into(),
            thread_id: t1id.clone(),
            content: "Write unit tests".into(),
            status: TodoStatus::Pending,
        },
    ] {
        todo_store::save_todo(&todo).await?;
    }

    for sa in [
        StoredSubagent {
            id: "sa1".into(),
            thread_id: t1id.clone(),
            name: "FileWriter".into(),
            description: "Writes and edits source files".into(),
            status: SubagentStatus::Completed,
        },
        StoredSubagent {
            id: "sa2".into(),
            thread_id: t1id.clone(),
            name: "TestRunner".into(),
            description: "Runs test suite and reports results".into(),
            status: SubagentStatus::Running,
        },
        StoredSubagent {
            id: "sa3".into(),
            thread_id: t1id.clone(),
            name: "CodeReviewer".into(),
            description: "Reviews code for issues and suggests improvements".into(),
            status: SubagentStatus::Pending,
        },
    ] {
        subagent_store::save_subagent(&sa).await?;
    }

    // --- Thread 2: Implement Auth Flow ---
    let t2 = thread_store::create_thread_with_status(
        "Implement Auth Flow",
        ThreadStatus::Interrupted,
        (now - chrono::Duration::hours(16)).to_rfc3339(),
    )
    .await?;
    let t2id = t2.thread_id.simple().to_string();

    for msg in [
        StoredMessage { id: "m5".into(), thread_id: t2id.clone(), role: Role::User, content: "Set up JWT-based auth with refresh tokens, protected routes, and session persistence.".into(), created_at: "2025-03-29T18:00:00Z".into() },
        StoredMessage { id: "m6".into(), thread_id: t2id.clone(), role: Role::Assistant, content: "I'll implement a complete auth flow. Starting with the JWT middleware and token storage.".into(), created_at: "2025-03-29T18:00:05Z".into() },
        StoredMessage { id: "m7".into(), thread_id: t2id.clone(), role: Role::User, content: "Also add Google OAuth as a second factor please.".into(), created_at: "2025-03-29T18:05:00Z".into() },
    ] { message_store::save_message(&msg).await?; }

    for todo in [
        StoredTodo {
            id: "todo6".into(),
            thread_id: t2id.clone(),
            content: "Set up JWT middleware".into(),
            status: TodoStatus::Completed,
        },
        StoredTodo {
            id: "todo7".into(),
            thread_id: t2id.clone(),
            content: "Implement refresh token rotation".into(),
            status: TodoStatus::InProgress,
        },
        StoredTodo {
            id: "todo8".into(),
            thread_id: t2id.clone(),
            content: "Add Google OAuth provider".into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "todo9".into(),
            thread_id: t2id.clone(),
            content: "Protect routes with auth guards".into(),
            status: TodoStatus::Pending,
        },
    ] {
        todo_store::save_todo(&todo).await?;
    }

    for sa in [
        StoredSubagent {
            id: "sa4".into(),
            thread_id: t2id.clone(),
            name: "AuthDesigner".into(),
            description: "Designs auth architecture".into(),
            status: SubagentStatus::Completed,
        },
        StoredSubagent {
            id: "sa5".into(),
            thread_id: t2id.clone(),
            name: "SecurityAuditor".into(),
            description: "Audits for security vulnerabilities".into(),
            status: SubagentStatus::Pending,
        },
    ] {
        subagent_store::save_subagent(&sa).await?;
    }

    // --- Thread 3: Database Migration ---
    let t3 = thread_store::create_thread_with_status(
        "Database Migration",
        ThreadStatus::Idle,
        (now - chrono::Duration::hours(46)).to_rfc3339(),
    )
    .await?;
    let t3id = t3.thread_id.simple().to_string();

    for msg in [
        StoredMessage { id: "m8".into(), thread_id: t3id.clone(), role: Role::User, content: "We need to migrate from SQLite to PostgreSQL without downtime. There are 2M records.".into(), created_at: "2025-03-28T12:00:00Z".into() },
        StoredMessage { id: "m9".into(), thread_id: t3id.clone(), role: Role::Assistant, content: "I'll design a zero-downtime migration strategy using dual-write with a backfill job.".into(), created_at: "2025-03-28T12:00:05Z".into() },
    ] { message_store::save_message(&msg).await?; }

    for todo in [
        StoredTodo {
            id: "todo10".into(),
            thread_id: t3id.clone(),
            content: "Audit current schema".into(),
            status: TodoStatus::Completed,
        },
        StoredTodo {
            id: "todo11".into(),
            thread_id: t3id.clone(),
            content: "Write migration scripts".into(),
            status: TodoStatus::Completed,
        },
        StoredTodo {
            id: "todo12".into(),
            thread_id: t3id.clone(),
            content: "Set up dual-write mode".into(),
            status: TodoStatus::InProgress,
        },
        StoredTodo {
            id: "todo13".into(),
            thread_id: t3id.clone(),
            content: "Backfill historical data".into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "todo14".into(),
            thread_id: t3id.clone(),
            content: "Verify data integrity".into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "todo15".into(),
            thread_id: t3id.clone(),
            content: "Cut over to PostgreSQL".into(),
            status: TodoStatus::Pending,
        },
    ] {
        todo_store::save_todo(&todo).await?;
    }

    // --- Thread 4: Setup CI Pipeline ---
    let t4 = thread_store::create_thread_with_status(
        "Setup CI Pipeline",
        ThreadStatus::Idle,
        (now - chrono::Duration::hours(73)).to_rfc3339(),
    )
    .await?;
    let t4id = t4.thread_id.simple().to_string();

    for msg in [
        StoredMessage { id: "m10".into(), thread_id: t4id.clone(), role: Role::User, content: "Set up GitHub Actions with test, lint, build, deploy stages and environment-specific secrets.".into(), created_at: "2025-03-27T09:00:00Z".into() },
        StoredMessage { id: "m11".into(), thread_id: t4id.clone(), role: Role::Assistant, content: "I'll create a comprehensive CI/CD pipeline. Let me start with the workflow configuration.".into(), created_at: "2025-03-27T09:00:05Z".into() },
    ] { message_store::save_message(&msg).await?; }

    for todo in [
        StoredTodo {
            id: "todo16".into(),
            thread_id: t4id.clone(),
            content: "Create .github/workflows/ci.yml".into(),
            status: TodoStatus::Completed,
        },
        StoredTodo {
            id: "todo17".into(),
            thread_id: t4id.clone(),
            content: "Add test and lint jobs".into(),
            status: TodoStatus::Completed,
        },
        StoredTodo {
            id: "todo18".into(),
            thread_id: t4id.clone(),
            content: "Configure deployment environments".into(),
            status: TodoStatus::Pending,
        },
        StoredTodo {
            id: "todo19".into(),
            thread_id: t4id.clone(),
            content: "Set up secret management".into(),
            status: TodoStatus::Pending,
        },
    ] {
        todo_store::save_todo(&todo).await?;
    }

    // --- Thread 5: Collaboration question ---
    let t5 = thread_store::create_thread_with_status(
        "What would be a good approach...",
        ThreadStatus::Idle,
        (now - chrono::Duration::hours(111)).to_rfc3339(),
    )
    .await?;
    let t5id = t5.thread_id.simple().to_string();

    for msg in [
        StoredMessage { id: "m12".into(), thread_id: t5id.clone(), role: Role::User, content: "What would be a good approach to implement real-time collaboration features?".into(), created_at: "2025-03-26T15:00:00Z".into() },
        StoredMessage { id: "m13".into(), thread_id: t5id.clone(), role: Role::Assistant, content: "For real-time collaboration, I'd recommend a CRDT-based approach using Yjs. Here's why:\n\n1. **Conflict-free merging** — no server coordination needed\n2. **Offline support** — changes sync when reconnected\n3. **Proven ecosystem** — works with CodeMirror, ProseMirror, etc.\n\nAlternatively, OT (Operational Transformation) works well if you need stricter ordering guarantees.".into(), created_at: "2025-03-26T15:00:05Z".into() },
    ] { message_store::save_message(&msg).await?; }

    Ok(())
}
