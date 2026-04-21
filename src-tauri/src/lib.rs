mod commands;
mod constants;
mod dto;
mod persistence;
mod plugins;
mod scanners;
mod services;
mod utils;

use tauri::Manager;

#[tauri::command]
fn update_tray_menu(
    app: tauri::AppHandle,
    show_text: String,
    quit_text: String,
) -> Result<(), String> {
    plugins::system_tray::update_tray_menu(&app, &show_text, &quit_text)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // When attempting to start a second instance, focus the existing main window
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
        .setup(|app| {
            // Register cleanup on main window close
            if let Some(window) = app.get_webview_window("main") {
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        // Cleanup inspector process before exit
                        tauri::async_runtime::block_on(async {
                            let _ = commands::mcp::cleanup_inspector_on_exit().await;
                        });
                    }
                });
            }
            Ok(())
        })
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

    // Only enable updater in release mode
    #[cfg(not(debug_assertions))]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
