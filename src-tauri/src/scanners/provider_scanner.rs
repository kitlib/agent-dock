use std::{env, fs, path::PathBuf};

use crate::dto::agents::{AgentResourceCountsDto, DiscoveredAgentDto};

#[derive(Clone)]
pub struct ProviderScanTarget {
    pub provider: &'static str,
    pub root_path: PathBuf,
    pub config_path: Option<PathBuf>,
    pub source_scope: &'static str,
}

pub fn default_scan_targets() -> Vec<ProviderScanTarget> {
    vec![
        ProviderScanTarget {
            provider: "cursor",
            root_path: PathBuf::from(".cursor"),
            config_path: Some(PathBuf::from(".cursor/mcp.json")),
            source_scope: "workspace",
        },
        ProviderScanTarget {
            provider: "claude",
            root_path: PathBuf::from(".claude"),
            config_path: Some(PathBuf::from(".claude/settings.json")),
            source_scope: "user",
        },
        ProviderScanTarget {
            provider: "codex",
            root_path: PathBuf::from(".codex"),
            config_path: Some(PathBuf::from(".codex/config.toml")),
            source_scope: "user",
        },
        ProviderScanTarget {
            provider: "antigravity",
            root_path: PathBuf::from(".agent"),
            config_path: Some(PathBuf::from(".agent/config.json")),
            source_scope: "manual",
        },
    ]
}

fn current_dir() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn provider_display_name(provider: &str) -> String {
    match provider {
        "cursor" => "Cursor",
        "claude" => "Claude Code",
        "codex" => "Codex CLI",
        "antigravity" => "Antigravity",
        _ => "Managed Agent",
    }
    .into()
}

fn provider_fingerprint(provider: &str, source_scope: &str) -> String {
    format!("{}-{}-default", provider, source_scope)
}

fn provider_resource_counts(provider: &str) -> AgentResourceCountsDto {
    match provider {
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

fn detect_status(target: &ProviderScanTarget, absolute_root: &PathBuf) -> (String, Option<String>) {
    if target.provider == "antigravity" {
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

    if let Some(config_path) = &target.config_path {
        let absolute_config = current_dir().join(config_path);
        if absolute_config.exists() && fs::read_to_string(&absolute_config).is_err() {
            return (
                "unreadable".into(),
                Some("AgentDock could not read the provider config file.".into()),
            );
        }
    }

    ("discovered".into(), None)
}

pub fn scan_discovered_agents() -> Vec<DiscoveredAgentDto> {
    let base_dir = current_dir();

    default_scan_targets()
        .into_iter()
        .filter_map(|target| {
            let absolute_root = base_dir.join(&target.root_path);
            let absolute_config = target.config_path.as_ref().map(|path| base_dir.join(path));
            let has_root = absolute_root.exists();
            let has_config = absolute_config
                .as_ref()
                .map(|path| path.exists())
                .unwrap_or(false);

            if !has_root && !has_config {
                return None;
            }

            let (status, reason) = detect_status(&target, &absolute_root);

            Some(DiscoveredAgentDto {
                discovery_id: format!("discovery-{}-{}", target.provider, target.source_scope),
                fingerprint: provider_fingerprint(target.provider, target.source_scope),
                provider: target.provider.into(),
                display_name: provider_display_name(target.provider),
                root_path: target.root_path.to_string_lossy().replace('\\', "/"),
                config_path: target
                    .config_path
                    .as_ref()
                    .map(|path| path.to_string_lossy().replace('\\', "/")),
                source_scope: target.source_scope.into(),
                status,
                reason,
                resource_counts: provider_resource_counts(target.provider),
                detected_at: "2026-03-25T10:20:00Z".into(),
            })
        })
        .collect()
}
