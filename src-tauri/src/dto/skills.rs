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
    pub source_kind: String,
    pub relative_path: String,
    pub description: String,
    pub status: String,
    pub skill_path: String,
    pub entry_file_path: String,
    pub agent_type: String,
    pub agent_name: String,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub frontmatter: Option<Value>,
    pub marketplace_source: Option<String>,
    pub marketplace_skill_id: Option<String>,
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
    pub source_kind: String,
    pub relative_path: String,
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
    pub marketplace_source: Option<String>,
    pub marketplace_skill_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSkillCopySourceDto {
    pub id: String,
    pub name: String,
    pub owner_agent_id: String,
    pub source_kind: String,
    pub relative_path: String,
    pub skill_path: String,
    pub entry_file_path: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSkillCopyTargetAgentDto {
    pub agent_id: String,
    pub agent_type: String,
    pub agent_name: String,
    pub root_path: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSkillCopyConflictDto {
    pub skill_id: String,
    pub skill_name: String,
    pub source_kind: String,
    pub destination_path: String,
    pub existing_path: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewLocalSkillCopyResultDto {
    pub target_agent_name: String,
    pub total_count: usize,
    pub conflict_count: usize,
    pub conflicts: Vec<LocalSkillCopyConflictDto>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSkillConflictResolutionDto {
    pub skill_id: String,
    pub action: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyLocalSkillsResultDto {
    pub copied_count: usize,
    pub skipped_count: usize,
}
