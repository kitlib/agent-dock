use std::{env, fs, path::{Path, PathBuf}};

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::dto::skills::{
    LocalSkillDetailDto, LocalSkillSummaryDto, SkillScanTargetDto, SkillSupportingFileDto,
};

#[derive(Clone)]
pub struct ParsedSkill {
    pub summary: LocalSkillSummaryDto,
    pub detail: LocalSkillDetailDto,
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn skill_id(agent_id: &str, skill_dir_name: &str) -> String {
    format!("{agent_id}::{skill_dir_name}")
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
        .unwrap_or_else(|| fallback.replace('-', " "))
}

fn description_from_frontmatter(frontmatter: Option<&Value>, fallback: &str) -> String {
    frontmatter
        .and_then(|value| value.get("description"))
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

fn supporting_files(skill_dir: &Path) -> Vec<SkillSupportingFileDto> {
    let Ok(entries) = fs::read_dir(skill_dir) else {
        return Vec::new();
    };

    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some("SKILL.md") {
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

fn parse_skill(scan_target: &SkillScanTargetDto, skill_dir: PathBuf) -> Option<ParsedSkill> {
    let entry_file = skill_dir.join("SKILL.md");
    if !entry_file.exists() {
        return None;
    }

    let skill_dir_name = skill_dir.file_name()?.to_string_lossy().to_string();
    let id = skill_id(&scan_target.agent_id, &skill_dir_name);
    let updated_at = updated_at(&entry_file);
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

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
    let name = name_from_frontmatter(frontmatter.as_ref(), &skill_dir_name);
    let description = description_from_frontmatter(frontmatter.as_ref(), &summary_text);
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
        enabled: true,
        tags: tags.clone(),
        usage_count: 0,
        updated_at: updated_at.clone(),
        owner_agent_id: scan_target.agent_id.clone(),
        source_label: format!("{} local", scan_target.display_name),
        description: description.clone(),
        status: status.clone(),
        skill_path: normalize_path(&skill_dir),
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
        enabled: true,
        tags,
        usage_count: 0,
        updated_at,
        markdown,
        owner_agent_id: scan_target.agent_id.clone(),
        source_label: format!("{} local", scan_target.display_name),
        status,
        skill_path: normalize_path(&skill_dir),
        entry_file_path: normalize_path(&entry_file),
        agent_type: scan_target.agent_type.clone(),
        agent_name: scan_target.display_name.clone(),
        warnings,
        errors,
        frontmatter,
        frontmatter_raw,
        supporting_files: supporting_files(&skill_dir),
        allowed_tools,
    };

    Some(ParsedSkill { summary, detail })
}

pub fn scan_skills(scan_targets: Vec<SkillScanTargetDto>) -> Vec<ParsedSkill> {
    let mut parsed_skills = Vec::new();

    for scan_target in scan_targets {
        let skills_root = resolve_scan_root(&scan_target.root_path);
        if !skills_root.exists() || !skills_root.is_dir() {
            continue;
        }

        let Ok(entries) = fs::read_dir(&skills_root) else {
            continue;
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if let Some(skill) = parse_skill(&scan_target, path) {
                parsed_skills.push(skill);
            }
        }
    }

    parsed_skills.sort_by(|left, right| right.summary.updated_at.cmp(&left.summary.updated_at));
    parsed_skills
}

#[cfg(test)]
mod tests {
    use super::{resolve_scan_root, scan_skills};
    use crate::dto::skills::SkillScanTargetDto;
    use std::{env, fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-dock-{name}-{unique}"))
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
            "agent-claude::find-skills".to_string(),
            "agent-claude::peon-ping-log".to_string(),
        ]);

        fs::remove_dir_all(&home).expect("cleanup temp dir");
    }

    #[test]
    fn scan_skills_reads_windows_claude_home_skill_directory_from_relative_root() {
        let home = temp_dir("claude-home");
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
            root_path: ".claude/skills".into(),
            display_name: "Claude Main".into(),
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
            "agent-claude::find-skills".to_string(),
            "agent-claude::peon-ping-log".to_string(),
        ]);
        assert!(skills.iter().all(|skill| skill.summary.owner_agent_id == "agent-claude"));
        assert!(skills.iter().all(|skill| skill.summary.skill_path.contains("/.claude/skills/")));

        fs::remove_dir_all(&home).expect("cleanup temp dir");
    }
}
