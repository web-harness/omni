use chrono::Utc;
use omni_protocol::{TableCheckpoint, ThreadState};
use omni_zenfs as zenfs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

const CHECKPOINTS_DIR: &str = "/home/db/checkpoints";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredThreadState {
    created_at: String,
    state: ThreadState,
}

fn thread_dir(thread_id: &str) -> String {
    format!("{CHECKPOINTS_DIR}/{thread_id}")
}

fn checkpoint_path(thread_id: &str, checkpoint_id: Uuid) -> String {
    format!("{}/{checkpoint_id}.json", thread_dir(thread_id))
}

pub async fn append_thread_state(
    thread_id: &str,
    values: serde_json::Value,
    messages: Option<Vec<omni_protocol::Message>>,
    metadata: Option<HashMap<String, serde_json::Value>>,
    checkpoint: Option<TableCheckpoint>,
) -> Result<ThreadState, std::io::Error> {
    let checkpoint = checkpoint.unwrap_or(TableCheckpoint {
        checkpoint_id: Uuid::new_v4(),
        extra: HashMap::new(),
    });
    let state = ThreadState {
        checkpoint: checkpoint.clone(),
        values,
        messages,
        metadata,
    };
    let stored = StoredThreadState {
        created_at: Utc::now().to_rfc3339(),
        state: state.clone(),
    };

    let dir = thread_dir(thread_id);
    zenfs::mkdir(&dir, true).await?;
    let bytes =
        serde_json::to_vec(&stored).map_err(|error| std::io::Error::other(error.to_string()))?;
    zenfs::write_file(
        &checkpoint_path(thread_id, checkpoint.checkpoint_id),
        &bytes,
    )
    .await?;
    Ok(state)
}

pub async fn list_thread_states(thread_id: &str) -> Result<Vec<ThreadState>, std::io::Error> {
    let dir = thread_dir(thread_id);
    if !zenfs::exists(&dir).await? {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for entry in zenfs::read_dir(&dir).await? {
        if !entry.name.ends_with(".json") {
            continue;
        }
        let path = format!("{dir}/{}", entry.name);
        let bytes = zenfs::read_file(&path).await?;
        if let Ok(state) = serde_json::from_slice::<StoredThreadState>(&bytes) {
            entries.push(state);
        }
    }

    entries.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(entries.into_iter().map(|entry| entry.state).collect())
}

pub async fn delete_thread_states(thread_id: &str) -> Result<(), std::io::Error> {
    let dir = thread_dir(thread_id);
    if zenfs::exists(&dir).await? {
        zenfs::remove(&dir, true).await?;
    }
    Ok(())
}

pub async fn copy_thread_states(
    source_thread_id: &str,
    target_thread_id: &str,
) -> Result<(), std::io::Error> {
    let states = list_thread_states(source_thread_id).await?;
    for state in states.into_iter().rev() {
        append_thread_state(
            target_thread_id,
            state.values,
            state.messages,
            state.metadata,
            Some(TableCheckpoint {
                checkpoint_id: Uuid::new_v4(),
                extra: state.checkpoint.extra,
            }),
        )
        .await?;
    }
    Ok(())
}
