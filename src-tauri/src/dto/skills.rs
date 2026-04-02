use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillScanTargetDto {
    pub agent_id: String,
    pub agent_type: String,
    pub root_path: String,
    pub display_name: String,
    pub source: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSupportingFileDto {
    pub path: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSkillSummaryDto {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub summary: String,
    pub enabled: bool,
    pub tags: Vec<String>,
    pub usage_count: u32,
    pub updated_at: String,
    pub owner_agent_id: String,
    pub source_label: String,
    pub description: String,
    pub status: String,
    pub skill_path: String,
    pub entry_file_path: String,
    pub agent_type: String,
    pub agent_name: String,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSkillDetailDto {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub enabled: bool,
    pub tags: Vec<String>,
    pub usage_count: u32,
    pub updated_at: String,
    pub markdown: String,
    pub owner_agent_id: String,
    pub source_label: String,
    pub status: String,
    pub skill_path: String,
    pub entry_file_path: String,
    pub agent_type: String,
    pub agent_name: String,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub frontmatter: Option<Value>,
    pub frontmatter_raw: Option<String>,
    pub supporting_files: Vec<SkillSupportingFileDto>,
    pub allowed_tools: Vec<String>,
}
