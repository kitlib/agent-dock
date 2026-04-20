use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpScanTargetDto {
    pub agent_id: String,
    pub agent_type: String,
    pub root_path: String,
    pub display_name: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalMcpServerDto {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub summary: String,
    pub enabled: bool,
    pub endpoint: String,
    pub transport: String,
    pub usage_count: u32,
    pub updated_at: String,
    pub document: String,
    pub config: String,
    pub owner_agent_id: String,
    pub source_label: String,
    pub agent_type: String,
    pub agent_name: String,
    pub config_path: String,
    pub scope: String,
    pub project_path: Option<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditableLocalMcpDto {
    pub server_name: String,
    pub transport: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub url: Option<String>,
    pub headers: BTreeMap<String, String>,
}
