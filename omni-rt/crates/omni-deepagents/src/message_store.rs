use omni_zenfs as zenfs;
use serde::{Deserialize, Serialize};

const MESSAGES_DIR: &str = "/home/db/messages";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: String,
    pub thread_id: String,
    pub role: Role,
    pub content: String,
    pub created_at: String,
}

pub async fn list_messages(thread_id: &str) -> Result<Vec<StoredMessage>, std::io::Error> {
    let dir = format!("{}/{}", MESSAGES_DIR, thread_id);
    if !zenfs::exists(&dir).await? {
        return Ok(vec![]);
    }
    let entries = zenfs::read_dir(&dir).await?;
    let mut messages = Vec::new();
    for entry in entries {
        if entry.name.ends_with(".json") {
            let path = format!("{}/{}", dir, entry.name);
            if let Ok(data) = zenfs::read_file(&path).await {
                if let Ok(msg) = serde_json::from_slice::<StoredMessage>(&data) {
                    messages.push(msg);
                }
            }
        }
    }
    messages.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(messages)
}

pub async fn save_message(msg: &StoredMessage) -> Result<(), std::io::Error> {
    let dir = format!("{}/{}", MESSAGES_DIR, msg.thread_id);
    if let Err(e) = zenfs::mkdir(&dir, true).await {
        if !e.to_string().contains("EEXIST") {
            return Err(e);
        }
    }
    let data = serde_json::to_vec(msg).map_err(|e| std::io::Error::other(e.to_string()))?;
    let path = format!("{}/{}.json", dir, msg.id);
    zenfs::write_file(&path, &data).await?;
    Ok(())
}

pub async fn delete_thread_messages(thread_id: &str) -> Result<(), std::io::Error> {
    let dir = format!("{}/{}", MESSAGES_DIR, thread_id);
    if zenfs::exists(&dir).await? {
        zenfs::remove(&dir, true).await?;
    }
    Ok(())
}
