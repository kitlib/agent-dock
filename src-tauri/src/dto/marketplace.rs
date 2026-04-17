use serde::{Deserialize, Serialize};

use crate::dto::skills::LocalSkillCopyTargetAgentDto;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceItemDto {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub skill_id: Option<String>,
    pub author: String,
    pub source: String,
    pub version: String,
    pub installs: u64,
    pub updated_at: String,
    pub install_state: String,
    pub description: String,
    pub highlights: Vec<String>,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsShSkillDto {
    pub id: String,
    pub skill_id: String,
    pub name: String,
    pub source: String,
    pub installs: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceSkillDetailDto {
    pub description: String,
    pub markdown: String,
    pub raw_markdown: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallMarketplaceSkillRequestDto {
    pub source: String,
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub target_agent: LocalSkillCopyTargetAgentDto,
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceInstallPreviewDto {
    pub skill_path: String,
    pub entry_file_path: String,
    pub has_conflict: bool,
    pub existing_path: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceInstallResultDto {
    pub skill_path: String,
    pub entry_file_path: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceSkillUpdateCheckDto {
    pub managed: bool,
    pub has_update: bool,
    pub source: Option<String>,
    pub skill_id: Option<String>,
}
