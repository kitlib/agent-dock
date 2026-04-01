use std::{env, fs, path::{Path, PathBuf}};

use crate::dto::agents::{AgentResourceCountsDto, DiscoveredAgentDto, ScanTargetDto};

#[derive(Clone)]
pub struct AgentScanTarget {
    pub agent: String,
    pub name: String,
    pub root_path: PathBuf,
}

pub fn scan_targets_from_dto(scan_targets: Vec<ScanTargetDto>) -> Vec<AgentScanTarget> {
    scan_targets
        .into_iter()
        .map(|target| AgentScanTarget {
            agent: target.agent,
            name: target.name,
            root_path: PathBuf::from(target.root_path),
        })
        .collect()
}

fn current_dir() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn user_home_dir() -> PathBuf {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(current_dir)
}

fn display_path(relative_path: &PathBuf) -> String {
    PathBuf::from("~")
        .join(relative_path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn agent_fingerprint(agent: &str) -> String {
    format!("{}-default", agent)
}

fn trim_trailing_slashes(value: &str) -> &str {
    value.trim_end_matches(['/', '\\'])
}

fn trim_leading_slashes(value: &str) -> &str {
    value.trim_start_matches(['/', '\\'])
}

fn build_skill_scan_root(agent: &str, root_path: &Path) -> Option<PathBuf> {
    let relative_skills_path = match agent {
        "adal"
        | "amp"
        | "antigravity"
        | "augment"
        | "claude"
        | "claude-plugin"
        | "cline"
        | "codebuddy"
        | "codex"
        | "command-code"
        | "continue"
        | "crush"
        | "cursor"
        | "factory"
        | "github-copilot"
        | "goose"
        | "iflow"
        | "junie"
        | "kilo"
        | "kimi"
        | "kiro"
        | "kode"
        | "mcpjam"
        | "mistral"
        | "mux"
        | "neovate"
        | "openclaw"
        | "opencode"
        | "openhands"
        | "pochi"
        | "qoder"
        | "qwen"
        | "replit"
        | "roo"
        | "trae"
        | "trae-cn"
        | "warp"
        | "windsurf"
        | "zencoder" => "skills/",
        "pi-mono" => "agent/skills/",
        _ => return None,
    };

    let normalized_root = PathBuf::from(trim_trailing_slashes(&root_path.to_string_lossy()).to_string());
    let normalized_relative = trim_leading_slashes(trim_trailing_slashes(relative_skills_path));
    Some(normalized_root.join(normalized_relative))
}

fn count_skill_directories(skills_root: &Path) -> u32 {
    if !skills_root.exists() || !skills_root.is_dir() {
        return 0;
    }

    fs::read_dir(skills_root)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter(|entry| {
            let path = entry.path();
            path.is_dir() && path.join("SKILL.md").exists()
        })
        .count() as u32
}

fn agent_resource_counts(agent: &str, absolute_root: &Path) -> AgentResourceCountsDto {
    let skill = build_skill_scan_root(agent, absolute_root)
        .map(|skills_root| count_skill_directories(&skills_root))
        .unwrap_or(0);

    let (mcp, subagent) = match agent {
        "cursor" => (1, 1),
        "claude" => (1, 1),
        "codex" => (1, 0),
        "antigravity" => (0, 1),
        _ => (0, 0),
    };

    AgentResourceCountsDto {
        skill,
        mcp,
        subagent,
    }
}

fn detect_status(target: &AgentScanTarget, absolute_root: &PathBuf) -> (String, Option<String>) {
    if target.agent == "antigravity" {
        let workflows_path = absolute_root.join("workflows");
        if workflows_path.exists() {
            let unreadable_workflow = fs::read_dir(&workflows_path)
                .ok()
                .into_iter()
                .flat_map(|entries| entries.filter_map(Result::ok))
                .any(|entry| fs::read_to_string(entry.path()).is_err());

            if unreadable_workflow {
                return (
                    "unreadable".into(),
                    Some("AgentDock could not read one workflow file.".into()),
                );
            }
        }
    }

    ("discovered".into(), None)
}

pub fn scan_discovered_agents(scan_targets: Vec<ScanTargetDto>) -> Vec<DiscoveredAgentDto> {
    println!(
        "[agent-scan] start user_home_dir={}",
        user_home_dir().display()
    );

    let discovered_agents: Vec<_> = scan_targets_from_dto(scan_targets)
        .into_iter()
        .filter_map(|target| {
            let base_dir = user_home_dir();
            let absolute_root = base_dir.join(&target.root_path);

            println!(
                "[agent-scan] checking agent={} baseDir={} root={} exists={}",
                target.agent,
                base_dir.display(),
                absolute_root.display(),
                absolute_root.exists()
            );

            if !absolute_root.exists() {
                return None;
            }

            let (status, reason) = detect_status(&target, &absolute_root);

            let agent = DiscoveredAgentDto {
                discovery_id: format!("discovery-{}", target.agent),
                fingerprint: agent_fingerprint(&target.agent),
                provider: target.agent.clone(),
                display_name: target.name.clone(),
                root_path: display_path(&target.root_path),
                status,
                reason,
                resource_counts: agent_resource_counts(&target.agent, &absolute_root),
                detected_at: "2026-03-25T10:20:00Z".into(),
            };

            println!(
                "[agent-scan] discovered agent={} status={} rootPath={} reason={}",
                agent.provider,
                agent.status,
                agent.root_path,
                agent.reason.as_deref().unwrap_or("<none>")
            );

            Some(agent)
        })
        .collect();

    println!("[agent-scan] finished count={}", discovered_agents.len());

    discovered_agents
}

#[cfg(test)]
mod tests {
    use super::{agent_resource_counts, build_skill_scan_root};
    use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-dock-{name}-{unique}"))
    }

    #[test]
    fn build_skill_scan_root_matches_provider_layout() {
        let claude_root = PathBuf::from("C:/Users/test/.claude");
        let pi_root = PathBuf::from("C:/Users/test/.pi");

        assert_eq!(
            build_skill_scan_root("claude", &claude_root),
            Some(PathBuf::from("C:/Users/test/.claude/skills"))
        );
        assert_eq!(
            build_skill_scan_root("pi-mono", &pi_root),
            Some(PathBuf::from("C:/Users/test/.pi/agent/skills"))
        );
    }

    #[test]
    fn agent_resource_counts_counts_only_skill_directories_with_skill_md() {
        let root = temp_dir("provider-scan");
        let skills_root = root.join("skills");
        let valid_skill = skills_root.join("release-checklist");
        let invalid_skill = skills_root.join("notes-only");

        fs::create_dir_all(&valid_skill).expect("create valid skill dir");
        fs::create_dir_all(&invalid_skill).expect("create invalid skill dir");
        fs::write(valid_skill.join("SKILL.md"), "# Release checklist").expect("write skill markdown");
        fs::write(invalid_skill.join("README.md"), "not a skill").expect("write non skill file");

        let counts = agent_resource_counts("claude", &root);
        assert_eq!(counts.skill, 1);
        assert_eq!(counts.mcp, 1);
        assert_eq!(counts.subagent, 1);

        fs::remove_dir_all(&root).expect("cleanup temp dir");
    }

    #[test]
    fn agent_resource_counts_returns_zero_when_skills_root_missing() {
        let root = temp_dir("provider-scan-missing");
        fs::create_dir_all(&root).expect("create temp root");

        let counts = agent_resource_counts("claude", &root);
        assert_eq!(counts.skill, 0);

        fs::remove_dir_all(&root).expect("cleanup temp dir");
    }
}
