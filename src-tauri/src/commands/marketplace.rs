use std::str::FromStr;
use std::{fs, path::PathBuf};

use tauri::{AppHandle, Manager, State};

use crate::dto::marketplace::{
    InstallMarketplaceSkillRequestDto, MarketplaceInstallMethodDto, MarketplaceInstallPreviewDto,
    MarketplaceInstallResultDto, MarketplaceItemsResponseDto, MarketplaceSkillDetailDto,
    MarketplaceSkillUpdateCheckDto,
};
use crate::infrastructure::utils::path::resolve_agent_root;
use crate::AppState;
use crate::repositories::marketplace_install_repository::MarketplaceInstallRecord;
use crate::scanners::skillssh_scanner::{
    self, LeaderboardType, MarketplaceInstallMethod, SkillsShSkillFileRecord,
};
use crate::services::ServiceError;

fn to_install_method(method: &MarketplaceInstallMethodDto) -> MarketplaceInstallMethod {
    match method {
        MarketplaceInstallMethodDto::Skillsh => MarketplaceInstallMethod::SkillsSh,
        MarketplaceInstallMethodDto::Github => MarketplaceInstallMethod::GitHub,
    }
}

#[tauri::command]
pub async fn fetch_skillssh_leaderboard(
    state: State<'_, AppState>,
    board: Option<String>,
    page: Option<usize>,
) -> Result<MarketplaceItemsResponseDto, String> {
    let board_value = board.unwrap_or_else(|| "all-time".to_string());
    let page_value = page.unwrap_or(0);
    let marketplace_service = state.marketplace_service.clone();

    let result = tauri::async_runtime::spawn_blocking(move || -> Result<MarketplaceItemsResponseDto, ServiceError> {
        let skills = skillssh_scanner::fetch_leaderboard(LeaderboardType::from_str(&board_value).unwrap_or(LeaderboardType::AllTime), page_value)
            .map_err(ServiceError::Scanner)?;

        Ok(MarketplaceItemsResponseDto {
            items: skills
                .items
                .into_iter()
                .map(|s| marketplace_service.to_skillssh_skill_dto(s))
                .map(|s| marketplace_service.to_marketplace_item(s))
                .collect(),
            total_skills: skills.total_skills,
            has_more: skills.has_more,
            page: skills.page,
        })
    })
    .await
    .map_err(|error| ServiceError::Internal(format!("Failed to join skills.sh leaderboard task: {error}")))?;

    Ok(result?)
}

#[tauri::command]
pub async fn search_skillssh_marketplace(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
    page: Option<usize>,
) -> Result<MarketplaceItemsResponseDto, String> {
    let bounded_limit = limit.unwrap_or(100).clamp(1, 100);
    let page_value = page.unwrap_or(0);
    let marketplace_service = state.marketplace_service.clone();

    let result = tauri::async_runtime::spawn_blocking(move || -> Result<MarketplaceItemsResponseDto, ServiceError> {
        let skills = skillssh_scanner::search_skills(&query, bounded_limit, page_value)
            .map_err(ServiceError::Scanner)?;

        Ok(MarketplaceItemsResponseDto {
            items: skills
                .items
                .into_iter()
                .map(|s| marketplace_service.to_skillssh_skill_dto(s))
                .map(|s| marketplace_service.to_marketplace_item(s))
                .collect(),
            total_skills: skills.total_skills,
            has_more: skills.has_more,
            page: skills.page,
        })
    })
    .await
    .map_err(|error| ServiceError::Internal(format!("Failed to join skills.sh search task: {error}")))?;

    Ok(result?)
}

#[tauri::command]
pub async fn get_skillssh_marketplace_detail(
    state: State<'_, AppState>,
    app: AppHandle,
    source: String,
    skill_id: String,
) -> Result<MarketplaceSkillDetailDto, String> {
    let cache_root_dir = app
        .path()
        .app_cache_dir()
        .map_err(|error| ServiceError::Internal(format!("Failed to resolve marketplace cache directory: {error}")))?;
    let marketplace_service = state.marketplace_service.clone();

    let result = tauri::async_runtime::spawn_blocking(move || -> Result<MarketplaceSkillDetailDto, ServiceError> {
        skillssh_scanner::fetch_skill_detail(&cache_root_dir, &source, &skill_id)
            .map_err(ServiceError::Scanner)
            .map(|d| marketplace_service.to_marketplace_skill_detail(d))
    })
    .await
    .map_err(|error| ServiceError::Internal(format!("Failed to join skills.sh detail task: {error}")))?;

    Ok(result?)
}

#[tauri::command]
pub async fn preview_skillssh_marketplace_install(
    state: State<'_, AppState>,
    request: InstallMarketplaceSkillRequestDto,
) -> Result<MarketplaceInstallPreviewDto, String> {
    let marketplace_service = state.marketplace_service.clone();
    let result = tauri::async_runtime::spawn_blocking(move || -> Result<MarketplaceInstallPreviewDto, ServiceError> {
        let (skill_path, entry_file_path) = marketplace_service.marketplace_skill_paths(
            &resolve_agent_root(&request.target_agent.root_path),
            &request.skill_id,
        );
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
    .map_err(|error| ServiceError::Internal(format!("Failed to join marketplace install preview task: {error}")))?;

    Ok(result?)
}

#[tauri::command]
pub async fn install_skillssh_marketplace_item(
    state: State<'_, AppState>,
    app: AppHandle,
    request: InstallMarketplaceSkillRequestDto,
) -> Result<MarketplaceInstallResultDto, String> {
    let cache_root_dir = app
        .path()
        .app_cache_dir()
        .map_err(|error| ServiceError::Internal(format!("Failed to resolve marketplace cache directory: {error}")))?;
    let marketplace_service = state.marketplace_service.clone();
    let install_repo = state.skill_operations_service.install_repo.clone();

    let result = tauri::async_runtime::spawn_blocking(move || -> Result<MarketplaceInstallResultDto, ServiceError> {
        let bundle = skillssh_scanner::fetch_skill_bundle(
            &cache_root_dir,
            &request.source,
            &request.skill_id,
            to_install_method(&request.install_method),
        ).map_err(ServiceError::Scanner)?;
        let (skill_path, entry_file_path) = marketplace_service.marketplace_skill_paths(
            &resolve_agent_root(&request.target_agent.root_path),
            &request.skill_id,
        );

        if entry_file_path.exists() || skill_path.exists() {
            if !request.overwrite {
                return Err(ServiceError::BusinessRuleViolation(format!(
                    "Marketplace skill already exists at {}",
                    skill_path.display()
                )));
            }

            remove_existing_skill_path(&skill_path)?;
        }

        if let Err(error) = write_marketplace_skill_bundle(&skill_path, &bundle.files) {
            let _ = fs::remove_dir_all(&skill_path);
            return Err(error);
        }

        install_repo.upsert(MarketplaceInstallRecord {
            source: request.source,
            skill_id: request.skill_id,
            install_method: match request.install_method {
                MarketplaceInstallMethodDto::Skillsh => "skillsh".into(),
                MarketplaceInstallMethodDto::Github => "github".into(),
            },
            skill_path: skill_path.to_string_lossy().replace('\\', "/"),
            entry_file_path: entry_file_path.to_string_lossy().replace('\\', "/"),
            installed_at: chrono::Utc::now().to_rfc3339(),
        }).map_err(ServiceError::from)?;

        Ok(MarketplaceInstallResultDto {
            skill_path: skill_path.to_string_lossy().replace('\\', "/"),
            entry_file_path: entry_file_path.to_string_lossy().replace('\\', "/"),
        })
    })
    .await
    .map_err(|error| ServiceError::Internal(format!("Failed to join marketplace install task: {error}")))?;

    Ok(result?)
}

#[tauri::command]
pub async fn check_local_marketplace_skill_update(
    state: State<'_, AppState>,
    app: AppHandle,
    skill_path: String,
    entry_file_path: String,
) -> Result<MarketplaceSkillUpdateCheckDto, String> {
    let install_repo = state.skill_operations_service.install_repo.clone();
    let records = install_repo.find_all();
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
        .map_err(|error| ServiceError::Internal(format!("Failed to resolve marketplace cache directory: {error}")))?;

    let result = tauri::async_runtime::spawn_blocking(move || -> Result<MarketplaceSkillUpdateCheckDto, ServiceError> {
        let _local_entry_path = resolve_existing_skill_entry_path(&entry_file_path)?;
        let remote_bundle = skillssh_scanner::fetch_skill_bundle(
            &cache_root_dir,
            &record.source,
            &record.skill_id,
            if record.install_method == "github" {
                MarketplaceInstallMethod::GitHub
            } else {
                MarketplaceInstallMethod::SkillsSh
            },
        ).map_err(ServiceError::Scanner)?;

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
    .map_err(|error| ServiceError::Internal(format!("Failed to join marketplace update check task: {error}")))?;

    Ok(result?)
}

fn remove_existing_skill_path(path: &PathBuf) -> Result<(), ServiceError> {
    if !path.exists() {
        return Ok(());
    }

    let remove_result = if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };

    remove_result.map_err(|error| ServiceError::Internal(format!("Failed to remove existing skill: {error}")))
}

fn resolve_existing_skill_entry_path(entry_file_path: &str) -> Result<PathBuf, ServiceError> {
    let active_entry_path = PathBuf::from(entry_file_path);
    let disabled_entry_path = PathBuf::from(format!("{entry_file_path}.disabled"));

    let active_exists = active_entry_path.is_file();
    let disabled_exists = disabled_entry_path.is_file();

    match (active_exists, disabled_exists) {
        (true, false) => Ok(active_entry_path),
        (false, true) => Ok(disabled_entry_path),
        (false, false) => Err(ServiceError::Internal(format!("Skill entry file not found: {entry_file_path}"))),
        (true, true) => Err(ServiceError::Internal(format!(
            "Skill entry files conflict: {} and {}",
            active_entry_path.display(),
            disabled_entry_path.display()
        ))),
    }
}

fn ensure_parent_dir_local(path: &std::path::Path) -> Result<(), ServiceError> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };

    fs::create_dir_all(parent)
        .map_err(|error| ServiceError::Internal(format!("Failed to create parent directory: {error}")))
}

fn write_marketplace_skill_bundle(
    skill_path: &std::path::Path,
    files: &[SkillsShSkillFileRecord],
) -> Result<(), ServiceError> {
    fs::create_dir_all(skill_path)
        .map_err(|error| ServiceError::Internal(format!("Failed to create marketplace skill directory: {error}")))?;

    for file in files {
        let destination_path = skill_path.join(&file.relative_path);
        ensure_parent_dir_local(&destination_path)?;
        fs::write(&destination_path, &file.contents)
            .map_err(|error| ServiceError::Internal(format!("Failed to write marketplace skill file: {error}")))?;
    }

    Ok(())
}

fn normalize_local_bundle_relative_path(path: &str) -> Result<String, ServiceError> {
    let normalized = path.replace('\\', "/");
    if normalized == "SKILL.md.disabled" {
        return Ok("SKILL.md".into());
    }

    if let Some(prefix) = normalized.strip_suffix("/SKILL.md.disabled") {
        return Ok(format!("{prefix}/SKILL.md"));
    }

    if normalized.ends_with(".disabled") {
        return Err(ServiceError::Internal(format!(
            "Unsupported disabled marketplace file detected: {normalized}"
        )));
    }

    Ok(normalized)
}

fn collect_local_bundle_files(
    root: &std::path::Path,
    current: &std::path::Path,
    files: &mut std::collections::HashMap<String, Vec<u8>>,
) -> Result<(), ServiceError> {
    let entries = fs::read_dir(current)
        .map_err(|error| ServiceError::Internal(format!("Failed to read local marketplace skill directory: {error}")))?;

    for entry in entries {
        let entry = entry
            .map_err(|error| ServiceError::Internal(format!("Failed to read local marketplace skill entry: {error}")))?;
        let path = entry.path();
        if path.is_dir() {
            collect_local_bundle_files(root, &path, files)?;
            continue;
        }

        let relative_path = path
            .strip_prefix(root)
            .map_err(|error| ServiceError::Internal(format!("Failed to normalize local marketplace skill path: {error}")))?
            .to_string_lossy()
            .replace('\\', "/");
        let normalized_relative_path = normalize_local_bundle_relative_path(&relative_path)?;
        if files.contains_key(&normalized_relative_path) {
            return Err(ServiceError::Internal(format!(
                "Conflicting local marketplace files found for {normalized_relative_path}"
            )));
        }
        let contents = fs::read(&path)
            .map_err(|error| ServiceError::Internal(format!("Failed to read local marketplace skill file: {error}")))?;
        files.insert(normalized_relative_path, contents);
    }

    Ok(())
}

fn local_bundle_matches_remote(
    skill_path: &std::path::Path,
    remote_files: &[SkillsShSkillFileRecord],
) -> Result<bool, ServiceError> {
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
