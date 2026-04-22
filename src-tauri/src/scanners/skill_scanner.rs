use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::dto::skills::{LocalSkillDetailDto, LocalSkillSummaryDto, SkillScanTargetDto};
use crate::infrastructure::utils::path::{normalize_path, resolve_agent_root};
use crate::scanners::skill_markdown::{
    resolved_description, split_frontmatter, summary_from_markdown,
};

const COMMANDS_SOURCE: &str = "commands";
const SKILL_ENTRY_FILE: &str = "SKILL.md";
const DISABLED_SKILL_ENTRY_FILE: &str = "SKILL.md.disabled";
const DISABLED_SUFFIX: &str = ".disabled";

#[derive(Clone)]
pub struct ParsedSkill {
    pub summary: LocalSkillSummaryDto,
    pub detail: LocalSkillDetailDto,
}

fn entry_file_name(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
}

fn is_disabled_entry(path: &Path) -> bool {
    entry_file_name(path)
        .map(|name| name.ends_with(DISABLED_SUFFIX))
        .unwrap_or(false)
}

fn enabled_entry_path(path: &Path) -> PathBuf {
    let Some(entry_name) = entry_file_name(path) else {
        return path.to_path_buf();
    };
    let enabled_name = entry_name
        .strip_suffix(DISABLED_SUFFIX)
        .unwrap_or(&entry_name);
    path.with_file_name(enabled_name)
}

fn skill_id(agent_id: &str, skill_name: &str, source: &str) -> String {
    format!("{agent_id}::{source}::{skill_name}")
}

fn tags_from_frontmatter(frontmatter: Option<&Value>) -> Vec<String> {
    frontmatter
        .and_then(|value| value.get("tags"))
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn name_from_frontmatter(frontmatter: Option<&Value>, fallback: &str) -> String {
    frontmatter
        .and_then(|value| value.get("name").or_else(|| value.get("title")))
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| fallback.to_string())
}

fn updated_at(path: &Path) -> String {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .map(DateTime::<Utc>::from)
        .map(|datetime| datetime.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string())
}

fn command_name(scan_root: &Path, entry_file: &Path) -> Option<String> {
    let canonical_entry_file = enabled_entry_path(entry_file);
    let relative_path = canonical_entry_file.strip_prefix(scan_root).ok()?;
    let without_extension = relative_path.with_extension("");
    Some(normalize_path(&without_extension))
}

fn display_name_from_path(path: &str) -> String {
    path.replace('/', ":")
}

fn relative_skill_path(
    scan_target: &SkillScanTargetDto,
    scan_root: &Path,
    skill_path: &Path,
) -> Option<String> {
    if scan_target.source == COMMANDS_SOURCE {
        let canonical_entry_file = enabled_entry_path(skill_path);
        return canonical_entry_file
            .strip_prefix(scan_root)
            .ok()
            .map(normalize_path);
    }

    skill_path.strip_prefix(scan_root).ok().map(normalize_path)
}

fn parse_skill(
    scan_target: &SkillScanTargetDto,
    scan_root: &Path,
    skill_path: PathBuf,
    entry_file: PathBuf,
    enabled: bool,
) -> Option<ParsedSkill> {
    if !entry_file.exists() {
        return None;
    }

    let canonical_entry_file = enabled_entry_path(&entry_file);
    let skill_name = if scan_target.source == COMMANDS_SOURCE {
        command_name(scan_root, &entry_file)?
    } else {
        skill_path.file_name()?.to_string_lossy().to_string()
    };
    let id = skill_id(&scan_target.agent_id, &skill_name, &scan_target.source);
    let relative_path = relative_skill_path(scan_target, scan_root, &skill_path)?;
    let updated_at = updated_at(&entry_file);
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if scan_target.source != COMMANDS_SOURCE
        && enabled
        && skill_path.join(DISABLED_SKILL_ENTRY_FILE).exists()
    {
        warnings.push("Conflicting skill entry files found; using enabled SKILL.md.".into());
    }

    let contents = match fs::read_to_string(&entry_file) {
        Ok(contents) => contents,
        Err(error) => {
            errors.push(error.to_string());
            String::new()
        }
    };

    let (frontmatter_raw, markdown) = split_frontmatter(&contents);
    let frontmatter = frontmatter_raw
        .as_ref()
        .and_then(|raw| serde_yaml::from_str::<Value>(raw).ok());
    if frontmatter_raw.is_some() && frontmatter.is_none() {
        warnings.push("Failed to parse frontmatter.".into());
    }

    let summary_text = summary_from_markdown(&markdown, "Local skill");
    let fallback_name = if scan_target.source == COMMANDS_SOURCE {
        display_name_from_path(&skill_name)
    } else {
        skill_name.clone()
    };
    let name = name_from_frontmatter(frontmatter.as_ref(), &fallback_name);
    let description = resolved_description(
        frontmatter.as_ref(),
        frontmatter_raw.as_deref(),
        &summary_text,
    );
    let tags = tags_from_frontmatter(frontmatter.as_ref());
    let status = if errors.is_empty() {
        if warnings.is_empty() {
            "ready"
        } else {
            "warning"
        }
    } else {
        "error"
    }
    .to_string();

    let summary = LocalSkillSummaryDto {
        id: id.clone(),
        kind: "skill".into(),
        name: name.clone(),
        summary: summary_text.clone(),
        enabled,
        tags: tags.clone(),
        usage_count: 0,
        updated_at: updated_at.clone(),
        owner_agent_id: scan_target.agent_id.clone(),
        source_label: format!("{} local", scan_target.display_name),
        source_kind: scan_target.source.clone(),
        relative_path: relative_path.clone(),
        description: description.clone(),
        status: status.clone(),
        skill_path: normalize_path(&skill_path),
        entry_file_path: normalize_path(&canonical_entry_file),
        agent_type: scan_target.agent_type.clone(),
        agent_name: scan_target.display_name.clone(),
        warnings: warnings.clone(),
        errors: errors.clone(),
        marketplace_source: None,
        marketplace_skill_id: None,
    };

    let detail = LocalSkillDetailDto {
        id,
        kind: "skill".into(),
        name,
        summary: summary_text,
        description,
        enabled,
        tags,
        usage_count: 0,
        updated_at,
        markdown,
        owner_agent_id: scan_target.agent_id.clone(),
        source_label: format!("{} local", scan_target.display_name),
        source_kind: scan_target.source.clone(),
        relative_path,
        status,
        skill_path: normalize_path(&skill_path),
        entry_file_path: normalize_path(&canonical_entry_file),
        agent_type: scan_target.agent_type.clone(),
        agent_name: scan_target.display_name.clone(),
        warnings,
        errors,
        marketplace_source: None,
        marketplace_skill_id: None,
    };

    Some(ParsedSkill { summary, detail })
}

fn collect_command_markdown_files(scan_root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(scan_root) else {
        return Vec::new();
    };

    let mut files = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_command_markdown_files(&path));
            continue;
        }

        if !path.is_file() {
            continue;
        }

        let Some(file_name) = entry_file_name(&path) else {
            continue;
        };
        if path.extension().and_then(|extension| extension.to_str()) == Some("md")
            || file_name.ends_with(".md.disabled")
        {
            files.push(path);
        }
    }

    files.sort();
    files
}

fn collect_command_entries(scan_root: &Path) -> Vec<(PathBuf, bool)> {
    let mut command_entries = BTreeMap::<PathBuf, (PathBuf, bool)>::new();

    for path in collect_command_markdown_files(scan_root) {
        let canonical_entry_path = enabled_entry_path(&path);
        let enabled = !is_disabled_entry(&path);
        command_entries
            .entry(canonical_entry_path)
            .and_modify(|entry| {
                if enabled {
                    *entry = (path.clone(), true);
                }
            })
            .or_insert((path, enabled));
    }

    command_entries.into_values().collect()
}

pub fn scan_skills(scan_targets: Vec<SkillScanTargetDto>) -> Vec<ParsedSkill> {
    let mut parsed_skills = Vec::new();

    for scan_target in scan_targets {
        let scan_root = resolve_agent_root(&scan_target.root_path);
        if !scan_root.exists() || !scan_root.is_dir() {
            continue;
        }

        if scan_target.source == COMMANDS_SOURCE {
            for (entry_file, enabled) in collect_command_entries(&scan_root) {
                if let Some(skill) = parse_skill(
                    &scan_target,
                    &scan_root,
                    entry_file.clone(),
                    entry_file,
                    enabled,
                ) {
                    parsed_skills.push(skill);
                }
            }
            continue;
        }

        let Ok(entries) = fs::read_dir(&scan_root) else {
            continue;
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let enabled_entry = path.join(SKILL_ENTRY_FILE);
            let disabled_entry = path.join(DISABLED_SKILL_ENTRY_FILE);
            let preferred_entry = if enabled_entry.exists() {
                Some((enabled_entry, true))
            } else if disabled_entry.exists() {
                Some((disabled_entry, false))
            } else {
                None
            };

            if let Some((entry_file, enabled)) = preferred_entry {
                if let Some(skill) =
                    parse_skill(&scan_target, &scan_root, path.clone(), entry_file, enabled)
                {
                    parsed_skills.push(skill);
                }
            }
        }
    }

    parsed_skills.sort_by(|left, right| right.summary.updated_at.cmp(&left.summary.updated_at));
    parsed_skills
}

#[cfg(test)]
mod tests {
    use super::{
        scan_skills, COMMANDS_SOURCE, DISABLED_SKILL_ENTRY_FILE,
        SKILL_ENTRY_FILE,
    };
    use crate::dto::skills::SkillScanTargetDto;
    use crate::infrastructure::utils::path::{normalize_path, resolve_agent_root};
    use std::{
        env, fs,
        path::{Path, PathBuf},
        sync::{Mutex, OnceLock},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-dock-{name}-{unique}"))
    }

    fn write_skill_entry(skill_dir: &PathBuf, entry_name: &str, contents: &str) {
        fs::create_dir_all(skill_dir).expect("create skill dir");
        fs::write(skill_dir.join(entry_name), contents).expect("write skill markdown");
    }

    fn user_home_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct UserHomeEnvGuard {
        previous_userprofile: Option<std::ffi::OsString>,
        previous_home: Option<std::ffi::OsString>,
        _lock: std::sync::MutexGuard<'static, ()>,
    }

    impl UserHomeEnvGuard {
        fn set(home: &Path) -> Self {
            let lock = user_home_lock()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let previous_userprofile = env::var_os("USERPROFILE");
            let previous_home = env::var_os("HOME");
            env::set_var("USERPROFILE", home);
            env::set_var("HOME", home);

            Self {
                previous_userprofile,
                previous_home,
                _lock: lock,
            }
        }
    }

    impl Drop for UserHomeEnvGuard {
        fn drop(&mut self) {
            if let Some(value) = self.previous_userprofile.as_ref() {
                env::set_var("USERPROFILE", value);
            } else {
                env::remove_var("USERPROFILE");
            }

            if let Some(value) = self.previous_home.as_ref() {
                env::set_var("HOME", value);
            } else {
                env::remove_var("HOME");
            }
        }
    }

    fn scan_test_skill(
        skill_name: &str,
        contents: &str,
    ) -> crate::scanners::skill_scanner::ParsedSkill {
        scan_test_skill_with_entry(skill_name, SKILL_ENTRY_FILE, contents)
    }

    fn scan_test_skill_with_entry(
        skill_name: &str,
        entry_name: &str,
        contents: &str,
    ) -> crate::scanners::skill_scanner::ParsedSkill {
        let root = temp_dir(skill_name);
        let skill_dir = root.join(skill_name);
        write_skill_entry(&skill_dir, entry_name, contents);

        let mut skills = scan_skills(vec![SkillScanTargetDto {
            agent_id: "agent-claude".into(),
            agent_type: "claude".into(),
            root_path: root.to_string_lossy().to_string(),
            display_name: "Claude Main".into(),
            source: "skills".into(),
        }]);

        let skill = skills.pop().expect("skill should be parsed");
        fs::remove_dir_all(&root).expect("cleanup temp dir");
        skill
    }

    #[test]
    fn resolve_scan_root_uses_user_home_for_relative_paths() {
        let test_home = temp_dir("resolve-relative-home");
        let _guard = UserHomeEnvGuard::set(&test_home);

        assert_eq!(
            resolve_agent_root(".claude/skills"),
            test_home.join(".claude/skills")
        );

        fs::remove_dir_all(&test_home).ok();
    }

    #[test]
    fn resolve_scan_root_expands_tilde_prefixed_paths() {
        let test_home = temp_dir("resolve-tilde-home");
        let _guard = UserHomeEnvGuard::set(&test_home);

        assert_eq!(
            resolve_agent_root("~/.claude/skills"),
            test_home.join(".claude/skills")
        );

        fs::remove_dir_all(&test_home).ok();
    }

    #[test]
    fn scan_skills_reads_windows_claude_home_skill_directory_from_tilde_root() {
        let home = temp_dir("claude-home-tilde");
        let skills_root = home.join(".claude").join("skills");
        let first_skill = skills_root.join("find-skills");
        let second_skill = skills_root.join("peon-ping-log");

        fs::create_dir_all(&first_skill).expect("create first skill dir");
        fs::create_dir_all(&second_skill).expect("create second skill dir");
        fs::write(
            first_skill.join("SKILL.md"),
            "# Find skills\nLocate local skills.",
        )
        .expect("write first skill markdown");
        fs::write(
            second_skill.join("SKILL.md"),
            "# Peon ping log\nInspect ping logs.",
        )
        .expect("write second skill markdown");

        let _user_home = UserHomeEnvGuard::set(&home);

        let skills = scan_skills(vec![SkillScanTargetDto {
            agent_id: "agent-claude".into(),
            agent_type: "claude".into(),
            root_path: "~/.claude/skills".into(),
            display_name: "Claude Main".into(),
            source: "skills".into(),
        }]);

        let mut ids = skills
            .iter()
            .map(|skill| skill.summary.id.clone())
            .collect::<Vec<_>>();
        ids.sort();

        assert_eq!(
            ids,
            vec![
                "agent-claude::skills::find-skills".to_string(),
                "agent-claude::skills::peon-ping-log".to_string(),
            ]
        );

        fs::remove_dir_all(&home).expect("cleanup temp dir");
    }

    #[test]
    fn scan_skills_reads_disabled_skill_entry_when_enabled_entry_is_missing() {
        let skill = scan_test_skill_with_entry(
            "disabled-entry-skill",
            DISABLED_SKILL_ENTRY_FILE,
            "# Disabled skill\n\nStill readable.\n",
        );

        assert!(!skill.summary.enabled);
        assert!(!skill.detail.enabled);
        assert!(skill.detail.entry_file_path.ends_with(SKILL_ENTRY_FILE));
        assert!(skill.detail.skill_path.ends_with("disabled-entry-skill"));
    }

    #[test]
    fn scan_skills_warns_when_both_enabled_and_disabled_entries_exist() {
        let root = temp_dir("conflicting-skill-entries");
        let skill_dir = root.join("demo-skill");
        write_skill_entry(&skill_dir, SKILL_ENTRY_FILE, "# Enabled skill\n\nBody.\n");
        write_skill_entry(
            &skill_dir,
            DISABLED_SKILL_ENTRY_FILE,
            "# Disabled skill\n\nBody.\n",
        );

        let skills = scan_skills(vec![SkillScanTargetDto {
            agent_id: "agent-claude".into(),
            agent_type: "claude".into(),
            root_path: root.to_string_lossy().to_string(),
            display_name: "Claude Main".into(),
            source: "skills".into(),
        }]);

        let skill = skills.first().expect("skill should be parsed");
        assert!(skill.summary.enabled);
        assert!(skill.detail.warnings.iter().any(
            |warning| warning == "Conflicting skill entry files found; using enabled SKILL.md."
        ));

        fs::remove_dir_all(&root).expect("cleanup temp dir");
    }

    #[test]
    fn scan_skills_reads_claude_commands_markdown_files_from_tilde_root() {
        let home = temp_dir("claude-commands-tilde");
        let commands_root = home.join(".claude").join("commands");

        fs::create_dir_all(&commands_root).expect("create commands dir");
        fs::write(
            commands_root.join("feat.md"),
            "---\nname: Feature command\ndescription: Command description\n---\n\n# Feature command\n\nRun feature flow.\n",
        )
        .expect("write command markdown");
        fs::write(
            commands_root.join("workflow.md"),
            "# Workflow\n\nRun workflow.",
        )
        .expect("write workflow markdown");

        let _user_home = UserHomeEnvGuard::set(&home);

        let skills = scan_skills(vec![SkillScanTargetDto {
            agent_id: "agent-claude".into(),
            agent_type: "claude".into(),
            root_path: "~/.claude/commands".into(),
            display_name: "Claude Main".into(),
            source: COMMANDS_SOURCE.into(),
        }]);

        let mut ids = skills
            .iter()
            .map(|skill| skill.summary.id.clone())
            .collect::<Vec<_>>();
        ids.sort();

        assert_eq!(
            ids,
            vec![
                "agent-claude::commands::feat".to_string(),
                "agent-claude::commands::workflow".to_string(),
            ]
        );
        assert!(skills.iter().all(|skill| skill.summary.kind == "skill"));
        assert!(skills
            .iter()
            .all(|skill| skill.detail.entry_file_path.ends_with(".md")));
        assert!(skills
            .iter()
            .all(|skill| skill.detail.skill_path == skill.detail.entry_file_path));

        fs::remove_dir_all(&home).expect("cleanup temp dir");
    }
    #[test]
    fn scan_skills_reads_disabled_command_entry_with_canonical_entry_path() {
        let home = temp_dir("claude-commands-disabled");
        let commands_root = home.join(".claude").join("commands");

        fs::create_dir_all(&commands_root).expect("create commands dir");
        fs::write(
            commands_root.join("feat.md.disabled"),
            "# Feature command\n\nRun feature flow.\n",
        )
        .expect("write disabled command markdown");

        let _user_home = UserHomeEnvGuard::set(&home);

        let skills = scan_skills(vec![SkillScanTargetDto {
            agent_id: "agent-claude".into(),
            agent_type: "claude".into(),
            root_path: "~/.claude/commands".into(),
            display_name: "Claude Main".into(),
            source: COMMANDS_SOURCE.into(),
        }]);

        let skill = skills.first().expect("skill should be parsed");
        assert_eq!(skill.summary.id, "agent-claude::commands::feat");
        assert!(!skill.summary.enabled);
        assert_eq!(
            skill.detail.entry_file_path.replace('\\', "/"),
            normalize_path(&commands_root.join("feat.md"))
        );
        assert_eq!(
            skill.detail.skill_path.replace('\\', "/"),
            normalize_path(&commands_root.join("feat.md.disabled"))
        );

        fs::remove_dir_all(&home).expect("cleanup temp dir");
    }

    #[test]
    fn scan_skills_reads_nested_claude_commands_markdown_files_from_tilde_root() {
        let home = temp_dir("claude-commands-nested-tilde");
        let nested_commands_root = home.join(".claude").join("commands").join("zcf");

        fs::create_dir_all(&nested_commands_root).expect("create nested commands dir");
        fs::write(
            nested_commands_root.join("feat.md"),
            "# Feat\n\nNested command.",
        )
        .expect("write nested feat markdown");
        fs::write(
            nested_commands_root.join("workflow.md"),
            "# Workflow\n\nNested workflow.",
        )
        .expect("write nested workflow markdown");

        let _user_home = UserHomeEnvGuard::set(&home);

        let skills = scan_skills(vec![SkillScanTargetDto {
            agent_id: "agent-claude".into(),
            agent_type: "claude".into(),
            root_path: "~/.claude/commands".into(),
            display_name: "Claude Main".into(),
            source: COMMANDS_SOURCE.into(),
        }]);

        let mut ids = skills
            .iter()
            .map(|skill| skill.summary.id.clone())
            .collect::<Vec<_>>();
        ids.sort();

        assert_eq!(
            ids,
            vec![
                "agent-claude::commands::zcf/feat".to_string(),
                "agent-claude::commands::zcf/workflow".to_string(),
            ]
        );

        fs::remove_dir_all(&home).expect("cleanup temp dir");
    }

    #[test]
    fn scan_skills_uses_frontmatter_description_when_yaml_is_valid() {
        let skill = scan_test_skill(
            "valid-frontmatter-skill",
            "---\nname: Valid frontmatter skill\ndescription: \"Description from frontmatter\"\n---\n\n# Heading\n\nSummary from markdown.\n",
        );

        assert_eq!(skill.detail.description, "Description from frontmatter");
        assert_eq!(skill.summary.description, "Description from frontmatter");
        assert!(skill.detail.warnings.is_empty());
    }

    #[test]
    fn scan_skills_recovers_single_line_description_from_invalid_yaml() {
        let skill = scan_test_skill(
            "invalid-frontmatter-single-line",
            "---\nname: invalid-frontmatter-single-line\ndescription: A股实时行情与分时量能分析。Use when: 查询实时行情\n---\n\n# Heading\n\nSummary from markdown.\n",
        );

        assert_eq!(
            skill.detail.description,
            "A股实时行情与分时量能分析。Use when: 查询实时行情"
        );
        assert_eq!(
            skill.summary.description,
            "A股实时行情与分时量能分析。Use when: 查询实时行情"
        );
        assert!(skill
            .detail
            .warnings
            .iter()
            .any(|warning| warning == "Failed to parse frontmatter."));
    }

    #[test]
    fn scan_skills_recovers_block_description_from_invalid_yaml() {
        let skill = scan_test_skill(
            "invalid-frontmatter-block",
            "---\nname: invalid-frontmatter-block\ndescription: >\n  第一行描述。\n  第二行描述。\nbad: [oops\n---\n\n# Heading\n\nSummary from markdown.\n",
        );

        assert_eq!(skill.detail.description, "第一行描述。 第二行描述。");
        assert_eq!(skill.summary.description, "第一行描述。 第二行描述。");
        assert!(skill
            .detail
            .warnings
            .iter()
            .any(|warning| warning == "Failed to parse frontmatter."));
    }

    #[test]
    fn scan_skills_falls_back_to_summary_when_description_cannot_be_recovered() {
        let skill = scan_test_skill(
            "invalid-frontmatter-no-description",
            "---\nname: invalid-frontmatter-no-description\ntags: [oops\n---\n\n# Heading\n\nSummary from markdown.\n",
        );

        assert_eq!(skill.detail.description, "Summary from markdown.");
        assert_eq!(skill.summary.description, "Summary from markdown.");
        assert!(skill
            .detail
            .warnings
            .iter()
            .any(|warning| warning == "Failed to parse frontmatter."));
    }
}
