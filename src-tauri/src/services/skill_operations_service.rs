use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::dto::skills::{
    CopyLocalSkillsResultDto, LocalSkillConflictResolutionDto, LocalSkillCopyConflictDto,
    LocalSkillCopySourceDto, LocalSkillCopyTargetAgentDto, PreviewLocalSkillCopyResultDto,
};
use crate::infrastructure::skill_fs;
use crate::infrastructure::utils::path::{normalize_path, resolve_agent_root};
use crate::repositories::marketplace_install_repository::MarketplaceInstallRepository;
use crate::scanners::agent_type_scanner;
use crate::services::ServiceError;

const DISABLED_SUFFIX: &str = ".disabled";

/// Domain model for a skill with business logic
#[derive(Debug, Clone)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub entry_file: PathBuf,
    pub enabled: bool,
    pub source_kind: String,
    pub owner_agent_id: String,
}

impl Skill {
    /// Toggle enabled state
    pub fn toggle_enabled(&mut self) -> Result<(), ServiceError> {
        let (active_path, disabled_path) = self.resolve_entry_paths()?;

        if self.enabled {
            // Disabling: check if disabled path already exists
            if disabled_path.exists() && active_path.exists() {
                return Err(ServiceError::ConflictingEntryFiles(
                    format!("{} and {}", active_path.display(), disabled_path.display())
                ));
            }
            self.enabled = false;
        } else {
            // Enabling: check if active path already exists
            if active_path.exists() && disabled_path.exists() {
                return Err(ServiceError::ConflictingEntryFiles(
                    format!("{} and {}", active_path.display(), disabled_path.display())
                ));
            }
            self.enabled = true;
        }

        Ok(())
    }

    /// Validate if this skill can be copied to a target agent
    pub fn validate_copy_destination(&self, target_agent_id: &str, target_supports_source: bool) -> Result<(), ServiceError> {
        // Cannot copy to same agent
        if self.owner_agent_id == target_agent_id {
            return Err(ServiceError::CannotCopyToSameAgent);
        }

        // Target must support this skill's source kind
        if !target_supports_source {
            return Err(ServiceError::UnsupportedSkillSource(self.source_kind.clone()));
        }

        Ok(())
    }

    /// Resolve entry file paths (active and disabled)
    pub fn resolve_entry_paths(&self) -> Result<(PathBuf, PathBuf), ServiceError> {
        Self::resolve_entry_paths_for(&self.entry_file)
    }

    /// Resolve entry file paths for a given entry file path (static version)
    pub fn resolve_entry_paths_for(entry_file: &Path) -> Result<(PathBuf, PathBuf), ServiceError> {
        let entry_name = entry_file
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| ServiceError::InvalidSkillPath(
                format!("Invalid entry file: {}", entry_file.display())
            ))?;

        let active_path = if entry_name.ends_with(DISABLED_SUFFIX) {
            entry_file.with_file_name(entry_name.trim_end_matches(DISABLED_SUFFIX))
        } else {
            entry_file.to_path_buf()
        };

        let disabled_path = if entry_name.ends_with(DISABLED_SUFFIX) {
            entry_file.to_path_buf()
        } else {
            entry_file.with_file_name(format!("{}{}", entry_name, DISABLED_SUFFIX))
        };

        Ok((active_path, disabled_path))
    }

    /// Check if an entry path has the disabled suffix
    pub fn is_disabled_entry(entry_path: &Path) -> bool {
        entry_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.ends_with(DISABLED_SUFFIX))
            .unwrap_or(false)
    }

    /// Get the entry file name
    pub fn entry_file_name(entry_path: &Path) -> Result<String, ServiceError> {
        entry_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
            .ok_or_else(|| ServiceError::InvalidSkillPath(
                format!("Invalid skill entry file path: {}", entry_path.display())
            ))
    }

    /// Validate that skill_path is consistent with the canonical entry path
    pub fn validate_path_consistency(skill_path: &Path, canonical_entry_path: &Path) -> Result<(), ServiceError> {
        if !skill_path.exists() {
            return Err(ServiceError::InvalidSkillPath(
                format!("Skill path not found: {}", skill_path.display())
            ));
        }

        if skill_path.is_dir() {
            let entry_parent = canonical_entry_path.parent().ok_or_else(|| {
                ServiceError::InvalidSkillPath(format!(
                    "Skill entry file has no parent directory: {}",
                    canonical_entry_path.display()
                ))
            })?;
            if entry_parent != skill_path {
                return Err(ServiceError::InvalidSkillPath(format!(
                    "Skill entry file does not belong to skill directory: {}",
                    canonical_entry_path.display()
                )));
            }
            return Ok(());
        }

        if skill_path.is_file() {
            let disabled_entry_candidate = Self::resolve_entry_paths_for(canonical_entry_path)?.1;
            if skill_path != canonical_entry_path && skill_path != disabled_entry_candidate {
                return Err(ServiceError::InvalidSkillPath(format!(
                    "Skill file path does not match entry file path: {}",
                    skill_path.display()
                )));
            }
            return Ok(());
        }

        Err(ServiceError::InvalidSkillPath(format!(
            "Skill path is neither a file nor a directory: {}",
            skill_path.display()
        )))
    }

    /// Resolve which entry file actually exists (active or disabled)
    pub fn resolve_existing_entry_path(skill_path: &Path, entry_file_path: &Path) -> Result<PathBuf, ServiceError> {
        let (active_entry_path, disabled_entry_path) = Self::resolve_entry_paths_for(entry_file_path)?;
        Self::validate_path_consistency(skill_path, &active_entry_path)?;

        match (active_entry_path.is_file(), disabled_entry_path.is_file()) {
            (true, false) => Ok(active_entry_path),
            (false, true) => Ok(disabled_entry_path),
            (false, false) => Err(ServiceError::InvalidSkillPath(format!(
                "Skill entry file not found: {}",
                entry_file_path.display()
            ))),
            (true, true) => Err(ServiceError::ConflictingEntryFiles(format!(
                "{} and {}",
                active_entry_path.display(),
                disabled_entry_path.display()
            ))),
        }
    }
}

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

#[derive(Clone)]
pub struct SkillOperationsService {
    pub install_repo: Arc<dyn MarketplaceInstallRepository>,
}

impl SkillOperationsService {
    pub fn new(install_repo: Arc<dyn MarketplaceInstallRepository>) -> Self {
        Self { install_repo }
    }

    /// Set skill enabled/disabled state
    pub fn set_skill_enabled(
        &self,
        skill_path: &str,
        entry_file_path: &str,
        enabled: bool,
    ) -> Result<(), ServiceError> {
        let skill_path = Path::new(skill_path);
        let entry_file_path = Path::new(entry_file_path);

        skill_fs::rename_entry_file(skill_path, entry_file_path, enabled)
    }

    /// Delete skill and remove marketplace install record
    pub fn delete_skill(&self, skill_path: &str, entry_file_path: &str) -> Result<(), ServiceError> {
        let skill_path_buf = Path::new(skill_path);
        let entry_file_path_buf = Path::new(entry_file_path);

        skill_fs::delete_skill(skill_path_buf, entry_file_path_buf)?;
        self.install_repo.delete(skill_path, entry_file_path)?;
        Ok(())
    }

    /// Preview skill copy operation and detect conflicts
    pub fn preview_skill_copy(
        &self,
        sources: Vec<LocalSkillCopySourceDto>,
        target_agent: LocalSkillCopyTargetAgentDto,
    ) -> Result<PreviewLocalSkillCopyResultDto, ServiceError> {
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

    /// Execute skill copy with conflict resolution
    pub fn execute_skill_copy(
        &self,
        sources: Vec<LocalSkillCopySourceDto>,
        target_agent: LocalSkillCopyTargetAgentDto,
        resolutions: Vec<LocalSkillConflictResolutionDto>,
    ) -> Result<CopyLocalSkillsResultDto, ServiceError> {
        let plans = build_copy_plans(&sources, &target_agent)?;
        let resolutions_by_skill = resolutions
            .into_iter()
            .map(|resolution| (resolution.skill_id, resolution.action))
            .collect::<HashMap<_, _>>();

        let mut copied_count = 0usize;
        let mut skipped_count = 0usize;

        for plan in plans {
            if let Some(existing_path) = plan.existing_path.as_ref() {
                let action = resolutions_by_skill
                    .get(&plan.source.id)
                    .ok_or_else(|| ServiceError::BusinessRuleViolation(format!("Conflict resolution missing for skill: {}", plan.source.id)))?;

                if action == OVERWRITE_ACTION {
                    skill_fs::remove_existing_skill(existing_path)?;
                } else if action == SKIP_ACTION {
                    skipped_count += 1;
                    continue;
                } else {
                    return Err(ServiceError::BusinessRuleViolation(format!("Unsupported conflict resolution action: {}", action)));
                }
            }

            skill_fs::copy_skill(&plan.source_path, &plan.destination_path)?;
            copied_count += 1;
        }

        Ok(CopyLocalSkillsResultDto {
            copied_count,
            skipped_count,
        })
    }

    /// Open skill folder using system explorer
    pub fn open_skill_folder(&self, app: tauri::AppHandle, skill_path: &str) -> Result<(), ServiceError> {
        let path = Path::new(skill_path);
        if !path.exists() {
            return Err(ServiceError::BusinessRuleViolation(format!("Skill path not found: {skill_path}")));
        }

        let open_path = if path.is_dir() {
            skill_path.to_string()
        } else {
            path.parent()
                .and_then(|parent| parent.to_str())
                .map(str::to_string)
                .ok_or_else(|| ServiceError::BusinessRuleViolation(format!("Skill path has no parent directory: {skill_path}")))?
        };

        app.opener()
            .open_path(open_path, None::<&str>)
            .map_err(|error: tauri_plugin_opener::Error| ServiceError::Internal(error.to_string()))
    }

    /// Open skill entry file using system default application
    pub fn open_skill_entry_file(
        &self,
        app: tauri::AppHandle,
        skill_path: &str,
        entry_file_path: &str,
    ) -> Result<(), ServiceError> {
        let skill_path = Path::new(skill_path);
        let entry_file_path = Path::new(entry_file_path);
        let open_path = Skill::resolve_existing_entry_path(skill_path, entry_file_path)?;
        let open_path = open_path.to_string_lossy().to_string();

        app.opener()
            .open_path(&open_path, None::<&str>)
            .map_err(|error: tauri_plugin_opener::Error| ServiceError::Internal(error.to_string()))
    }
}

use tauri_plugin_opener::OpenerExt;

fn resolve_copy_target_root(
    target_agent: &LocalSkillCopyTargetAgentDto,
    source_kind: &str,
) -> Result<PathBuf, ServiceError> {
    let absolute_root = resolve_agent_root(&target_agent.root_path);
    let target_root = if source_kind == SKILLS_SOURCE {
        agent_type_scanner::build_skill_scan_root(&target_agent.agent_type, &absolute_root)
    } else if source_kind == COMMANDS_SOURCE {
        agent_type_scanner::build_commands_scan_root(&target_agent.agent_type, &absolute_root)
    } else {
        None
    };

    target_root.ok_or_else(|| {
        ServiceError::BusinessRuleViolation(format!(
            "Agent type {} does not support {} resources",
            target_agent.agent_type, source_kind
        ))
    })
}

fn resolve_copy_source_path(source: &LocalSkillCopySourceDto) -> Result<PathBuf, ServiceError> {
    match source.source_kind.as_str() {
        SKILLS_SOURCE => {
            let source_path = PathBuf::from(&source.skill_path);
            if !source_path.exists() {
                return Err(ServiceError::BusinessRuleViolation(format!(
                    "Skill source path not found: {}",
                    source.skill_path
                )));
            }
            Ok(source_path)
        }
        COMMANDS_SOURCE => {
            let skill_path = Path::new(&source.skill_path);
            let entry_file_path = Path::new(&source.entry_file_path);
            Skill::resolve_existing_entry_path(skill_path, entry_file_path).map_err(Into::into)
        }
        _ => Err(ServiceError::BusinessRuleViolation(format!(
            "Unsupported skill source kind: {}",
            source.source_kind
        ))),
    }
}

fn build_copy_destination_path(
    source: &LocalSkillCopySourceDto,
    target_root: &Path,
    source_path: &Path,
) -> Result<PathBuf, ServiceError> {
    let canonical_destination_path = target_root.join(Path::new(&source.relative_path));
    if source.source_kind == COMMANDS_SOURCE && Skill::is_disabled_entry(source_path) {
        let (_, disabled_path) = Skill::resolve_entry_paths_for(&canonical_destination_path)?;
        return Ok(disabled_path);
    }

    Ok(canonical_destination_path)
}

fn resolve_existing_destination_path(
    source_kind: &str,
    destination_path: &Path,
) -> Result<Option<PathBuf>, ServiceError> {
    if source_kind == SKILLS_SOURCE {
        return Ok(destination_path.exists().then(|| destination_path.to_path_buf()));
    }

    let (active_path, disabled_path) = Skill::resolve_entry_paths_for(destination_path)?;

    if active_path.exists() {
        return Ok(Some(active_path));
    }

    Ok(disabled_path.exists().then_some(disabled_path))
}

fn build_copy_plans(
    sources: &[LocalSkillCopySourceDto],
    target_agent: &LocalSkillCopyTargetAgentDto,
) -> Result<Vec<PlannedSkillCopy>, ServiceError> {
    if sources.is_empty() {
        return Err(ServiceError::BusinessRuleViolation("No local skills selected for copy.".into()));
    }

    let mut seen_source_ids = HashSet::new();
    let mut seen_destination_paths = HashMap::<String, String>::new();
    let mut plans = Vec::new();

    for source in sources {
        if !seen_source_ids.insert(source.id.clone()) {
            return Err(ServiceError::BusinessRuleViolation(format!(
                "Duplicate skill source in copy request: {}",
                source.id
            )));
        }
        if source.owner_agent_id == target_agent.agent_id {
            return Err(ServiceError::BusinessRuleViolation("Cannot copy skills into the same agent.".into()));
        }

        let source_path = resolve_copy_source_path(source)?;
        let target_root = resolve_copy_target_root(target_agent, &source.source_kind)?;
        let destination_path = build_copy_destination_path(source, &target_root, &source_path)?;
        let normalized_destination_path = normalize_path(&destination_path);
        if let Some(existing_skill_id) =
            seen_destination_paths.insert(normalized_destination_path.clone(), source.id.clone())
        {
            return Err(ServiceError::BusinessRuleViolation(format!(
                "Copy request contains duplicate destination path {} for {} and {}",
                normalized_destination_path, existing_skill_id, source.id
            )));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use crate::repositories::marketplace_install_repository::MarketplaceInstallRecord;
    use crate::repositories::errors::RepositoryError;

    struct MockInstallRepo;
    impl MarketplaceInstallRepository for MockInstallRepo {
        fn find_all(&self) -> Vec<MarketplaceInstallRecord> { vec![] }
        fn save_all(&self, _records: &[MarketplaceInstallRecord]) -> Result<(), RepositoryError> { Ok(()) }
        fn upsert(&self, _record: MarketplaceInstallRecord) -> Result<(), RepositoryError> { Ok(()) }
        fn delete(&self, _skill_path: &str, _entry_file_path: &str) -> Result<(), RepositoryError> { Ok(()) }
    }

    #[test]
    fn test_set_skill_enabled_logic() {
        let dir = tempdir().expect("create temp dir");
        let skill_path = dir.path().join("test-skill");
        fs::create_dir(&skill_path).expect("create skill dir");

        let entry_file = skill_path.join("index.js");
        fs::write(&entry_file, "console.log('hello')").expect("write entry file");

        let repo = Arc::new(MockInstallRepo);
        let service = SkillOperationsService::new(repo);

        // 1. Initial state: enabled (index.js exists)
        assert!(entry_file.exists());

        // 2. Disable: rename index.js to index.js.disabled
        let result = service.set_skill_enabled(
            skill_path.to_str().unwrap(),
            entry_file.to_str().unwrap(),
            false
        );
        assert!(result.is_ok(), "set_skill_enabled(false) failed: {:?}", result.err());
        assert!(!entry_file.exists());
        assert!(skill_path.join("index.js.disabled").exists());

        // 3. Enable: rename back to index.js
        let result = service.set_skill_enabled(
            skill_path.to_str().unwrap(),
            entry_file.to_str().unwrap(),
            true
        );
        assert!(result.is_ok(), "set_skill_enabled(true) failed: {:?}", result.err());
        assert!(entry_file.exists());
        assert!(!skill_path.join("index.js.disabled").exists());
    }

    #[test]
    fn test_delete_skill_logic() {
        let dir = tempdir().expect("create temp dir");
        let skill_path = dir.path().join("test-skill-to-delete");
        fs::create_dir(&skill_path).expect("create skill dir");

        let entry_file = skill_path.join("index.js");
        fs::write(&entry_file, "content").expect("write entry file");

        let repo = Arc::new(MockInstallRepo);
        let service = SkillOperationsService::new(repo);

        let result = service.delete_skill(
            skill_path.to_str().unwrap(),
            entry_file.to_str().unwrap()
        );
        assert!(result.is_ok(), "delete_skill failed: {:?}", result.err());

        assert!(!skill_path.exists());
    }

    const SKILL_ENTRY_FILE: &str = "SKILL.md";

    fn temp_dir(name: &str) -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-dock-skill-ops-test-{name}-{unique}"))
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
            entry_file_path: skill_dir
                .join(SKILL_ENTRY_FILE)
                .to_string_lossy()
                .to_string(),
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
    fn preview_local_skill_copy_reports_directory_conflicts() {
        let root = temp_dir("preview-copy-conflict");
        let source_skill_dir = root.join("source-skill");
        write_entry(&source_skill_dir, SKILL_ENTRY_FILE);

        let target_root = root.join("target-agent");
        let target_skill_dir = target_root.join(".claude/skills/source-skill");
        write_entry(&target_skill_dir, SKILL_ENTRY_FILE);

        let repo = Arc::new(MockInstallRepo);
        let service = SkillOperationsService::new(repo);

        let preview = service.preview_skill_copy(
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

        let repo = Arc::new(MockInstallRepo);
        let service = SkillOperationsService::new(repo);

        let result = service.execute_skill_copy(
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

        let repo = Arc::new(MockInstallRepo);
        let service = SkillOperationsService::new(repo);

        let result = service.execute_skill_copy(
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

        let repo = Arc::new(MockInstallRepo);
        let service = SkillOperationsService::new(repo);

        let error = service.execute_skill_copy(
            vec![skill_copy_source(&source_skill_dir, "source-skill")],
            copy_target_agent(&target_root.join(".claude")),
            vec![],
        )
        .expect_err("missing conflict resolution should fail");

        assert!(error.to_string().contains("Conflict resolution missing"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }
}
