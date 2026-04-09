use std::{env, fs, path::{Path, PathBuf}};

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::dto::skills::{
    LocalSkillDetailDto, LocalSkillSummaryDto, SkillScanTargetDto, SkillSupportingFileDto,
};

const COMMANDS_SOURCE: &str = "commands";
const SKILL_ENTRY_FILE: &str = "SKILL.md";
const DISABLED_SKILL_ENTRY_FILE: &str = "SKILL.md.disabled";

#[derive(Clone)]
pub struct ParsedSkill {
    pub summary: LocalSkillSummaryDto,
    pub detail: LocalSkillDetailDto,
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn skill_id(agent_id: &str, skill_name: &str, source: &str) -> String {
    format!("{agent_id}::{source}::{skill_name}")
}

fn split_frontmatter(contents: &str) -> (Option<String>, String) {
    let normalized = contents.replace("\r\n", "\n");
    if !normalized.starts_with("---\n") {
        return (None, normalized);
    }

    let remainder = &normalized[4..];
    if let Some(index) = remainder.find("\n---\n") {
        let frontmatter = remainder[..index].to_string();
        let body = remainder[index + 5..].to_string();
        (Some(frontmatter), body)
    } else {
        (None, normalized)
    }
}

fn summary_from_markdown(markdown: &str) -> String {
    markdown
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.chars().take(140).collect())
        .unwrap_or_else(|| "Local skill".into())
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

fn allowed_tools_from_frontmatter(frontmatter: Option<&Value>) -> Vec<String> {
    frontmatter
        .and_then(|value| value.get("allowed_tools").or_else(|| value.get("allowedTools")))
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

fn description_from_frontmatter(frontmatter: Option<&Value>) -> Option<String> {
    frontmatter
        .and_then(|value| value.get("description"))
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn description_from_frontmatter_raw(frontmatter_raw: Option<&str>) -> Option<String> {
    let mut lines = frontmatter_raw?.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() || line != trimmed {
            continue;
        }

        let Some(raw_value) = trimmed.strip_prefix("description:") else {
            continue;
        };

        let value = raw_value.trim();
        if value.is_empty() {
            return None;
        }

        if matches!(value, "|" | ">" | "|-" | ">-" | "|+" | ">+") {
            let mut block_lines = Vec::new();

            while let Some(next_line) = lines.peek() {
                if next_line.trim().is_empty() {
                    block_lines.push(String::new());
                    lines.next();
                    continue;
                }

                if next_line.trim_start() == *next_line {
                    break;
                }

                block_lines.push(next_line.trim().to_string());
                lines.next();
            }

            let block_text = if value.starts_with('>') {
                block_lines.join(" ")
            } else {
                block_lines.join("\n")
            };
            let normalized = block_text.trim().to_string();
            return (!normalized.is_empty()).then_some(normalized);
        }

        let normalized = value
            .trim_matches(|character| matches!(character, '"' | '\''))
            .trim()
            .to_string();
        return (!normalized.is_empty()).then_some(normalized);
    }

    None
}

fn resolved_description(
    frontmatter: Option<&Value>,
    frontmatter_raw: Option<&str>,
    fallback: &str,
) -> String {
    description_from_frontmatter(frontmatter)
        .or_else(|| description_from_frontmatter_raw(frontmatter_raw))
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

fn supporting_files(path: &Path, entry_file_name: &str) -> Vec<SkillSupportingFileDto> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };

    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some(entry_file_name) {
                return None;
            }
            Some(SkillSupportingFileDto {
                path: normalize_path(&path),
            })
        })
        .collect();
    files.sort_by(|left, right| left.path.cmp(&right.path));
    files
}

fn user_home_dir() -> PathBuf {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn resolve_scan_root(root_path: &str) -> PathBuf {
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

fn command_name(scan_root: &Path, entry_file: &Path) -> Option<String> {
    let relative_path = entry_file.strip_prefix(scan_root).ok()?;
    let without_extension = relative_path.with_extension("");
    Some(normalize_path(&without_extension))
}

fn display_name_from_path(path: &str) -> String {
    path.replace('/', ":")
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

    let skill_name = if scan_target.source == COMMANDS_SOURCE {
        command_name(scan_root, &entry_file)?
    } else {
        skill_path.file_name()?.to_string_lossy().to_string()
    };
    let id = skill_id(&scan_target.agent_id, &skill_name, &scan_target.source);
    let updated_at = updated_at(&entry_file);
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if scan_target.source != COMMANDS_SOURCE && enabled && skill_path.join(DISABLED_SKILL_ENTRY_FILE).exists() {
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

    let summary_text = summary_from_markdown(&markdown);
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
    let allowed_tools = allowed_tools_from_frontmatter(frontmatter.as_ref());
    let status = if errors.is_empty() {
        if warnings.is_empty() { "ready" } else { "warning" }
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
        description: description.clone(),
        status: status.clone(),
        skill_path: normalize_path(&skill_path),
        entry_file_path: normalize_path(&entry_file),
        agent_type: scan_target.agent_type.clone(),
        agent_name: scan_target.display_name.clone(),
        warnings: warnings.clone(),
        errors: errors.clone(),
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
        status,
        skill_path: normalize_path(&skill_path),
        entry_file_path: normalize_path(&entry_file),
        agent_type: scan_target.agent_type.clone(),
        agent_name: scan_target.display_name.clone(),
        warnings,
        errors,
        frontmatter,
        frontmatter_raw,
        supporting_files: supporting_files(&skill_path, entry_file.file_name()?.to_str()?),
        allowed_tools,
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

        if path.is_file() && path.extension().and_then(|extension| extension.to_str()) == Some("md") {
            files.push(path);
        }
    }

    files.sort();
    files
}

pub fn scan_skills(scan_targets: Vec<SkillScanTargetDto>) -> Vec<ParsedSkill> {
    let mut parsed_skills = Vec::new();

    for scan_target in scan_targets {
        let scan_root = resolve_scan_root(&scan_target.root_path);
        if !scan_root.exists() || !scan_root.is_dir() {
            continue;
        }

        if scan_target.source == COMMANDS_SOURCE {
            for path in collect_command_markdown_files(&scan_root) {
                if let Some(skill) = parse_skill(&scan_target, &scan_root, path.clone(), path, true) {
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
                if let Some(skill) = parse_skill(&scan_target, &scan_root, path.clone(), entry_file, enabled) {
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
        resolve_scan_root, scan_skills, COMMANDS_SOURCE, DISABLED_SKILL_ENTRY_FILE, SKILL_ENTRY_FILE,
    };
    use crate::dto::skills::SkillScanTargetDto;
    use std::{env, fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

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

    fn scan_test_skill(skill_name: &str, contents: &str) -> crate::scanners::skill_scanner::ParsedSkill {
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
        let home = env::var_os("USERPROFILE")
            .or_else(|| env::var_os("HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        assert_eq!(resolve_scan_root(".claude/skills"), home.join(".claude/skills"));
    }

    #[test]
    fn resolve_scan_root_expands_tilde_prefixed_paths() {
        let home = env::var_os("USERPROFILE")
            .or_else(|| env::var_os("HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        assert_eq!(resolve_scan_root("~/.claude/skills"), home.join(".claude/skills"));
    }

    #[test]
    fn scan_skills_reads_windows_claude_home_skill_directory_from_tilde_root() {
        let home = temp_dir("claude-home-tilde");
        let skills_root = home.join(".claude").join("skills");
        let first_skill = skills_root.join("find-skills");
        let second_skill = skills_root.join("peon-ping-log");

        fs::create_dir_all(&first_skill).expect("create first skill dir");
        fs::create_dir_all(&second_skill).expect("create second skill dir");
        fs::write(first_skill.join("SKILL.md"), "# Find skills\nLocate local skills.")
            .expect("write first skill markdown");
        fs::write(second_skill.join("SKILL.md"), "# Peon ping log\nInspect ping logs.")
            .expect("write second skill markdown");

        let previous_userprofile = env::var_os("USERPROFILE");
        env::set_var("USERPROFILE", &home);

        let skills = scan_skills(vec![SkillScanTargetDto {
            agent_id: "agent-claude".into(),
            agent_type: "claude".into(),
            root_path: "~/.claude/skills".into(),
            display_name: "Claude Main".into(),
            source: "skills".into(),
        }]);

        if let Some(value) = previous_userprofile {
            env::set_var("USERPROFILE", value);
        } else {
            env::remove_var("USERPROFILE");
        }

        let mut ids = skills
            .iter()
            .map(|skill| skill.summary.id.clone())
            .collect::<Vec<_>>();
        ids.sort();

        assert_eq!(ids, vec![
            "agent-claude::skills::find-skills".to_string(),
            "agent-claude::skills::peon-ping-log".to_string(),
        ]);

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
        assert!(skill.detail.entry_file_path.ends_with(DISABLED_SKILL_ENTRY_FILE));
    }

    #[test]
    fn scan_skills_warns_when_both_enabled_and_disabled_entries_exist() {
        let root = temp_dir("conflicting-skill-entries");
        let skill_dir = root.join("demo-skill");
        write_skill_entry(&skill_dir, SKILL_ENTRY_FILE, "# Enabled skill\n\nBody.\n");
        write_skill_entry(&skill_dir, DISABLED_SKILL_ENTRY_FILE, "# Disabled skill\n\nBody.\n");

        let skills = scan_skills(vec![SkillScanTargetDto {
            agent_id: "agent-claude".into(),
            agent_type: "claude".into(),
            root_path: root.to_string_lossy().to_string(),
            display_name: "Claude Main".into(),
            source: "skills".into(),
        }]);

        let skill = skills.first().expect("skill should be parsed");
        assert!(skill.summary.enabled);
        assert!(skill
            .detail
            .warnings
            .iter()
            .any(|warning| warning == "Conflicting skill entry files found; using enabled SKILL.md."));

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

        let previous_userprofile = env::var_os("USERPROFILE");
        env::set_var("USERPROFILE", &home);

        let skills = scan_skills(vec![SkillScanTargetDto {
            agent_id: "agent-claude".into(),
            agent_type: "claude".into(),
            root_path: "~/.claude/commands".into(),
            display_name: "Claude Main".into(),
            source: COMMANDS_SOURCE.into(),
        }]);

        if let Some(value) = previous_userprofile {
            env::set_var("USERPROFILE", value);
        } else {
            env::remove_var("USERPROFILE");
        }

        let mut ids = skills
            .iter()
            .map(|skill| skill.summary.id.clone())
            .collect::<Vec<_>>();
        ids.sort();

        assert_eq!(ids, vec![
            "agent-claude::commands::feat".to_string(),
            "agent-claude::commands::workflow".to_string(),
        ]);
        assert!(skills.iter().all(|skill| skill.summary.kind == "skill"));
        assert!(skills.iter().all(|skill| skill.detail.entry_file_path.ends_with(".md")));

        fs::remove_dir_all(&home).expect("cleanup temp dir");
    }
    #[test]
    fn scan_skills_reads_nested_claude_commands_markdown_files_from_tilde_root() {
        let home = temp_dir("claude-commands-nested-tilde");
        let nested_commands_root = home.join(".claude").join("commands").join("zcf");

        fs::create_dir_all(&nested_commands_root).expect("create nested commands dir");
        fs::write(nested_commands_root.join("feat.md"), "# Feat\n\nNested command.")
            .expect("write nested feat markdown");
        fs::write(nested_commands_root.join("workflow.md"), "# Workflow\n\nNested workflow.")
            .expect("write nested workflow markdown");

        let previous_userprofile = env::var_os("USERPROFILE");
        env::set_var("USERPROFILE", &home);

        let skills = scan_skills(vec![SkillScanTargetDto {
            agent_id: "agent-claude".into(),
            agent_type: "claude".into(),
            root_path: "~/.claude/commands".into(),
            display_name: "Claude Main".into(),
            source: COMMANDS_SOURCE.into(),
        }]);

        if let Some(value) = previous_userprofile {
            env::set_var("USERPROFILE", value);
        } else {
            env::remove_var("USERPROFILE");
        }

        let mut ids = skills
            .iter()
            .map(|skill| skill.summary.id.clone())
            .collect::<Vec<_>>();
        ids.sort();

        assert_eq!(ids, vec![
            "agent-claude::commands::zcf/feat".to_string(),
            "agent-claude::commands::zcf/workflow".to_string(),
        ]);

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

        assert_eq!(skill.detail.description, "A股实时行情与分时量能分析。Use when: 查询实时行情");
        assert_eq!(skill.summary.description, "A股实时行情与分时量能分析。Use when: 查询实时行情");
        assert!(skill.detail.warnings.iter().any(|warning| warning == "Failed to parse frontmatter."));
    }

    #[test]
    fn scan_skills_recovers_block_description_from_invalid_yaml() {
        let skill = scan_test_skill(
            "invalid-frontmatter-block",
            "---\nname: invalid-frontmatter-block\ndescription: >\n  第一行描述。\n  第二行描述。\nbad: [oops\n---\n\n# Heading\n\nSummary from markdown.\n",
        );

        assert_eq!(skill.detail.description, "第一行描述。 第二行描述。");
        assert_eq!(skill.summary.description, "第一行描述。 第二行描述。");
        assert!(skill.detail.warnings.iter().any(|warning| warning == "Failed to parse frontmatter."));
    }

    #[test]
    fn scan_skills_falls_back_to_summary_when_description_cannot_be_recovered() {
        let skill = scan_test_skill(
            "invalid-frontmatter-no-description",
            "---\nname: invalid-frontmatter-no-description\ntags: [oops\n---\n\n# Heading\n\nSummary from markdown.\n",
        );

        assert_eq!(skill.detail.description, "Summary from markdown.");
        assert_eq!(skill.summary.description, "Summary from markdown.");
        assert!(skill.detail.warnings.iter().any(|warning| warning == "Failed to parse frontmatter."));
    }

}
