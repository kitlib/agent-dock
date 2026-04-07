use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentResourceCountsDto {
    pub skill: u32,
    pub command: u32,
    pub mcp: u32,
    pub subagent: u32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredAgentDto {
    pub discovery_id: String,
    pub fingerprint: String,
    pub agent_type: String,
    pub display_name: String,
    pub root_path: String,
    pub status: String,
    pub reason: Option<String>,
    pub resource_counts: AgentResourceCountsDto,
    pub detected_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedAgentDto {
    pub managed_agent_id: String,
    pub fingerprint: String,
    pub alias: Option<String>,
    pub enabled: bool,
    pub hidden: bool,
    pub imported_at: String,
    pub source: String,
    #[serde(alias = "provider")]
    pub agent_type: Option<String>,
    pub root_path: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedAgentDto {
    pub id: String,
    pub discovery_id: String,
    pub fingerprint: String,
    pub agent_type: String,
    pub name: String,
    pub alias: Option<String>,
    pub role: String,
    pub root_path: String,
    pub managed: bool,
    pub managed_agent_id: Option<String>,
    pub enabled: bool,
    pub hidden: bool,
    pub health: String,
    pub status: String,
    pub status_label: String,
    pub summary: String,
    pub group_id: String,
    pub resource_counts: AgentResourceCountsDto,
    pub last_scanned_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedAgentCandidateDto {
    pub id: String,
    pub fingerprint: String,
    pub agent_type: String,
    pub display_name: String,
    pub root_path: String,
    pub resource_counts: AgentResourceCountsDto,
    pub state: String,
    pub reason: Option<String>,
    pub managed_agent_id: Option<String>,
    pub managed: bool,
    pub detected_at: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanTargetDto {
    pub agent_type: String,
    pub name: String,
    pub root_path: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualAgentDraftDto {
    pub agent_type: String,
    pub name: String,
    pub root_path: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportAgentsResultDto {
    pub imported_agents: Vec<ResolvedAgentDto>,
    pub resolved_agents: Vec<ResolvedAgentDto>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveAgentResultDto {
    pub removed_agent_id: String,
    pub resolved_agents: Vec<ResolvedAgentDto>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAgentResultDto {
    pub deleted_agent_id: String,
    pub resolved_agents: Vec<ResolvedAgentDto>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentResultDto {
    pub agent: ResolvedAgentDto,
    pub resolved_agents: Vec<ResolvedAgentDto>,
}
