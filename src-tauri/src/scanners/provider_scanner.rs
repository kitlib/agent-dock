use std::{env, fs, path::PathBuf};

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

fn agent_resource_counts(agent: &str) -> AgentResourceCountsDto {
    match agent {
        "cursor" => AgentResourceCountsDto {
            skill: 6,
            mcp: 1,
            subagent: 1,
        },
        "claude" => AgentResourceCountsDto {
            skill: 6,
            mcp: 1,
            subagent: 1,
        },
        "codex" => AgentResourceCountsDto {
            skill: 3,
            mcp: 1,
            subagent: 0,
        },
        "antigravity" => AgentResourceCountsDto {
            skill: 4,
            mcp: 0,
            subagent: 1,
        },
        _ => AgentResourceCountsDto {
            skill: 0,
            mcp: 0,
            subagent: 0,
        },
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
                resource_counts: agent_resource_counts(&target.agent),
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
