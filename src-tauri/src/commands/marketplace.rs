use std::{env, fs, path::PathBuf};

use tauri::{AppHandle, Manager};

use crate::dto::marketplace::{
    InstallMarketplaceSkillRequestDto, MarketplaceInstallPreviewDto, MarketplaceInstallResultDto,
    MarketplaceItemDto, MarketplaceSkillDetailDto, MarketplaceSkillUpdateCheckDto,
};
use crate::persistence::marketplace_install_store::{self, MarketplaceInstallRecord};
use crate::scanners::agent_type_scanner;
use crate::scanners::skillssh_scanner::{self, LeaderboardType, SkillsShSkillFileRecord};
use crate::services::marketplace_service;

#[tauri::command]
pub async fn fetch_skillssh_leaderboard(
    board: Option<String>,
) -> Result<Vec<MarketplaceItemDto>, String> {
    let board_value = board.unwrap_or_else(|| "hot".to_string());

    tauri::async_runtime::spawn_blocking(move || {
        skillssh_scanner::fetch_leaderboard(LeaderboardType::from_str(&board_value)).map(|skills| {
            skills
                .into_iter()
                .map(marketplace_service::to_skillssh_skill_dto)
                .map(marketplace_service::to_marketplace_item)
                .collect()
        })
    })
    .await
    .map_err(|error| format!("Failed to join skills.sh leaderboard task: {error}"))?
}

#[tauri::command]
pub async fn search_skillssh_marketplace(
    query: String,
    limit: Option<usize>,
) -> Result<Vec<MarketplaceItemDto>, String> {
    let bounded_limit = limit.unwrap_or(60).clamp(1, 100);

    tauri::async_runtime::spawn_blocking(move || {
        skillssh_scanner::search_skills(&query, bounded_limit).map(|skills| {
            skills
                .into_iter()
                .map(marketplace_service::to_skillssh_skill_dto)
                .map(marketplace_service::to_marketplace_item)
                .collect()
        })
    })
    .await
    .map_err(|error| format!("Failed to join skills.sh search task: {error}"))?
}

#[tauri::command]
pub async fn get_skillssh_marketplace_detail(
    app: AppHandle,
    source: String,
    skill_id: String,
) -> Result<MarketplaceSkillDetailDto, String> {
    let cache_root_dir = app
        .path()
        .app_cache_dir()
        .map_err(|error| format!("Failed to resolve marketplace cache directory: {error}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        skillssh_scanner::fetch_skill_detail(&cache_root_dir, &source, &skill_id)
            .map(marketplace_service::to_marketplace_skill_detail)
    })
    .await
    .map_err(|error| format!("Failed to join skills.sh detail task: {error}"))?
}

fn user_home_dir() -> PathBuf {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn resolve_agent_root_path(root_path: &str) -> PathBuf {
    if let Some(relative_path) = root_path
        .strip_prefix("~/")
        .or_else(|| root_path.strip_prefix("~\\"))
    {
        return user_home_dir().join(relative_path);
    }

    let path = PathBuf::from(root_path);
    if path.is_absolute() {
        return path;
    }

    user_home_dir().join(path)
}

fn remove_existing_skill_path(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|error| format!("Failed to remove existing skill: {error}"))
    } else {
        fs::remove_file(path)
            .map_err(|error| format!("Failed to remove existing skill file: {error}"))
    }
}

fn resolve_existing_skill_entry_path(entry_file_path: &str) -> Result<PathBuf, String> {
    let active_entry_path = PathBuf::from(entry_file_path);
    let disabled_entry_path = PathBuf::from(format!("{entry_file_path}.disabled"));

    let active_exists = active_entry_path.is_file();
    let disabled_exists = disabled_entry_path.is_file();

    match (active_exists, disabled_exists) {
        (true, false) => Ok(active_entry_path),
        (false, true) => Ok(disabled_entry_path),
        (false, false) => Err(format!("Skill entry file not found: {entry_file_path}")),
        (true, true) => Err(format!(
            "Skill entry files conflict: {} and {}",
            active_entry_path.display(),
            disabled_entry_path.display()
        )),
    }
}

fn resolve_marketplace_install_paths(
    request: &InstallMarketplaceSkillRequestDto,
) -> Result<(PathBuf, PathBuf), String> {
    let agent_root = resolve_agent_root_path(&request.target_agent.root_path);
    let skills_root =
        agent_type_scanner::build_skill_scan_root(&request.target_agent.agent_type, &agent_root)
            .ok_or_else(|| {
                format!(
                    "Agent type {} does not support skills.",
                    request.target_agent.agent_type
                )
            })?;
    Ok(marketplace_service::marketplace_skill_paths(
        &skills_root,
        &request.skill_id,
    ))
}

fn ensure_parent_dir(path: &std::path::Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };

    fs::create_dir_all(parent)
        .map_err(|error| format!("Failed to create parent directory: {error}"))
}

fn write_marketplace_skill_bundle(
    skill_path: &std::path::Path,
    files: &[SkillsShSkillFileRecord],
) -> Result<(), String> {
    fs::create_dir_all(skill_path)
        .map_err(|error| format!("Failed to create marketplace skill directory: {error}"))?;

    for file in files {
        let destination_path = skill_path.join(&file.relative_path);
        ensure_parent_dir(&destination_path)?;
        fs::write(&destination_path, &file.contents)
            .map_err(|error| format!("Failed to write marketplace skill file: {error}"))?;
    }

    Ok(())
}

fn normalize_local_bundle_relative_path(path: &str) -> Result<String, String> {
    let normalized = path.replace('\\', "/");
    if normalized == "SKILL.md.disabled" {
        return Ok("SKILL.md".into());
    }

    if let Some(prefix) = normalized.strip_suffix("/SKILL.md.disabled") {
        return Ok(format!("{prefix}/SKILL.md"));
    }

    if normalized.ends_with(".disabled") {
        return Err(format!(
            "Unsupported disabled marketplace file detected: {normalized}"
        ));
    }

    Ok(normalized)
}

fn collect_local_bundle_files(
    root: &std::path::Path,
    current: &std::path::Path,
    files: &mut std::collections::HashMap<String, Vec<u8>>,
) -> Result<(), String> {
    let entries = fs::read_dir(current)
        .map_err(|error| format!("Failed to read local marketplace skill directory: {error}"))?;

    for entry in entries {
        let entry = entry
            .map_err(|error| format!("Failed to read local marketplace skill entry: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_local_bundle_files(root, &path, files)?;
            continue;
        }

        let relative_path = path
            .strip_prefix(root)
            .map_err(|error| format!("Failed to normalize local marketplace skill path: {error}"))?
            .to_string_lossy()
            .replace('\\', "/");
        let normalized_relative_path = normalize_local_bundle_relative_path(&relative_path)?;
        if files.contains_key(&normalized_relative_path) {
            return Err(format!(
                "Conflicting local marketplace files found for {normalized_relative_path}"
            ));
        }
        let contents = fs::read(&path)
            .map_err(|error| format!("Failed to read local marketplace skill file: {error}"))?;
        files.insert(normalized_relative_path, contents);
    }

    Ok(())
}

fn local_bundle_matches_remote(
    skill_path: &std::path::Path,
    remote_files: &[SkillsShSkillFileRecord],
) -> Result<bool, String> {
    if !skill_path.exists() {
        return Ok(false);
    }

    let mut local_files = std::collections::HashMap::new();
    collect_local_bundle_files(skill_path, skill_path, &mut local_files)?;

    let mut remote_file_count = 0usize;
    for remote_file in remote_files {
        remote_file_count += 1;
        let Some(local_contents) = local_files.remove(&remote_file.relative_path) else {
            return Ok(false);
        };
        if local_contents != remote_file.contents {
            return Ok(false);
        }
    }

    Ok(remote_file_count > 0 && local_files.is_empty())
}

#[tauri::command]
pub async fn preview_skillssh_marketplace_install(
    request: InstallMarketplaceSkillRequestDto,
) -> Result<MarketplaceInstallPreviewDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (skill_path, entry_file_path) = resolve_marketplace_install_paths(&request)?;
        let has_conflict = skill_path.exists() || entry_file_path.exists();
        let existing_path = if has_conflict {
            Some(skill_path.to_string_lossy().replace('\\', "/"))
        } else {
            None
        };

        Ok(MarketplaceInstallPreviewDto {
            skill_path: skill_path.to_string_lossy().replace('\\', "/"),
            entry_file_path: entry_file_path.to_string_lossy().replace('\\', "/"),
            has_conflict,
            existing_path,
        })
    })
    .await
    .map_err(|error| format!("Failed to join marketplace install preview task: {error}"))?
}

#[tauri::command]
pub async fn install_skillssh_marketplace_item(
    app: AppHandle,
    request: InstallMarketplaceSkillRequestDto,
) -> Result<MarketplaceInstallResultDto, String> {
    let cache_root_dir = app
        .path()
        .app_cache_dir()
        .map_err(|error| format!("Failed to resolve marketplace cache directory: {error}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        let bundle = skillssh_scanner::fetch_skill_bundle(
            &cache_root_dir,
            &request.source,
            &request.skill_id,
        )?;
        let (skill_path, entry_file_path) = resolve_marketplace_install_paths(&request)?;

        if entry_file_path.exists() || skill_path.exists() {
            if !request.overwrite {
                return Err(format!(
                    "Marketplace skill already exists at {}",
                    skill_path.display()
                ));
            }

            remove_existing_skill_path(&skill_path)?;
        }

        if let Err(error) = write_marketplace_skill_bundle(&skill_path, &bundle.files) {
            let _ = fs::remove_dir_all(&skill_path);
            return Err(error);
        }

        marketplace_install_store::upsert_marketplace_install(MarketplaceInstallRecord {
            source: request.source,
            skill_id: request.skill_id,
            skill_path: skill_path.to_string_lossy().replace('\\', "/"),
            entry_file_path: entry_file_path.to_string_lossy().replace('\\', "/"),
            installed_at: chrono::Utc::now().to_rfc3339(),
        })?;

        Ok(MarketplaceInstallResultDto {
            skill_path: skill_path.to_string_lossy().replace('\\', "/"),
            entry_file_path: entry_file_path.to_string_lossy().replace('\\', "/"),
        })
    })
    .await
    .map_err(|error| format!("Failed to join marketplace install task: {error}"))?
}

#[tauri::command]
pub async fn check_local_marketplace_skill_update(
    app: AppHandle,
    skill_path: String,
    entry_file_path: String,
) -> Result<MarketplaceSkillUpdateCheckDto, String> {
    let records = marketplace_install_store::load_marketplace_installs();
    let record = records.into_iter().find(|record| {
        record.skill_path == skill_path || record.entry_file_path == entry_file_path
    });

    let Some(record) = record else {
        return Ok(MarketplaceSkillUpdateCheckDto {
            managed: false,
            has_update: false,
            source: None,
            skill_id: None,
        });
    };

    let cache_root_dir = app
        .path()
        .app_cache_dir()
        .map_err(|error| format!("Failed to resolve marketplace cache directory: {error}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        let _local_entry_path = resolve_existing_skill_entry_path(&entry_file_path)?;
        let remote_bundle = skillssh_scanner::fetch_skill_bundle(
            &cache_root_dir,
            &record.source,
            &record.skill_id,
        )?;

        Ok(MarketplaceSkillUpdateCheckDto {
            managed: true,
            has_update: !local_bundle_matches_remote(
                &PathBuf::from(&skill_path),
                &remote_bundle.files,
            )?,
            source: Some(record.source),
            skill_id: Some(record.skill_id),
        })
    })
    .await
    .map_err(|error| format!("Failed to join marketplace update check task: {error}"))?
}

#[cfg(test)]
mod tests {
    use super::{local_bundle_matches_remote, normalize_local_bundle_relative_path};
    use crate::scanners::skillssh_scanner::SkillsShSkillFileRecord;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-dock-marketplace-{name}-{unique}"))
    }

    #[test]
    fn normalize_local_bundle_relative_path_maps_disabled_skill_entry() {
        assert_eq!(
            normalize_local_bundle_relative_path("nested/SKILL.md.disabled").expect("normalize"),
            "nested/SKILL.md"
        );
    }

    #[test]
    fn local_bundle_matches_remote_accepts_disabled_entry_file() {
        let root = temp_dir("disabled-entry");
        fs::create_dir_all(root.join("nested")).expect("create temp tree");
        fs::write(root.join("SKILL.md.disabled"), b"# Demo").expect("write disabled entry");
        fs::write(root.join("nested/config.json"), br#"{"ok":true}"#).expect("write config");

        let remote_files = vec![
            SkillsShSkillFileRecord {
                relative_path: "SKILL.md".into(),
                contents: b"# Demo".to_vec(),
            },
            SkillsShSkillFileRecord {
                relative_path: "nested/config.json".into(),
                contents: br#"{"ok":true}"#.to_vec(),
            },
        ];

        let matches = local_bundle_matches_remote(&root, &remote_files).expect("compare bundle");
        assert!(matches);

        let _ = fs::remove_dir_all(root);
    }
}
