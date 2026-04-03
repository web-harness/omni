use omni_protocol::{Message, Run, RunSearchRequest, RunStatus};
use omni_zenfs as zenfs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

const RUNS_DIR: &str = "/home/db/runs";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredRunEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub event: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredRun {
    pub run: Run,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<Message>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<StoredRunEvent>>,
}

fn run_path(run_id: Uuid) -> String {
    format!("{}/{}.json", RUNS_DIR, run_id)
}

fn metadata_matches(
    actual: &HashMap<String, serde_json::Value>,
    expected: &Option<HashMap<String, serde_json::Value>>,
) -> bool {
    match expected {
        Some(expected) => expected
            .iter()
            .all(|(key, value)| actual.get(key) == Some(value)),
        None => true,
    }
}

pub async fn save_run(record: &StoredRun) -> Result<(), std::io::Error> {
    zenfs::mkdir(RUNS_DIR, true).await?;
    let data = serde_json::to_vec(record).map_err(|e| std::io::Error::other(e.to_string()))?;
    zenfs::write_file(&run_path(record.run.run_id), &data).await
}

pub async fn get_run(run_id: Uuid) -> Result<Option<StoredRun>, std::io::Error> {
    let path = run_path(run_id);
    if !zenfs::exists(&path).await? {
        return Ok(None);
    }
    let data = zenfs::read_file(&path).await?;
    let run = serde_json::from_slice::<StoredRun>(&data)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(Some(run))
}

pub async fn list_runs() -> Result<Vec<StoredRun>, std::io::Error> {
    if !zenfs::exists(RUNS_DIR).await? {
        return Ok(vec![]);
    }

    let entries = zenfs::read_dir(RUNS_DIR).await?;
    let mut runs = Vec::new();
    for entry in entries {
        if !entry.name.ends_with(".json") {
            continue;
        }
        let path = format!("{}/{}", RUNS_DIR, entry.name);
        if let Ok(data) = zenfs::read_file(&path).await {
            if let Ok(run) = serde_json::from_slice::<StoredRun>(&data) {
                runs.push(run);
            }
        }
    }

    runs.sort_by(|left, right| right.run.updated_at.cmp(&left.run.updated_at));
    Ok(runs)
}

pub async fn search_runs(request: &RunSearchRequest) -> Result<Vec<StoredRun>, std::io::Error> {
    let runs = list_runs().await?;
    let filtered = runs
        .into_iter()
        .filter(|record| {
            if let Some(status) = request.status {
                if record.run.status != status {
                    return false;
                }
            }
            if let Some(thread_id) = request.thread_id {
                if record.run.stream.create.thread_id != Some(thread_id) {
                    return false;
                }
            }
            if let Some(agent_id) = &request.agent_id {
                if record.run.stream.create.agent_id.as_ref() != Some(agent_id) {
                    return false;
                }
            }
            metadata_matches(&record.run.stream.create.metadata, &request.metadata)
        })
        .collect::<Vec<_>>();

    let offset = request.offset.max(0) as usize;
    let limit = request.limit.max(1) as usize;
    Ok(filtered.into_iter().skip(offset).take(limit).collect())
}

pub async fn delete_run(run_id: Uuid) -> Result<(), std::io::Error> {
    let path = run_path(run_id);
    if zenfs::exists(&path).await? {
        zenfs::remove(&path, false).await?;
    }
    Ok(())
}

pub fn is_terminal_status(status: RunStatus) -> bool {
    matches!(
        status,
        RunStatus::Error | RunStatus::Success | RunStatus::Timeout | RunStatus::Interrupted
    )
}
