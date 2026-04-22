use tauri::State;

use crate::dto::mcp::{
    EditableLocalMcpDto, ImportLocalMcpResultDto, LocalMcpServerDto,
    McpScanTargetDto, UpdateLocalMcpResultDto,
};
use crate::AppState;

#[tauri::command]
pub fn list_local_mcps(
    state: State<'_, AppState>,
    scan_targets: Vec<McpScanTargetDto>,
) -> Result<Vec<LocalMcpServerDto>, String> {
    Ok(state.mcp_service.list_local_mcps(scan_targets)?)
}

#[tauri::command]
pub fn open_mcp_config_folder(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    config_path: String,
) -> Result<(), String> {
    Ok(state.mcp_service.open_mcp_config_folder(app, &config_path)?)
}

#[tauri::command]
pub fn open_mcp_config_file(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    config_path: String,
) -> Result<(), String> {
    Ok(state.mcp_service.open_mcp_config_file(app, &config_path)?)
}

#[tauri::command]
pub fn get_local_mcp_edit_data(
    state: State<'_, AppState>,
    agent_type: String,
    config_path: String,
    server_name: String,
    scope: String,
    project_path: Option<String>,
) -> Result<EditableLocalMcpDto, String> {
    Ok(state.mcp_service.get_local_mcp_edit_data(agent_type, config_path, server_name, scope, project_path)?)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn update_local_mcp(
    state: State<'_, AppState>,
    agent_type: String,
    config_path: String,
    server_name: String,
    scope: String,
    project_path: Option<String>,
    next_server_name: String,
    transport: String,
    command: Option<String>,
    args: Vec<String>,
    env: std::collections::BTreeMap<String, String>,
    url: Option<String>,
    headers: std::collections::BTreeMap<String, String>,
) -> Result<UpdateLocalMcpResultDto, String> {
    Ok(state.mcp_service.update_local_mcp(
        agent_type,
        config_path,
        server_name,
        scope,
        project_path,
        next_server_name,
        transport,
        command,
        args,
        env,
        url,
        headers,
    )?)
}

#[tauri::command]
pub fn delete_local_mcp(
    state: State<'_, AppState>,
    agent_type: String,
    config_path: String,
    server_name: String,
    scope: String,
    project_path: Option<String>,
) -> Result<(), String> {
    Ok(state.mcp_service.delete_local_mcp(agent_type, config_path, server_name, scope, project_path)?)
}

#[tauri::command]
pub fn import_local_mcp_json(
    state: State<'_, AppState>,
    agent_type: String,
    root_path: String,
    json_payload: String,
    conflict_strategy: String,
) -> Result<ImportLocalMcpResultDto, String> {
    Ok(state.mcp_service.import_local_mcp_json(agent_type, root_path, json_payload, conflict_strategy)?)
}

#[tauri::command]
pub async fn launch_mcp_inspector(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    config: EditableLocalMcpDto,
) -> Result<(), String> {
    state.mcp_service.launch_mcp_inspector(app, config).await
}

#[tauri::command]
pub async fn stop_mcp_inspector(state: State<'_, AppState>,) -> Result<(), String> {
    state.mcp_service.stop_mcp_inspector().await
}

/// Cleanup inspector process on application exit
pub async fn cleanup_inspector_on_exit() -> Result<(), String> {
    crate::infrastructure::mcp::McpInspectorManager::cleanup_on_exit().await
}
