mod commands;
pub mod constants;
pub mod dto;
pub mod infrastructure;
mod plugins;
pub mod repositories;
pub mod scanners;
pub mod services;

use std::sync::Arc;
use tauri::Manager;

use crate::infrastructure::persistence::{JsonAgentRepository, JsonMarketplaceInstallRepository};
use crate::services::agent_discovery_service::AgentDiscoveryService;
use crate::services::marketplace_service::MarketplaceService;
use crate::services::mcp_service::McpService;
use crate::services::skill_discovery_service::SkillDiscoveryService;
use crate::services::skill_operations_service::SkillOperationsService;

pub struct AppState {
    pub agent_discovery_service: AgentDiscoveryService,
    pub skill_discovery_service: SkillDiscoveryService,
    pub skill_operations_service: SkillOperationsService,
    pub mcp_service: McpService,
    pub marketplace_service: MarketplaceService,
}

impl AppState {
    pub fn new() -> Self {
        let agent_repo = Arc::new(JsonAgentRepository::new());
        let install_repo = Arc::new(JsonMarketplaceInstallRepository::new());

        Self {
            agent_discovery_service: AgentDiscoveryService::new(agent_repo.clone()),
            skill_discovery_service: SkillDiscoveryService::new(install_repo.clone()),
            skill_operations_service: SkillOperationsService::new(install_repo.clone()),
            mcp_service: McpService::new(),
            marketplace_service: MarketplaceService::new(),
            // Implementations can be injected here when needed.
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command]
fn update_tray_menu(
    app: tauri::AppHandle,
    show_text: String,
    quit_text: String,
) -> Result<(), String> {
    plugins::system_tray::update_tray_menu(&app, &show_text, &quit_text)
}

fn setup_window_handlers(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                tauri::async_runtime::block_on(async {
                    let _ = commands::mcp::cleanup_inspector_on_exit().await;
                });
            }
        });
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new();

    let builder = tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
                let _ = window.show();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(plugins::system_tray::init())
        .setup(setup_window_handlers)
        .invoke_handler(tauri::generate_handler![
            update_tray_menu,
            commands::agents::list_managed_agents,
            commands::agents::list_resolved_agents,
            commands::agents::scan_agents,
            commands::agents::import_agents,
            commands::agents::remove_managed_agent,
            commands::agents::delete_agent,
            commands::agents::create_agent,
            commands::agents::refresh_agent_discovery,
            commands::mcp::list_local_mcps,
            commands::mcp::open_mcp_config_folder,
            commands::mcp::open_mcp_config_file,
            commands::mcp::get_local_mcp_edit_data,
            commands::mcp::update_local_mcp,
            commands::mcp::delete_local_mcp,
            commands::mcp::import_local_mcp_json,
            commands::mcp::launch_mcp_inspector,
            commands::mcp::stop_mcp_inspector,
            commands::marketplace::fetch_skillssh_leaderboard,
            commands::marketplace::get_skillssh_marketplace_detail,
            commands::marketplace::preview_skillssh_marketplace_install,
            commands::marketplace::install_skillssh_marketplace_item,
            commands::marketplace::check_local_marketplace_skill_update,
            commands::marketplace::search_skillssh_marketplace,
            commands::skills::list_local_skills,
            commands::skills::get_local_skill_detail,
            commands::skills::set_local_skill_enabled,
            commands::skills::open_skill_folder,
            commands::skills::open_skill_entry_file,
            commands::skills::delete_local_skill,
            commands::skills::preview_local_skill_copy,
            commands::skills::copy_local_skills
        ]);

    #[cfg(not(debug_assertions))]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
