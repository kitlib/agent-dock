use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;

/// MCP import conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpImportConflictStrategy {
    Overwrite,
    Skip,
}

impl FromStr for McpImportConflictStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "overwrite" => Ok(Self::Overwrite),
            "skip" => Ok(Self::Skip),
            _ => Err(format!("Unsupported conflict strategy: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpScanTargetDto {
    pub agent_id: String,
    pub agent_type: String,
    pub root_path: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Imported MCP server DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedMcpServer {
    pub transport: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub url: Option<String>,
    pub headers: BTreeMap<String, String>,
}

/// Import local MCP result DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportLocalMcpResultDto {
    pub config_path: String,
    pub imported_count: u32,
    pub skipped_count: u32,
    pub imported_names: Vec<String>,
    pub skipped_names: Vec<String>,
}

/// Update local MCP result DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocalMcpResultDto {
    pub config_path: String,
    pub server_name: String,
}
