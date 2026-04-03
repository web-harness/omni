use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSearchRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub agent_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub capabilities: AgentCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    #[serde(rename = "ap.io.messages")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ap_io_messages: Option<bool>,
    #[serde(rename = "ap.io.streaming")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ap_io_streaming: Option<bool>,
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSchema {
    pub agent_id: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_schema: Option<serde_json::Value>,
}

fn default_limit() -> i32 {
    10
}
