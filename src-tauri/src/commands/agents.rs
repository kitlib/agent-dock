use crate::dto::agents::{
    AgentConflictDto, CreateAgentResultDto, DeleteAgentResultDto, DiscoveredAgentDto,
    ImportAgentsResultDto, ManagedAgentDto, ManualAgentDraftDto, RemoveAgentResultDto,
    ResolvedAgentDto, ScannedAgentCandidateDto,
};
use crate::services::agent_discovery_service;

#[tauri::command]
pub fn list_discovered_agents() -> Result<Vec<DiscoveredAgentDto>, String> {
    Ok(agent_discovery_service::list_discovered_agents())
}

#[tauri::command]
pub fn list_managed_agents() -> Result<Vec<ManagedAgentDto>, String> {
    Ok(agent_discovery_service::list_managed_agents())
}

#[tauri::command]
pub fn list_agent_conflicts() -> Result<Vec<AgentConflictDto>, String> {
    Ok(agent_discovery_service::list_agent_conflicts())
}

#[tauri::command]
pub fn list_resolved_agents() -> Result<Vec<ResolvedAgentDto>, String> {
    Ok(agent_discovery_service::list_resolved_agents())
}

#[tauri::command]
pub fn scan_agents() -> Result<Vec<ScannedAgentCandidateDto>, String> {
    Ok(agent_discovery_service::scan_agents())
}

#[tauri::command]
pub fn refresh_agent_discovery() -> Result<Vec<ResolvedAgentDto>, String> {
    Ok(agent_discovery_service::refresh_agent_discovery())
}

#[tauri::command]
pub fn import_agents(candidate_ids: Vec<String>) -> Result<ImportAgentsResultDto, String> {
    agent_discovery_service::import_agents(candidate_ids)
}

#[tauri::command]
pub fn remove_managed_agent(managed_agent_id: String) -> Result<RemoveAgentResultDto, String> {
    agent_discovery_service::remove_managed_agent(managed_agent_id)
}

#[tauri::command]
pub fn delete_agent(managed_agent_id: String) -> Result<DeleteAgentResultDto, String> {
    agent_discovery_service::delete_agent(managed_agent_id)
}

#[tauri::command]
pub fn create_agent(draft: ManualAgentDraftDto) -> Result<CreateAgentResultDto, String> {
    agent_discovery_service::create_agent(draft)
}
