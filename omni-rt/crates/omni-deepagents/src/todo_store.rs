use omni_zenfs as zenfs;
use serde::{Deserialize, Serialize};

const TODOS_DIR: &str = "/home/db/todos";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTodo {
    pub id: String,
    pub thread_id: String,
    pub content: String,
    pub status: TodoStatus,
}

pub async fn list_todos(thread_id: &str) -> Result<Vec<StoredTodo>, std::io::Error> {
    let dir = format!("{}/{}", TODOS_DIR, thread_id);
    if !zenfs::exists(&dir).await? {
        return Ok(vec![]);
    }
    let entries = zenfs::read_dir(&dir).await?;
    let mut todos = Vec::new();
    for entry in entries {
        if entry.name.ends_with(".json") {
            let path = format!("{}/{}", dir, entry.name);
            if let Ok(data) = zenfs::read_file(&path).await {
                if let Ok(todo) = serde_json::from_slice::<StoredTodo>(&data) {
                    todos.push(todo);
                }
            }
        }
    }
    Ok(todos)
}

pub async fn save_todo(todo: &StoredTodo) -> Result<(), std::io::Error> {
    let dir = format!("{}/{}", TODOS_DIR, todo.thread_id);
    if let Err(e) = zenfs::mkdir(&dir, true).await {
        if !e.to_string().contains("EEXIST") {
            return Err(e);
        }
    }
    let data = serde_json::to_vec(todo).map_err(|e| std::io::Error::other(e.to_string()))?;
    let path = format!("{}/{}.json", dir, todo.id);
    zenfs::write_file(&path, &data).await?;
    Ok(())
}

pub async fn delete_thread_todos(thread_id: &str) -> Result<(), std::io::Error> {
    let dir = format!("{}/{}", TODOS_DIR, thread_id);
    if zenfs::exists(&dir).await? {
        zenfs::remove(&dir, true).await?;
    }
    Ok(())
}
