use std::collections::HashSet;

use crate::dto::agents::{
    AgentResourceCountsDto, CreateAgentResultDto, DeleteAgentResultDto, DiscoveredAgentDto,
    ImportAgentsResultDto, ManagedAgentDto, ManualAgentDraftDto, RemoveAgentResultDto,
    ResolvedAgentDto, ScanTargetDto, ScannedAgentCandidateDto,
};
use crate::persistence::managed_agents_store;
use crate::scanners::provider_scanner;

fn role_for_provider(provider: &str) -> String {
    match provider {
        "cursor" => "AI coding assistant",
        "claude" => "CLI coding assistant",
        "codex" => "Terminal coding assistant",
        "antigravity" => "Workflow automation assistant",
        _ => "Managed agent",
    }
    .into()
}

fn discovered_summary(provider: &str) -> String {
    match provider {
        "cursor" => "Detected Cursor in the user directory and ready to import into AgentDock.",
        "claude" => "Detected Claude Code in the user directory and ready to import into AgentDock.",
        "codex" => "Detected Codex CLI in the user directory and ready to import into AgentDock.",
        "antigravity" => "Detected Antigravity in the user directory and ready to import into AgentDock.",
        _ => "Detected in the user directory and ready to import into AgentDock.",
    }
    .into()
}

fn manual_resolved_agent(entry: &ManagedAgentDto) -> ResolvedAgentDto {
    let id_suffix = entry
        .managed_agent_id
        .strip_prefix("managed-")
        .unwrap_or(entry.managed_agent_id.as_str());
    let provider = entry.provider.as_deref().unwrap_or_else(|| {
        entry
            .fingerprint
            .split_once('-')
            .map(|(provider, _)| provider)
            .unwrap_or("claude")
    });
    let name = entry.alias.clone().unwrap_or_else(|| {
        id_suffix
            .split('-')
            .filter(|part| !part.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    });
    let is_managed = !entry.hidden;

    ResolvedAgentDto {
        id: format!("agent-{}", id_suffix),
        discovery_id: format!("discovery-{}", id_suffix),
        fingerprint: entry.fingerprint.clone(),
        provider: provider.into(),
        name,
        alias: entry.alias.clone(),
        role: "Manually managed agent".into(),
        root_path: entry.root_path.clone().unwrap_or_else(|| ".".into()),
        managed: is_managed,
        managed_agent_id: Some(entry.managed_agent_id.clone()),
        enabled: if is_managed { entry.enabled } else { false },
        hidden: false,
        health: "ok".into(),
        status: "discovered".into(),
        status_label: if is_managed {
            "Managed".into()
        } else {
            "Saved".into()
        },
        summary: if is_managed {
            "Created manually and ready for local resource management.".into()
        } else {
            "Saved manually and ready to import back into AgentDock.".into()
        },
        group_id: "assistant".into(),
        resource_counts: AgentResourceCountsDto {
            skill: 0,
            mcp: 0,
            subagent: 0,
        },
        last_scanned_at: entry.imported_at.clone(),
    }
}

fn merge_resolved_agents(
    discovered_agents: &[DiscoveredAgentDto],
    managed_agents: &[ManagedAgentDto],
) -> Vec<ResolvedAgentDto> {
    let discovered_fingerprints: HashSet<_> = discovered_agents
        .iter()
        .map(|agent| agent.fingerprint.as_str())
        .collect();
    let mut resolved_agents: Vec<_> = discovered_agents
        .iter()
        .map(|agent| {
            let managed = managed_agents
                .iter()
                .find(|entry| entry.fingerprint == agent.fingerprint);

            ResolvedAgentDto {
                id: agent.discovery_id.replacen("discovery-", "agent-", 1),
                discovery_id: agent.discovery_id.clone(),
                fingerprint: agent.fingerprint.clone(),
                provider: agent.provider.clone(),
                name: agent.display_name.clone(),
                alias: managed.and_then(|entry| entry.alias.clone()),
                role: role_for_provider(&agent.provider),
                root_path: agent.root_path.clone(),
                managed: managed.map(|entry| !entry.hidden).unwrap_or(false),
                managed_agent_id: managed.map(|entry| entry.managed_agent_id.clone()),
                enabled: managed
                    .map(|entry| !entry.hidden && entry.enabled)
                    .unwrap_or(false),
                hidden: managed.map(|entry| entry.hidden).unwrap_or(false),
                health: if agent.status == "unreadable" {
                    "error".into()
                } else {
                    "ok".into()
                },
                status: agent.status.clone(),
                status_label: if managed.map(|entry| !entry.hidden).unwrap_or(false) {
                    "Managed".into()
                } else if agent.status == "unreadable" {
                    "Unreadable".into()
                } else {
                    "Discovered".into()
                },
                summary: if managed.is_some() {
                    format!("Imported {} into AgentDock management.", agent.display_name)
                } else {
                    discovered_summary(&agent.provider)
                },
                group_id: "assistant".into(),
                resource_counts: agent.resource_counts.clone(),
                last_scanned_at: agent.detected_at.clone(),
            }
        })
        .collect();

    for entry in managed_agents {
        if discovered_fingerprints.contains(entry.fingerprint.as_str()) {
            continue;
        }

        resolved_agents.push(manual_resolved_agent(entry));
    }

    resolved_agents
}

fn scanned_candidates(
    discovered_agents: &[DiscoveredAgentDto],
    resolved_agents: &[ResolvedAgentDto],
) -> Vec<ScannedAgentCandidateDto> {
    discovered_agents
        .iter()
        .map(|agent| {
            let resolved = resolved_agents
                .iter()
                .find(|entry| entry.discovery_id == agent.discovery_id);
            let state = if resolved.map(|entry| entry.managed).unwrap_or(false) {
                "imported"
            } else if agent.status == "unreadable" || agent.status == "invalid" {
                "unreadable"
            } else {
                "ready"
            };

            ScannedAgentCandidateDto {
                id: agent.discovery_id.replacen("discovery-", "candidate-", 1),
                fingerprint: agent.fingerprint.clone(),
                provider: agent.provider.clone(),
                display_name: agent.display_name.clone(),
                root_path: agent.root_path.clone(),
                resource_counts: agent.resource_counts.clone(),
                state: state.into(),
                reason: agent.reason.clone(),
                managed_agent_id: resolved.and_then(|entry| entry.managed_agent_id.clone()),
                managed: resolved.map(|entry| entry.managed).unwrap_or(false),
                detected_at: agent.detected_at.clone(),
            }
        })
        .collect()
}

fn discovered_agents_for_scan(scan_targets: Vec<ScanTargetDto>) -> Vec<DiscoveredAgentDto> {
    provider_scanner::scan_discovered_agents(scan_targets)
}

pub fn list_managed_agents() -> Vec<ManagedAgentDto> {
    managed_agents_store::load_managed_agents()
}

pub fn list_resolved_agents(scan_targets: Vec<ScanTargetDto>) -> Vec<ResolvedAgentDto> {
    let discovered_agents = discovered_agents_for_scan(scan_targets);
    let managed_agents = list_managed_agents();
    merge_resolved_agents(&discovered_agents, &managed_agents)
}

pub fn scan_agents(scan_targets: Vec<ScanTargetDto>) -> Vec<ScannedAgentCandidateDto> {
    let discovered_agents = discovered_agents_for_scan(scan_targets);
    let managed_agents = list_managed_agents();
    let resolved_agents = merge_resolved_agents(&discovered_agents, &managed_agents);
    scanned_candidates(&discovered_agents, &resolved_agents)
}

pub fn refresh_agent_discovery(scan_targets: Vec<ScanTargetDto>) -> Vec<ResolvedAgentDto> {
    let discovered_agents = discovered_agents_for_scan(scan_targets);
    let managed_agents = list_managed_agents();
    merge_resolved_agents(&discovered_agents, &managed_agents)
}

pub fn import_agents(
    candidate_ids: Vec<String>,
    scan_targets: Vec<ScanTargetDto>,
) -> Result<ImportAgentsResultDto, String> {
    let discovered_agents = discovered_agents_for_scan(scan_targets);
    let mut managed_agents = list_managed_agents();

    let mut imported_fingerprints = Vec::new();

    for candidate_id in candidate_ids {
        let discovery_id = candidate_id.replacen("candidate-", "discovery-", 1);
        let candidate_fingerprint = discovered_agents
            .iter()
            .find(|entry| entry.discovery_id == discovery_id)
            .map(|entry| entry.fingerprint.clone())
            .or_else(|| {
                managed_agents
                    .iter()
                    .find(|entry| {
                        entry.managed_agent_id.replacen("managed-", "discovery-", 1) == discovery_id
                    })
                    .map(|entry| entry.fingerprint.clone())
            });

        let Some(fingerprint) = candidate_fingerprint else {
            continue;
        };

        if let Some(existing_entry) = managed_agents
            .iter_mut()
            .find(|entry| entry.fingerprint == fingerprint)
        {
            existing_entry.enabled = true;
            existing_entry.hidden = false;
            imported_fingerprints.push(existing_entry.fingerprint.clone());
            continue;
        }

        if let Some(agent) = discovered_agents
            .iter()
            .find(|entry| entry.fingerprint == fingerprint)
        {
            let managed_agent_id = discovery_id.replacen("discovery-", "managed-", 1);
            managed_agents.push(ManagedAgentDto {
                managed_agent_id,
                fingerprint: agent.fingerprint.clone(),
                alias: None,
                enabled: true,
                hidden: false,
                imported_at: agent.detected_at.clone(),
                source: "auto-imported".into(),
                provider: Some(agent.provider.clone()),
                root_path: Some(agent.root_path.clone()),
            });
            imported_fingerprints.push(agent.fingerprint.clone());
        }
    }

    managed_agents_store::save_managed_agents(&managed_agents)?;

    let resolved_agents = merge_resolved_agents(&discovered_agents, &managed_agents);
    let imported_agents = resolved_agents
        .iter()
        .filter(|agent| {
            imported_fingerprints
                .iter()
                .any(|fingerprint| fingerprint == &agent.fingerprint)
        })
        .cloned()
        .collect();

    Ok(ImportAgentsResultDto {
        imported_agents,
        resolved_agents,
    })
}

pub fn remove_managed_agent(
    managed_agent_id: String,
    scan_targets: Vec<ScanTargetDto>,
) -> Result<RemoveAgentResultDto, String> {
    let discovered_agents = discovered_agents_for_scan(scan_targets);
    let mut managed_agents = list_managed_agents();

    let removed_agent_id = merge_resolved_agents(&discovered_agents, &managed_agents)
        .into_iter()
        .find(|agent| agent.managed_agent_id.as_deref() == Some(managed_agent_id.as_str()))
        .map(|agent| agent.id)
        .unwrap_or_default();

    for entry in &mut managed_agents {
        if entry.managed_agent_id == managed_agent_id {
            entry.enabled = false;
            entry.hidden = true;
        }
    }

    managed_agents_store::save_managed_agents(&managed_agents)?;

    Ok(RemoveAgentResultDto {
        removed_agent_id,
        resolved_agents: merge_resolved_agents(&discovered_agents, &managed_agents),
    })
}

pub fn delete_agent(
    managed_agent_id: String,
    scan_targets: Vec<ScanTargetDto>,
) -> Result<DeleteAgentResultDto, String> {
    let discovered_agents = discovered_agents_for_scan(scan_targets);
    let managed_agents = list_managed_agents();

    let deleted_agent_id = merge_resolved_agents(&discovered_agents, &managed_agents)
        .into_iter()
        .find(|agent| agent.managed_agent_id.as_deref() == Some(managed_agent_id.as_str()))
        .map(|agent| agent.id)
        .unwrap_or_default();

    let next_managed_agents: Vec<_> = managed_agents
        .into_iter()
        .filter(|entry| entry.managed_agent_id != managed_agent_id)
        .collect();

    managed_agents_store::save_managed_agents(&next_managed_agents)?;

    Ok(DeleteAgentResultDto {
        deleted_agent_id,
        resolved_agents: merge_resolved_agents(&discovered_agents, &next_managed_agents),
    })
}

pub fn create_agent(draft: ManualAgentDraftDto) -> Result<CreateAgentResultDto, String> {
    let mut managed_agents = list_managed_agents();
    let discovered_agents = Vec::new();
    let id_suffix = draft
        .name
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch: char| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part: &&str| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let id_suffix = if id_suffix.is_empty() {
        "manual-agent".to_string()
    } else {
        id_suffix
    };
    let fingerprint = format!("{}-{}", draft.provider, id_suffix);
    let managed_agent_id = format!("managed-{}", id_suffix);

    managed_agents.retain(|entry| entry.fingerprint != fingerprint);
    managed_agents.push(ManagedAgentDto {
        managed_agent_id: managed_agent_id.clone(),
        fingerprint: fingerprint.clone(),
        alias: Some(draft.name.trim().into()),
        enabled: true,
        hidden: false,
        imported_at: "2026-03-25T10:30:00Z".into(),
        source: "manual-imported".into(),
        provider: Some(draft.provider.clone()),
        root_path: Some(draft.root_path.trim().into()),
    });

    managed_agents_store::save_managed_agents(&managed_agents)?;

    let resolved_agents = merge_resolved_agents(&discovered_agents, &managed_agents);
    let agent = resolved_agents
        .iter()
        .find(|entry| entry.managed_agent_id.as_deref() == Some(managed_agent_id.as_str()))
        .cloned()
        .ok_or_else(|| "Failed to create managed agent.".to_string())?;

    Ok(CreateAgentResultDto {
        agent,
        resolved_agents,
    })
}
