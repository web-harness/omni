use chrono::Utc;
use omni_protocol::ThreadStatus;
use omni_zenfs as zenfs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

const THREADS_DIR: &str = "/home/db/threads";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredThread {
    pub thread_id: String,
    pub title: String,
    pub status: ThreadStatus,
    pub created_at: String,
    pub updated_at: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub async fn list_threads() -> Result<Vec<StoredThread>, std::io::Error> {
    if !zenfs::exists(THREADS_DIR).await? {
        return Ok(vec![]);
    }

    let entries = zenfs::read_dir(THREADS_DIR).await?;
    let mut threads = Vec::new();

    for entry in entries {
        if entry.name.ends_with(".json") {
            let path = format!("{}/{}", THREADS_DIR, entry.name);
            if let Ok(data) = zenfs::read_file(&path).await {
                if let Ok(thread) = serde_json::from_slice::<StoredThread>(&data) {
                    threads.push(thread);
                }
            }
        }
    }

    threads.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(threads)
}

pub async fn get_thread(id: &str) -> Result<Option<StoredThread>, std::io::Error> {
    let path = format!("{}/{}.json", THREADS_DIR, id);
    if !zenfs::exists(&path).await? {
        return Ok(None);
    }
    let data = zenfs::read_file(&path).await?;
    let thread = serde_json::from_slice::<StoredThread>(&data)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(Some(thread))
}

pub async fn create_thread(title: Option<&str>) -> Result<StoredThread, std::io::Error> {
    create_thread_with_status(
        title.unwrap_or("New Thread"),
        ThreadStatus::Idle,
        Utc::now().to_rfc3339(),
    )
    .await
}

pub async fn create_thread_with_status(
    title: &str,
    status: ThreadStatus,
    updated_at: String,
) -> Result<StoredThread, std::io::Error> {
    if let Err(e) = zenfs::mkdir(THREADS_DIR, true).await {
        if !e.to_string().contains("EEXIST") {
            return Err(e);
        }
    }
    let id = Uuid::new_v4().simple().to_string();
    let now = Utc::now().to_rfc3339();
    let thread = StoredThread {
        thread_id: id.clone(),
        title: title.to_string(),
        status,
        created_at: now,
        updated_at,
        metadata: HashMap::new(),
    };
    let data = serde_json::to_vec(&thread).map_err(|e| std::io::Error::other(e.to_string()))?;
    let path = format!("{}/{}.json", THREADS_DIR, id);
    zenfs::write_file(&path, &data).await?;
    Ok(thread)
}

pub async fn update_thread(
    id: &str,
    title: Option<&str>,
    status: Option<ThreadStatus>,
) -> Result<Option<StoredThread>, std::io::Error> {
    let mut thread = match get_thread(id).await? {
        Some(t) => t,
        None => return Ok(None),
    };
    if let Some(t) = title {
        thread.title = t.to_string();
    }
    if let Some(s) = status {
        thread.status = s;
    }
    thread.updated_at = Utc::now().to_rfc3339();
    let data = serde_json::to_vec(&thread).map_err(|e| std::io::Error::other(e.to_string()))?;
    let path = format!("{}/{}.json", THREADS_DIR, id);
    zenfs::write_file(&path, &data).await?;
    Ok(Some(thread))
}

pub async fn delete_thread(id: &str) -> Result<(), std::io::Error> {
    let path = format!("{}/{}.json", THREADS_DIR, id);
    if zenfs::exists(&path).await? {
        zenfs::remove(&path, false).await?;
    }
    Ok(())
}
