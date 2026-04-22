use tauri::State;

use crate::dto::skills::{
    CopyLocalSkillsResultDto, LocalSkillConflictResolutionDto, LocalSkillCopySourceDto,
    LocalSkillCopyTargetAgentDto, LocalSkillDetailDto, LocalSkillSummaryDto,
    SkillScanTargetDto, PreviewLocalSkillCopyResultDto,
};
use crate::AppState;

#[tauri::command]
pub fn list_local_skills(
    state: State<'_, AppState>,
    scan_targets: Vec<SkillScanTargetDto>,
) -> Result<Vec<LocalSkillSummaryDto>, String> {
    Ok(state.skill_discovery_service.list_local_skills(scan_targets))
}

#[tauri::command]
pub fn get_local_skill_detail(
    state: State<'_, AppState>,
    scan_targets: Vec<SkillScanTargetDto>,
    skill_id: String,
) -> Result<LocalSkillDetailDto, String> {
    Ok(state.skill_discovery_service.get_local_skill_detail(scan_targets, &skill_id)?)
}

#[tauri::command]
pub fn set_local_skill_enabled(
    state: State<'_, AppState>,
    skill_path: String,
    entry_file_path: String,
    enabled: bool,
) -> Result<(), String> {
    Ok(state.skill_operations_service.set_skill_enabled(&skill_path, &entry_file_path, enabled)?)
}

#[tauri::command]
pub fn open_skill_folder(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    skill_path: String,
) -> Result<(), String> {
    Ok(state.skill_operations_service.open_skill_folder(app, &skill_path)?)
}

#[tauri::command]
pub fn open_skill_entry_file(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    skill_path: String,
    entry_file_path: String,
) -> Result<(), String> {
    Ok(state.skill_operations_service.open_skill_entry_file(app, &skill_path, &entry_file_path)?)
}

#[tauri::command]
pub fn delete_local_skill(
    state: State<'_, AppState>,
    skill_path: String,
    entry_file_path: String,
) -> Result<(), String> {
    Ok(state.skill_operations_service.delete_skill(&skill_path, &entry_file_path)?)
}

#[tauri::command]
pub fn preview_local_skill_copy(
    state: State<'_, AppState>,
    sources: Vec<LocalSkillCopySourceDto>,
    target_agent: LocalSkillCopyTargetAgentDto,
) -> Result<PreviewLocalSkillCopyResultDto, String> {
    Ok(state.skill_operations_service.preview_skill_copy(sources, target_agent)?)
}

#[tauri::command]
pub fn copy_local_skills(
    state: State<'_, AppState>,
    sources: Vec<LocalSkillCopySourceDto>,
    target_agent: LocalSkillCopyTargetAgentDto,
    resolutions: Vec<LocalSkillConflictResolutionDto>,
) -> Result<CopyLocalSkillsResultDto, String> {
    Ok(state.skill_operations_service.execute_skill_copy(sources, target_agent, resolutions)?)
}
