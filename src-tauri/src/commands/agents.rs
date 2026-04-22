use crate::dto::agents::{
    CreateAgentResultDto, DeleteAgentResultDto, ImportAgentsResultDto, ManagedAgentDto,
    ManualAgentDraftDto, RemoveAgentResultDto, ResolvedAgentDto, ScanTargetDto,
    ScannedAgentCandidateDto,
};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn list_managed_agents(state: State<'_, AppState>) -> Result<Vec<ManagedAgentDto>, String> {
    Ok(state.agent_discovery_service.list_managed_agents())
}

#[tauri::command]
pub fn list_resolved_agents(
    state: State<'_, AppState>,
    scan_targets: Vec<ScanTargetDto>,
) -> Result<Vec<ResolvedAgentDto>, String> {
    Ok(state.agent_discovery_service.list_resolved_agents(scan_targets))
}

#[tauri::command]
pub fn scan_agents(
    state: State<'_, AppState>,
    scan_targets: Vec<ScanTargetDto>,
) -> Result<Vec<ScannedAgentCandidateDto>, String> {
    Ok(state.agent_discovery_service.scan_agents(scan_targets))
}

#[tauri::command]
pub fn refresh_agent_discovery(
    state: State<'_, AppState>,
    scan_targets: Vec<ScanTargetDto>,
) -> Result<Vec<ResolvedAgentDto>, String> {
    Ok(state.agent_discovery_service.refresh_agent_discovery(
        scan_targets,
    ))
}

#[tauri::command]
pub fn import_agents(
    state: State<'_, AppState>,
    candidate_ids: Vec<String>,
    scan_targets: Vec<ScanTargetDto>,
) -> Result<ImportAgentsResultDto, String> {
    Ok(state.agent_discovery_service.import_agents(candidate_ids, scan_targets)?)
}

#[tauri::command]
pub fn remove_managed_agent(
    state: State<'_, AppState>,
    managed_agent_id: String,
    scan_targets: Vec<ScanTargetDto>,
) -> Result<RemoveAgentResultDto, String> {
    Ok(state.agent_discovery_service.remove_managed_agent(managed_agent_id, scan_targets)?)
}

#[tauri::command]
pub fn delete_agent(
    state: State<'_, AppState>,
    managed_agent_id: String,
    scan_targets: Vec<ScanTargetDto>,
) -> Result<DeleteAgentResultDto, String> {
    Ok(state.agent_discovery_service.delete_agent(managed_agent_id, scan_targets)?)
}

#[tauri::command]
pub fn create_agent(
    state: State<'_, AppState>,
    draft: ManualAgentDraftDto,
) -> Result<CreateAgentResultDto, String> {
    Ok(state.agent_discovery_service.create_agent(draft)?)
}
