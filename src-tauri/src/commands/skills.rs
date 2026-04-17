use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
};

use tauri_plugin_opener::OpenerExt;

use crate::dto::skills::{
    CopyLocalSkillsResultDto, LocalSkillConflictResolutionDto, LocalSkillCopyConflictDto,
    LocalSkillCopySourceDto, LocalSkillCopyTargetAgentDto, LocalSkillDetailDto,
    LocalSkillSummaryDto, PreviewLocalSkillCopyResultDto, SkillScanTargetDto,
};
use crate::scanners::agent_type_scanner;
use crate::services::skill_discovery_service;

const DISABLED_SUFFIX: &str = ".disabled";
const SKILLS_SOURCE: &str = "skills";
const COMMANDS_SOURCE: &str = "commands";
const OVERWRITE_ACTION: &str = "overwrite";
const SKIP_ACTION: &str = "skip";

#[derive(Clone)]
struct PlannedSkillCopy {
    source: LocalSkillCopySourceDto,
    source_path: PathBuf,
    destination_path: PathBuf,
    existing_path: Option<PathBuf>,
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
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

fn entry_file_name(entry_path: &Path, entry_file_path: &str) -> Result<String, String> {
    entry_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .ok_or_else(|| format!("Invalid skill entry file path: {entry_file_path}"))
}

fn is_disabled_entry(entry_path: &Path) -> bool {
    entry_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.ends_with(DISABLED_SUFFIX))
        .unwrap_or(false)
}

fn enabled_entry_path(entry_path: &Path, entry_file_path: &str) -> Result<PathBuf, String> {
    let entry_name = entry_file_name(entry_path, entry_file_path)?;
    let enabled_name = entry_name.strip_suffix(DISABLED_SUFFIX).unwrap_or(&entry_name);
    Ok(entry_path.with_file_name(enabled_name))
}

fn disabled_entry_path(entry_path: &Path, entry_file_path: &str) -> Result<PathBuf, String> {
    let entry_name = entry_file_name(entry_path, entry_file_path)?;
    Ok(entry_path.with_file_name(format!("{}{}", entry_name, DISABLED_SUFFIX)))
}

fn validate_skill_path(skill_path: &str, canonical_entry_path: &Path) -> Result<(), String> {
    let skill_entry = Path::new(skill_path);
    if !skill_entry.exists() {
        return Err(format!("Skill path not found: {skill_path}"));
    }

    if skill_entry.is_dir() {
        let entry_parent = canonical_entry_path.parent().ok_or_else(|| {
            format!(
                "Skill entry file has no parent directory: {}",
                canonical_entry_path.display()
            )
        })?;
        if entry_parent != skill_entry {
            return Err(format!(
                "Skill entry file does not belong to skill directory: {}",
                canonical_entry_path.display()
            ));
        }
        return Ok(());
    }

    if skill_entry.is_file() {
        let canonical_entry_path_str = canonical_entry_path.to_string_lossy();
        let disabled_entry_candidate =
            disabled_entry_path(canonical_entry_path, canonical_entry_path_str.as_ref())?;
        if skill_entry != canonical_entry_path && skill_entry != disabled_entry_candidate {
            return Err(format!(
                "Skill file path does not match entry file path: {skill_path}"
            ));
        }
        return Ok(());
    }

    Err(format!("Skill path is neither a file nor a directory: {skill_path}"))
}

fn resolve_skill_entry_paths(entry_file_path: &str) -> Result<(PathBuf, PathBuf), String> {
    let entry_path = Path::new(entry_file_path);
    let active_entry_path = enabled_entry_path(entry_path, entry_file_path)?;
    let disabled_entry_path = disabled_entry_path(&active_entry_path, entry_file_path)?;

    Ok((active_entry_path, disabled_entry_path))
}

fn resolve_existing_skill_entry_path(
    skill_path: &str,
    entry_file_path: &str,
) -> Result<PathBuf, String> {
    let (active_entry_path, disabled_entry_path) = resolve_skill_entry_paths(entry_file_path)?;

    validate_skill_path(skill_path, &active_entry_path)?;

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

fn set_local_skill_enabled_at_path(
    skill_path: &str,
    entry_file_path: &str,
    enabled: bool,
) -> Result<(), String> {
    let (active_entry_path, disabled_entry_path) = resolve_skill_entry_paths(entry_file_path)?;

    validate_skill_path(skill_path, &active_entry_path)?;

    let active_exists = active_entry_path.is_file();
    let disabled_exists = disabled_entry_path.is_file();
    if !active_exists && !disabled_exists {
        return Err(format!("Skill entry file not found: {entry_file_path}"));
    }
    if active_exists && disabled_exists {
        let conflict_path = if enabled {
            &active_entry_path
        } else {
            &disabled_entry_path
        };
        return Err(format!(
            "Target entry file already exists: {}",
            conflict_path.display()
        ));
    }

    if enabled {
        if active_exists {
            return Ok(());
        }
        if active_entry_path.exists() {
            return Err(format!(
                "Target entry file already exists: {}",
                active_entry_path.display()
            ));
        }
        return fs::rename(&disabled_entry_path, &active_entry_path)
            .map_err(|error| error.to_string());
    }

    if disabled_exists {
        return Ok(());
    }
    if disabled_entry_path.exists() {
        return Err(format!(
            "Target entry file already exists: {}",
            disabled_entry_path.display()
        ));
    }

    fs::rename(&active_entry_path, &disabled_entry_path).map_err(|error| error.to_string())
}

fn delete_local_skill_at_path(skill_path: &str, entry_file_path: &str) -> Result<(), String> {
    let (active_entry_path, disabled_entry_path) = resolve_skill_entry_paths(entry_file_path)?;

    validate_skill_path(skill_path, &active_entry_path)?;

    let skill_entry = Path::new(skill_path);
    if skill_entry.is_dir() {
        if !active_entry_path.is_file() && !disabled_entry_path.is_file() {
            return Err(format!("Skill entry file not found: {entry_file_path}"));
        }

        return fs::remove_dir_all(skill_entry).map_err(|error| error.to_string());
    }

    let existing_entry_path = resolve_existing_skill_entry_path(skill_path, entry_file_path)?;
    fs::remove_file(existing_entry_path).map_err(|error| error.to_string())
}

fn resolve_copy_target_root(
    target_agent: &LocalSkillCopyTargetAgentDto,
    source_kind: &str,
) -> Result<PathBuf, String> {
    let absolute_root = resolve_agent_root_path(&target_agent.root_path);
    let target_root = match source_kind {
        SKILLS_SOURCE => {
            agent_type_scanner::build_skill_scan_root(&target_agent.agent_type, &absolute_root)
        }
        COMMANDS_SOURCE => {
            agent_type_scanner::build_commands_scan_root(&target_agent.agent_type, &absolute_root)
        }
        _ => None,
    };

    target_root.ok_or_else(|| {
        format!(
            "Agent type {} does not support {} resources",
            target_agent.agent_type, source_kind
        )
    })
}

fn resolve_copy_source_path(source: &LocalSkillCopySourceDto) -> Result<PathBuf, String> {
    match source.source_kind.as_str() {
        SKILLS_SOURCE => {
            let source_path = PathBuf::from(&source.skill_path);
            if !source_path.exists() {
                return Err(format!("Skill source path not found: {}", source.skill_path));
            }
            Ok(source_path)
        }
        COMMANDS_SOURCE => resolve_existing_skill_entry_path(&source.skill_path, &source.entry_file_path),
        _ => Err(format!("Unsupported skill source kind: {}", source.source_kind)),
    }
}

fn build_copy_destination_path(
    source: &LocalSkillCopySourceDto,
    target_root: &Path,
    source_path: &Path,
) -> Result<PathBuf, String> {
    let canonical_destination_path = target_root.join(Path::new(&source.relative_path));
    if source.source_kind == COMMANDS_SOURCE && is_disabled_entry(source_path) {
        let canonical_path = canonical_destination_path.to_string_lossy().to_string();
        return disabled_entry_path(&canonical_destination_path, &canonical_path);
    }

    Ok(canonical_destination_path)
}

fn resolve_existing_destination_path(
    source_kind: &str,
    destination_path: &Path,
) -> Result<Option<PathBuf>, String> {
    if source_kind == SKILLS_SOURCE {
        return Ok(destination_path.exists().then(|| destination_path.to_path_buf()));
    }

    let destination_path_str = destination_path.to_string_lossy().to_string();
    let active_destination_path = enabled_entry_path(destination_path, &destination_path_str)?;
    let active_destination_path_str = active_destination_path.to_string_lossy().to_string();
    let disabled_destination_path =
        disabled_entry_path(&active_destination_path, &active_destination_path_str)?;

    if active_destination_path.exists() {
        return Ok(Some(active_destination_path));
    }
    if disabled_destination_path.exists() {
        return Ok(Some(disabled_destination_path));
    }

    Ok(None)
}

fn build_copy_plans(
    sources: &[LocalSkillCopySourceDto],
    target_agent: &LocalSkillCopyTargetAgentDto,
) -> Result<Vec<PlannedSkillCopy>, String> {
    if sources.is_empty() {
        return Err("No local skills selected for copy.".into());
    }

    let mut seen_source_ids = HashSet::new();
    let mut seen_destination_paths = HashMap::<String, String>::new();
    let mut plans = Vec::new();

    for source in sources {
        if !seen_source_ids.insert(source.id.clone()) {
            return Err(format!("Duplicate skill source in copy request: {}", source.id));
        }
        if source.owner_agent_id == target_agent.agent_id {
            return Err("Cannot copy skills into the same agent.".into());
        }

        let source_path = resolve_copy_source_path(source)?;
        let target_root = resolve_copy_target_root(target_agent, &source.source_kind)?;
        let destination_path = build_copy_destination_path(source, &target_root, &source_path)?;
        let normalized_destination_path = normalize_path(&destination_path);
        if let Some(existing_skill_id) =
            seen_destination_paths.insert(normalized_destination_path.clone(), source.id.clone())
        {
            return Err(format!(
                "Copy request contains duplicate destination path {} for {} and {}",
                normalized_destination_path, existing_skill_id, source.id
            ));
        }

        let existing_path =
            resolve_existing_destination_path(&source.source_kind, &destination_path)?;
        plans.push(PlannedSkillCopy {
            source: source.clone(),
            source_path,
            destination_path,
            existing_path,
        });
    }

    Ok(plans)
}

fn ensure_parent_directory(path: &Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    fs::create_dir_all(parent).map_err(|error| error.to_string())
}

fn remove_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        return fs::remove_dir_all(path).map_err(|error| error.to_string());
    }

    fs::remove_file(path).map_err(|error| error.to_string())
}

fn copy_directory_recursive(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| error.to_string())?;

    let entries = fs::read_dir(source).map_err(|error| error.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let next_destination = destination.join(entry.file_name());
        if path.is_dir() {
            copy_directory_recursive(&path, &next_destination)?;
            continue;
        }

        ensure_parent_directory(&next_destination)?;
        fs::copy(&path, &next_destination).map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn copy_planned_skill(plan: &PlannedSkillCopy) -> Result<(), String> {
    if plan.source_path.is_dir() {
        return copy_directory_recursive(&plan.source_path, &plan.destination_path);
    }

    ensure_parent_directory(&plan.destination_path)?;
    fs::copy(&plan.source_path, &plan.destination_path).map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_local_skills(
    scan_targets: Vec<SkillScanTargetDto>,
) -> Result<Vec<LocalSkillSummaryDto>, String> {
    println!(
        "[skills] list_local_skills command targets: {:?}",
        scan_targets
            .iter()
            .map(|target| format!("{}|{}|{}", target.agent_id, target.agent_type, target.root_path))
            .collect::<Vec<_>>()
    );

    let skills = skill_discovery_service::list_local_skills(scan_targets);

    println!(
        "[skills] list_local_skills command result: {:?}",
        skills
            .iter()
            .map(|skill| format!(
                "{}|owner={}|agentType={}|path={}",
                skill.id, skill.owner_agent_id, skill.agent_type, skill.skill_path
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
            .map(|target| format!("{}|{}|{}", target.agent_id, target.agent_type, target.root_path))
            .collect::<Vec<_>>()
    );

    let detail = skill_discovery_service::get_local_skill_detail(scan_targets, &skill_id)?;

    println!(
        "[skills] get_local_skill_detail command result: {}|owner={}|agentType={}|path={}",
        detail.id, detail.owner_agent_id, detail.agent_type, detail.skill_path
    );

    Ok(detail)
}

#[tauri::command]
pub fn set_local_skill_enabled(
    skill_path: String,
    entry_file_path: String,
    enabled: bool,
) -> Result<(), String> {
    set_local_skill_enabled_at_path(&skill_path, &entry_file_path, enabled)
}

#[tauri::command]
pub fn open_skill_folder(app: tauri::AppHandle, skill_path: String) -> Result<(), String> {
    let path = Path::new(&skill_path);
    if !path.exists() {
        return Err(format!("Skill path not found: {skill_path}"));
    }

    let open_path = if path.is_dir() {
        &skill_path
    } else {
        path.parent()
            .and_then(|parent| parent.to_str())
            .ok_or_else(|| format!("Skill path has no parent directory: {skill_path}"))?
    };

    app.opener()
        .open_path(open_path, None::<&str>)
        .map_err(|error: tauri_plugin_opener::Error| error.to_string())
}

#[tauri::command]
pub fn open_skill_entry_file(
    app: tauri::AppHandle,
    skill_path: String,
    entry_file_path: String,
) -> Result<(), String> {
    let open_path = resolve_existing_skill_entry_path(&skill_path, &entry_file_path)?;
    let open_path = open_path.to_string_lossy().to_string();

    app.opener()
        .open_path(&open_path, None::<&str>)
        .map_err(|error: tauri_plugin_opener::Error| error.to_string())
}

#[tauri::command]
pub fn delete_local_skill(skill_path: String, entry_file_path: String) -> Result<(), String> {
    delete_local_skill_at_path(&skill_path, &entry_file_path)
}

#[tauri::command]
pub fn preview_local_skill_copy(
    sources: Vec<LocalSkillCopySourceDto>,
    target_agent: LocalSkillCopyTargetAgentDto,
) -> Result<PreviewLocalSkillCopyResultDto, String> {
    let plans = build_copy_plans(&sources, &target_agent)?;
    let conflicts = plans
        .iter()
        .filter_map(|plan| {
            let existing_path = plan.existing_path.as_ref()?;
            Some(LocalSkillCopyConflictDto {
                skill_id: plan.source.id.clone(),
                skill_name: plan.source.name.clone(),
                source_kind: plan.source.source_kind.clone(),
                destination_path: normalize_path(&plan.destination_path),
                existing_path: normalize_path(existing_path),
            })
        })
        .collect::<Vec<_>>();

    Ok(PreviewLocalSkillCopyResultDto {
        target_agent_name: target_agent.agent_name,
        total_count: plans.len(),
        conflict_count: conflicts.len(),
        conflicts,
    })
}

#[tauri::command]
pub fn copy_local_skills(
    sources: Vec<LocalSkillCopySourceDto>,
    target_agent: LocalSkillCopyTargetAgentDto,
    resolutions: Vec<LocalSkillConflictResolutionDto>,
) -> Result<CopyLocalSkillsResultDto, String> {
    let plans = build_copy_plans(&sources, &target_agent)?;
    let resolutions_by_skill = resolutions
        .into_iter()
        .map(|resolution| (resolution.skill_id, resolution.action))
        .collect::<HashMap<_, _>>();

    let mut copied_count = 0usize;
    let mut skipped_count = 0usize;

    for plan in plans {
        if let Some(existing_path) = plan.existing_path.as_ref() {
            let Some(action) = resolutions_by_skill.get(&plan.source.id) else {
                return Err(format!(
                    "Conflict resolution missing for skill: {}",
                    plan.source.id
                ));
            };

            match action.as_str() {
                OVERWRITE_ACTION => remove_path(existing_path)?,
                SKIP_ACTION => {
                    skipped_count += 1;
                    continue;
                }
                _ => {
                    return Err(format!(
                        "Unsupported conflict resolution action: {}",
                        action
                    ));
                }
            }
        }

        copy_planned_skill(&plan)?;
        copied_count += 1;
    }

    Ok(CopyLocalSkillsResultDto {
        copied_count,
        skipped_count,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        copy_local_skills, delete_local_skill_at_path, preview_local_skill_copy,
        resolve_existing_skill_entry_path, set_local_skill_enabled_at_path,
    };
    use crate::dto::skills::{
        LocalSkillConflictResolutionDto, LocalSkillCopySourceDto, LocalSkillCopyTargetAgentDto,
    };
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    const SKILL_ENTRY_FILE: &str = "SKILL.md";
    const DISABLED_SKILL_ENTRY_FILE: &str = "SKILL.md.disabled";

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-dock-skills-command-{name}-{unique}"))
    }

    fn write_entry(skill_dir: &PathBuf, name: &str) -> PathBuf {
        fs::create_dir_all(skill_dir).expect("create skill dir");
        let path = skill_dir.join(name);
        fs::write(&path, "# Test skill\n\nBody.").expect("write skill entry");
        path
    }

    fn copy_target_agent(root_path: &PathBuf) -> LocalSkillCopyTargetAgentDto {
        LocalSkillCopyTargetAgentDto {
            agent_id: "agent-target".into(),
            agent_type: "claude".into(),
            agent_name: "Claude Target".into(),
            root_path: root_path.to_string_lossy().to_string(),
        }
    }

    fn skill_copy_source(skill_dir: &PathBuf, name: &str) -> LocalSkillCopySourceDto {
        LocalSkillCopySourceDto {
            id: format!("skill::{name}"),
            name: name.into(),
            owner_agent_id: "agent-source".into(),
            source_kind: "skills".into(),
            relative_path: name.into(),
            skill_path: skill_dir.to_string_lossy().to_string(),
            entry_file_path: skill_dir.join(SKILL_ENTRY_FILE).to_string_lossy().to_string(),
        }
    }

    fn command_copy_source(command_path: &PathBuf, id: &str) -> LocalSkillCopySourceDto {
        let command_file_name = command_path
            .file_name()
            .and_then(|value| value.to_str())
            .expect("command file name");
        let canonical_entry_path = if command_file_name.ends_with(".disabled") {
            PathBuf::from(command_path.to_string_lossy().trim_end_matches(".disabled"))
        } else {
            command_path.clone()
        };

        LocalSkillCopySourceDto {
            id: id.into(),
            name: id.into(),
            owner_agent_id: "agent-source".into(),
            source_kind: "commands".into(),
            relative_path: canonical_entry_path
                .file_name()
                .and_then(|value| value.to_str())
                .expect("command file name")
                .into(),
            skill_path: command_path.to_string_lossy().to_string(),
            entry_file_path: canonical_entry_path.to_string_lossy().to_string(),
        }
    }

    #[test]
    fn disable_skill_renames_enabled_entry() {
        let root = temp_dir("disable");
        let skill_dir = root.join("demo-skill");
        let entry = write_entry(&skill_dir, SKILL_ENTRY_FILE);

        set_local_skill_enabled_at_path(
            &skill_dir.to_string_lossy(),
            &entry.to_string_lossy(),
            false,
        )
        .expect("disable skill");

        assert!(!skill_dir.join(SKILL_ENTRY_FILE).exists());
        assert!(skill_dir.join(DISABLED_SKILL_ENTRY_FILE).exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn enable_skill_renames_disabled_entry() {
        let root = temp_dir("enable");
        let skill_dir = root.join("demo-skill");
        let entry = skill_dir.join(SKILL_ENTRY_FILE);
        write_entry(&skill_dir, DISABLED_SKILL_ENTRY_FILE);

        set_local_skill_enabled_at_path(
            &skill_dir.to_string_lossy(),
            &entry.to_string_lossy(),
            true,
        )
        .expect("enable skill");

        assert!(skill_dir.join(SKILL_ENTRY_FILE).exists());
        assert!(!skill_dir.join(DISABLED_SKILL_ENTRY_FILE).exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn toggle_skill_rejects_conflicting_entries() {
        let root = temp_dir("conflict");
        let skill_dir = root.join("demo-skill");
        let entry = write_entry(&skill_dir, SKILL_ENTRY_FILE);
        write_entry(&skill_dir, DISABLED_SKILL_ENTRY_FILE);

        let error = set_local_skill_enabled_at_path(
            &skill_dir.to_string_lossy(),
            &entry.to_string_lossy(),
            false,
        )
        .expect_err("conflict should fail");

        assert!(error.contains("Target entry file already exists"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn toggle_skill_rejects_missing_entry_file() {
        let root = temp_dir("missing-entry");
        let skill_dir = root.join("demo-skill");
        fs::create_dir_all(&skill_dir).expect("create skill dir");
        let missing_entry = skill_dir.join(SKILL_ENTRY_FILE);

        let error = set_local_skill_enabled_at_path(
            &skill_dir.to_string_lossy(),
            &missing_entry.to_string_lossy(),
            false,
        )
        .expect_err("missing entry should fail");

        assert!(error.contains("Skill entry file not found"));
        assert!(!skill_dir.join(SKILL_ENTRY_FILE).exists());
        assert!(!skill_dir.join(DISABLED_SKILL_ENTRY_FILE).exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn toggle_skill_accepts_disabled_entry_path_when_disabling() {
        let root = temp_dir("stale-entry");
        let skill_dir = root.join("demo-skill");
        write_entry(&skill_dir, SKILL_ENTRY_FILE);
        let stale_entry = skill_dir.join(DISABLED_SKILL_ENTRY_FILE);

        set_local_skill_enabled_at_path(
            &skill_dir.to_string_lossy(),
            &stale_entry.to_string_lossy(),
            false,
        )
        .expect("disable using disabled entry path");

        assert!(!skill_dir.join(SKILL_ENTRY_FILE).exists());
        assert!(skill_dir.join(DISABLED_SKILL_ENTRY_FILE).exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn toggle_skill_rejects_mismatched_skill_file_path() {
        let root = temp_dir("mismatched-file");
        fs::create_dir_all(&root).expect("create temp root");
        let skill_file = root.join("feat.md");
        let entry_file = root.join("other.md");
        fs::write(&skill_file, "# Command\n").expect("write skill markdown");
        fs::write(&entry_file, "# Other\n").expect("write entry markdown");

        let error = set_local_skill_enabled_at_path(
            &skill_file.to_string_lossy(),
            &entry_file.to_string_lossy(),
            false,
        )
        .expect_err("mismatched file should fail");

        assert!(error.contains("Skill file path does not match entry file path"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn disable_skill_with_custom_entry_file() {
        let root = temp_dir("custom-disable");
        let skill_dir = root.join("demo-skill");
        let entry = write_entry(&skill_dir, "custom-skill.md");

        set_local_skill_enabled_at_path(
            &skill_dir.to_string_lossy(),
            &entry.to_string_lossy(),
            false,
        )
        .expect("disable skill with custom entry");

        assert!(!skill_dir.join("custom-skill.md").exists());
        assert!(skill_dir.join("custom-skill.md.disabled").exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn disable_single_file_skill_renames_entry() {
        let root = temp_dir("single-file-disable");
        fs::create_dir_all(&root).expect("create temp root");
        let entry = root.join("feat.md");
        fs::write(&entry, "# Command\n").expect("write markdown");

        set_local_skill_enabled_at_path(&entry.to_string_lossy(), &entry.to_string_lossy(), false)
            .expect("disable single-file skill");

        assert!(!entry.exists());
        assert!(root.join("feat.md.disabled").exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn enable_single_file_skill_uses_canonical_entry_path() {
        let root = temp_dir("single-file-enable");
        fs::create_dir_all(&root).expect("create temp root");
        let canonical_entry = root.join("feat.md");
        let disabled_entry = root.join("feat.md.disabled");
        fs::write(&disabled_entry, "# Command\n").expect("write markdown");

        set_local_skill_enabled_at_path(
            &disabled_entry.to_string_lossy(),
            &canonical_entry.to_string_lossy(),
            true,
        )
        .expect("enable single-file skill");

        assert!(canonical_entry.exists());
        assert!(!disabled_entry.exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn enable_skill_with_custom_entry_file_uses_canonical_path() {
        let root = temp_dir("custom-enable");
        let skill_dir = root.join("demo-skill");
        let entry = skill_dir.join("custom-skill.md");
        write_entry(&skill_dir, "custom-skill.md.disabled");

        set_local_skill_enabled_at_path(
            &skill_dir.to_string_lossy(),
            &entry.to_string_lossy(),
            true,
        )
        .expect("enable skill with custom entry");

        assert!(skill_dir.join("custom-skill.md").exists());
        assert!(!skill_dir.join("custom-skill.md.disabled").exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn resolve_existing_skill_entry_path_returns_enabled_entry() {
        let root = temp_dir("resolve-enabled");
        let skill_dir = root.join("demo-skill");
        let entry = write_entry(&skill_dir, SKILL_ENTRY_FILE);

        let resolved = resolve_existing_skill_entry_path(
            &skill_dir.to_string_lossy(),
            &entry.to_string_lossy(),
        )
        .expect("resolve enabled entry");

        assert_eq!(resolved, entry);

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn resolve_existing_skill_entry_path_returns_disabled_entry() {
        let root = temp_dir("resolve-disabled");
        let skill_dir = root.join("demo-skill");
        let canonical_entry = skill_dir.join(SKILL_ENTRY_FILE);
        let disabled_entry = write_entry(&skill_dir, DISABLED_SKILL_ENTRY_FILE);

        let resolved = resolve_existing_skill_entry_path(
            &skill_dir.to_string_lossy(),
            &canonical_entry.to_string_lossy(),
        )
        .expect("resolve disabled entry");

        assert_eq!(resolved, disabled_entry);

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn resolve_existing_skill_entry_path_rejects_conflicting_entries() {
        let root = temp_dir("resolve-conflict");
        let skill_dir = root.join("demo-skill");
        let entry = write_entry(&skill_dir, SKILL_ENTRY_FILE);
        write_entry(&skill_dir, DISABLED_SKILL_ENTRY_FILE);

        let error = resolve_existing_skill_entry_path(
            &skill_dir.to_string_lossy(),
            &entry.to_string_lossy(),
        )
        .expect_err("conflicting entries should fail");

        assert!(error.contains("Skill entry files conflict"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn preview_local_skill_copy_reports_directory_conflicts() {
        let root = temp_dir("preview-copy-conflict");
        let source_skill_dir = root.join("source-skill");
        write_entry(&source_skill_dir, SKILL_ENTRY_FILE);

        let target_root = root.join("target-agent");
        let target_skill_dir = target_root.join(".claude/skills/source-skill");
        write_entry(&target_skill_dir, SKILL_ENTRY_FILE);

        let preview = preview_local_skill_copy(
            vec![skill_copy_source(&source_skill_dir, "source-skill")],
            copy_target_agent(&target_root.join(".claude")),
        )
        .expect("preview copy");

        assert_eq!(preview.total_count, 1);
        assert_eq!(preview.conflict_count, 1);
        assert_eq!(preview.conflicts[0].skill_id, "skill::source-skill");

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn copy_local_skills_skips_conflicting_directory_when_requested() {
        let root = temp_dir("copy-skip-conflict");
        let source_skill_dir = root.join("source-skill");
        write_entry(&source_skill_dir, SKILL_ENTRY_FILE);

        let target_root = root.join("target-agent");
        let target_skill_dir = target_root.join(".claude/skills/source-skill");
        write_entry(&target_skill_dir, SKILL_ENTRY_FILE);
        fs::write(target_skill_dir.join("data.txt"), "existing").expect("write existing marker");

        let result = copy_local_skills(
            vec![skill_copy_source(&source_skill_dir, "source-skill")],
            copy_target_agent(&target_root.join(".claude")),
            vec![LocalSkillConflictResolutionDto {
                skill_id: "skill::source-skill".into(),
                action: "skip".into(),
            }],
        )
        .expect("copy local skills");

        assert_eq!(result.copied_count, 0);
        assert_eq!(result.skipped_count, 1);
        assert!(target_skill_dir.join("data.txt").exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn copy_local_skills_overwrites_conflicting_command_file() {
        let root = temp_dir("copy-overwrite-command");
        let source_root = root.join("source-commands");
        fs::create_dir_all(&source_root).expect("create source command dir");
        let source_command = source_root.join("feat.md");
        fs::write(&source_command, "# New command\n").expect("write source command");

        let target_root = root.join("target-agent");
        let target_command_root = target_root.join(".claude/commands");
        fs::create_dir_all(&target_command_root).expect("create target command dir");
        let target_command = target_command_root.join("feat.md");
        fs::write(&target_command, "# Old command\n").expect("write target command");

        let result = copy_local_skills(
            vec![command_copy_source(&source_command, "command::feat")],
            copy_target_agent(&target_root.join(".claude")),
            vec![LocalSkillConflictResolutionDto {
                skill_id: "command::feat".into(),
                action: "overwrite".into(),
            }],
        )
        .expect("copy local commands");

        assert_eq!(result.copied_count, 1);
        assert_eq!(result.skipped_count, 0);
        assert_eq!(
            fs::read_to_string(&target_command).expect("read copied command"),
            "# New command\n"
        );

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn copy_local_skills_requires_conflict_resolution() {
        let root = temp_dir("copy-missing-resolution");
        let source_skill_dir = root.join("source-skill");
        write_entry(&source_skill_dir, SKILL_ENTRY_FILE);

        let target_root = root.join("target-agent");
        let target_skill_dir = target_root.join(".claude/skills/source-skill");
        write_entry(&target_skill_dir, SKILL_ENTRY_FILE);

        let error = copy_local_skills(
            vec![skill_copy_source(&source_skill_dir, "source-skill")],
            copy_target_agent(&target_root.join(".claude")),
            vec![],
        )
        .expect_err("missing conflict resolution should fail");

        assert!(error.contains("Conflict resolution missing"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn delete_local_skill_removes_directory_skill() {
        let root = temp_dir("delete-directory");
        let skill_dir = root.join("demo-skill");
        let entry = write_entry(&skill_dir, SKILL_ENTRY_FILE);

        delete_local_skill_at_path(&skill_dir.to_string_lossy(), &entry.to_string_lossy())
            .expect("delete skill directory");

        assert!(!skill_dir.exists());
        fs::remove_dir_all(root).expect("cleanup temp dir");
    }
}
