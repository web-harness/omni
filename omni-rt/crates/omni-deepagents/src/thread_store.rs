use chrono::Utc;
use omni_protocol::{Thread, ThreadCreate, ThreadPatch, ThreadStatus};
use omni_zenfs as zenfs;
use std::collections::HashMap;
use uuid::Uuid;

const THREADS_DIR: &str = "/home/db/threads";

fn thread_path(id: Uuid) -> String {
    format!("{}/{}.json", THREADS_DIR, id)
}

fn thread_path_candidates(id: &str) -> Vec<String> {
    let mut candidates = vec![format!("{}/{}.json", THREADS_DIR, id)];
    if let Ok(uuid) = Uuid::parse_str(id) {
        let canonical = format!("{}/{}.json", THREADS_DIR, uuid);
        if !candidates.contains(&canonical) {
            candidates.push(canonical);
        }
        let legacy = format!("{}/{}.json", THREADS_DIR, uuid.simple());
        if !candidates.contains(&legacy) {
            candidates.push(legacy);
        }
    }
    candidates
}

pub async fn list_threads() -> Result<Vec<Thread>, std::io::Error> {
    if !zenfs::exists(THREADS_DIR).await? {
        return Ok(vec![]);
    }

    let entries = zenfs::read_dir(THREADS_DIR).await?;
    let mut threads = Vec::new();

    for entry in entries {
        if entry.name.ends_with(".json") {
            let path = format!("{}/{}", THREADS_DIR, entry.name);
            if let Ok(data) = zenfs::read_file(&path).await {
                if let Ok(thread) = serde_json::from_slice::<Thread>(&data) {
                    threads.push(thread);
                }
            }
        }
    }

    threads.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(threads)
}

pub async fn get_thread(id: &str) -> Result<Option<Thread>, std::io::Error> {
    for path in thread_path_candidates(id) {
        if !zenfs::exists(&path).await? {
            continue;
        }
        let data = zenfs::read_file(&path).await?;
        let thread = serde_json::from_slice::<Thread>(&data)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        return Ok(Some(thread));
    }
    Ok(None)
}

pub async fn create_thread(title: Option<&str>) -> Result<Thread, std::io::Error> {
    let mut metadata = HashMap::new();
    metadata.insert(
        "title".to_string(),
        serde_json::Value::String(title.unwrap_or("New Thread").to_string()),
    );
    metadata.insert(
        "workspace".to_string(),
        serde_json::Value::String("/home/workspace".to_string()),
    );
    create_thread_from_request(ThreadCreate {
        thread_id: None,
        metadata: Some(metadata),
        if_exists: None,
    })
    .await
}

pub async fn create_thread_with_status(
    title: &str,
    workspace: &str,
    status: ThreadStatus,
    updated_at: String,
) -> Result<Thread, std::io::Error> {
    let mut metadata = HashMap::new();
    metadata.insert(
        "title".to_string(),
        serde_json::Value::String(title.to_string()),
    );
    metadata.insert(
        "workspace".to_string(),
        serde_json::Value::String(workspace.to_string()),
    );
    let mut thread = create_thread_from_request(ThreadCreate {
        thread_id: None,
        metadata: Some(metadata),
        if_exists: None,
    })
    .await?;
    thread.status = status;
    thread.updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    persist_thread(&thread).await?;
    Ok(thread)
}

pub async fn create_thread_from_request(req: ThreadCreate) -> Result<Thread, std::io::Error> {
    if let Err(e) = zenfs::mkdir(THREADS_DIR, true).await {
        if !e.to_string().contains("EEXIST") {
            return Err(e);
        }
    }
    let id = req.thread_id.unwrap_or_else(Uuid::new_v4);
    let now = Utc::now();
    let mut metadata = req.metadata.unwrap_or_default();
    if !metadata.contains_key("workspace") {
        metadata.insert(
            "workspace".to_string(),
            serde_json::Value::String("/home/workspace".to_string()),
        );
    }
    let thread = Thread {
        thread_id: id,
        created_at: now,
        updated_at: now,
        metadata,
        status: ThreadStatus::Idle,
        values: None,
        messages: None,
    };
    persist_thread(&thread).await?;
    Ok(thread)
}

pub async fn update_thread(id: &str, patch: ThreadPatch) -> Result<Option<Thread>, std::io::Error> {
    let mut thread = match get_thread(id).await? {
        Some(t) => t,
        None => return Ok(None),
    };

    if let Some(metadata) = patch.metadata {
        for (k, v) in metadata {
            thread.metadata.insert(k, v);
        }
    }
    if let Some(values) = patch.values {
        thread.values = Some(values);
    }
    if let Some(messages) = patch.messages {
        thread.messages = Some(messages);
    }
    thread.updated_at = Utc::now();
    persist_thread(&thread).await?;
    Ok(Some(thread))
}

pub async fn delete_thread(id: &str) -> Result<(), std::io::Error> {
    for path in thread_path_candidates(id) {
        if zenfs::exists(&path).await? {
            zenfs::remove(&path, false).await?;
        }
    }
    Ok(())
}

pub async fn set_thread_status(
    id: &str,
    status: ThreadStatus,
) -> Result<Option<Thread>, std::io::Error> {
    let mut thread = match get_thread(id).await? {
        Some(thread) => thread,
        None => return Ok(None),
    };

    thread.status = status;
    thread.updated_at = Utc::now();
    persist_thread(&thread).await?;
    Ok(Some(thread))
}

pub async fn save_thread(thread: &Thread) -> Result<(), std::io::Error> {
    if let Err(error) = zenfs::mkdir(THREADS_DIR, true).await {
        if !error.to_string().contains("EEXIST") {
            return Err(error);
        }
    }
    persist_thread(thread).await
}

async fn persist_thread(thread: &Thread) -> Result<(), std::io::Error> {
    let data = serde_json::to_vec(thread).map_err(|e| std::io::Error::other(e.to_string()))?;
    let path = thread_path(thread.thread_id);
    if let Some(legacy_path) = thread_path_candidates(&thread.thread_id.to_string())
        .into_iter()
        .find(|candidate| candidate.ends_with(&format!("/{}.json", thread.thread_id.simple())))
    {
        if legacy_path != path && zenfs::exists(&legacy_path).await? {
            zenfs::remove(&legacy_path, false).await?;
        }
    }
    zenfs::write_file(&path, &data).await
}
