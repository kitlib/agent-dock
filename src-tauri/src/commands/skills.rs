use std::path::Path;

use tauri_plugin_opener::OpenerExt;

use crate::dto::skills::{LocalSkillDetailDto, LocalSkillSummaryDto, SkillScanTargetDto};
use crate::services::skill_discovery_service;

#[tauri::command]
pub fn list_local_skills(scan_targets: Vec<SkillScanTargetDto>) -> Result<Vec<LocalSkillSummaryDto>, String> {
    println!(
        "[skills] list_local_skills command targets: {:?}",
        scan_targets
            .iter()
            .map(|target| format!("{}|{}|{}", target.agent_id, target.provider, target.root_path))
            .collect::<Vec<_>>()
    );

    let skills = skill_discovery_service::list_local_skills(scan_targets);

    println!(
        "[skills] list_local_skills command result: {:?}",
        skills
            .iter()
            .map(|skill| format!(
                "{}|owner={}|provider={}|path={}",
                skill.id, skill.owner_agent_id, skill.provider, skill.skill_path
            ))
            .collect::<Vec<_>>()
    );

    Ok(skills)
}

#[tauri::command]
pub fn get_local_skill_detail(
    scan_targets: Vec<SkillScanTargetDto>,
    skill_id: String,
) -> Result<LocalSkillDetailDto, String> {
    println!(
        "[skills] get_local_skill_detail command request: skill_id={}, targets={:?}",
        skill_id,
        scan_targets
            .iter()
            .map(|target| format!("{}|{}|{}", target.agent_id, target.provider, target.root_path))
            .collect::<Vec<_>>()
    );

    let detail = skill_discovery_service::get_local_skill_detail(scan_targets, &skill_id)?;

    println!(
        "[skills] get_local_skill_detail command result: {}|owner={}|provider={}|path={}",
        detail.id, detail.owner_agent_id, detail.provider, detail.skill_path
    );

    Ok(detail)
}

#[tauri::command]
pub fn open_skill_folder(app: tauri::AppHandle, skill_path: String) -> Result<(), String> {
    let path = Path::new(&skill_path);
    if !path.exists() {
        return Err(format!("Skill folder not found: {skill_path}"));
    }

    if !path.is_dir() {
        return Err(format!("Skill path is not a directory: {skill_path}"));
    }

    app.opener()
        .open_path(&skill_path, None::<&str>)
        .map_err(|error: tauri_plugin_opener::Error| error.to_string())
}
