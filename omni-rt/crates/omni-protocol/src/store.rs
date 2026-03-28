use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorePutRequest {
    pub namespace: Vec<String>,
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreDeleteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<Vec<String>>,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreSearchRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace_prefix: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<serde_json::Value>,
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreListNamespacesRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<i32>,
    #[serde(default = "default_limit_namespaces")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub namespace: Vec<String>,
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_limit() -> i32 {
    10
}

fn default_limit_namespaces() -> i32 {
    100
}
