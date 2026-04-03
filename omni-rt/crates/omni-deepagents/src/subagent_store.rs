use omni_zenfs as zenfs;
use serde::{Deserialize, Serialize};

const SUBAGENTS_DIR: &str = "/home/db/subagents";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubagentStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSubagent {
    pub id: String,
    pub thread_id: String,
    pub name: String,
    pub description: String,
    pub status: SubagentStatus,
}

pub async fn list_subagents(thread_id: &str) -> Result<Vec<StoredSubagent>, std::io::Error> {
    let dir = format!("{}/{}", SUBAGENTS_DIR, thread_id);
    if !zenfs::exists(&dir).await? {
        return Ok(vec![]);
    }
    let entries = zenfs::read_dir(&dir).await?;
    let mut subagents = Vec::new();
    for entry in entries {
        if entry.name.ends_with(".json") {
            let path = format!("{}/{}", dir, entry.name);
            if let Ok(data) = zenfs::read_file(&path).await {
                if let Ok(sa) = serde_json::from_slice::<StoredSubagent>(&data) {
                    subagents.push(sa);
                }
            }
        }
    }
    Ok(subagents)
}

pub async fn save_subagent(sa: &StoredSubagent) -> Result<(), std::io::Error> {
    let dir = format!("{}/{}", SUBAGENTS_DIR, sa.thread_id);
    if let Err(e) = zenfs::mkdir(&dir, true).await {
        if !e.to_string().contains("EEXIST") {
            return Err(e);
        }
    }
    let data = serde_json::to_vec(sa).map_err(|e| std::io::Error::other(e.to_string()))?;
    let path = format!("{}/{}.json", dir, sa.id);
    zenfs::write_file(&path, &data).await?;
    Ok(())
}

pub async fn delete_thread_subagents(thread_id: &str) -> Result<(), std::io::Error> {
    let dir = format!("{}/{}", SUBAGENTS_DIR, thread_id);
    if zenfs::exists(&dir).await? {
        zenfs::remove(&dir, true).await?;
    }
    Ok(())
}
