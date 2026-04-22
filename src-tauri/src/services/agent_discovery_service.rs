use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::dto::agents::{
    AgentResourceCountsDto, CreateAgentResultDto, DeleteAgentResultDto, DiscoveredAgentDto,
    ImportAgentsResultDto, ManagedAgentDto, ManualAgentDraftDto, RemoveAgentResultDto,
    ResolvedAgentDto, ScanTargetDto, ScannedAgentCandidateDto,
};
use crate::repositories::agent_repository::{AgentId, AgentRepository, ManagedAgent};
use crate::scanners::agent_type_scanner;
use crate::services::ServiceError;

#[derive(Clone)]
pub struct AgentDiscoveryService {
    agent_repo: Arc<dyn AgentRepository>,
}

impl AgentDiscoveryService {
    pub fn new(agent_repo: Arc<dyn AgentRepository>) -> Self {
        Self { agent_repo }
    }

    pub fn list_managed_agents(&self) -> Vec<ManagedAgentDto> {
        self.load_managed_agents().iter().map(to_dto).collect()
    }

    pub fn list_resolved_agents(&self, scan_targets: Vec<ScanTargetDto>) -> Vec<ResolvedAgentDto> {
        let discovered_agents = self.discovered_agents_for_scan(scan_targets);
        let managed_agents = self.load_managed_agents();
        merge_resolved_agents(&discovered_agents, &managed_agents)
    }

    pub fn scan_agents(&self, scan_targets: Vec<ScanTargetDto>) -> Vec<ScannedAgentCandidateDto> {
        let discovered_agents = self.discovered_agents_for_scan(scan_targets);
        let managed_agents = self.load_managed_agents();
        let resolved_agents = merge_resolved_agents(&discovered_agents, &managed_agents);
        scanned_candidates(&discovered_agents, &resolved_agents)
    }

    pub fn refresh_agent_discovery(&self, scan_targets: Vec<ScanTargetDto>) -> Vec<ResolvedAgentDto> {
        let discovered_agents = self.discovered_agents_for_scan(scan_targets);
        let managed_agents = self.load_managed_agents();
        merge_resolved_agents(&discovered_agents, &managed_agents)
    }

    pub fn import_agents(
        &self,
        candidate_ids: Vec<String>,
        scan_targets: Vec<ScanTargetDto>,
    ) -> Result<ImportAgentsResultDto, ServiceError> {
        let discovered_agents = self.discovered_agents_for_scan(scan_targets);
        let mut managed_agents = self.load_managed_agents();

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
                        .find(|entry| entry.id.0.replacen("managed-", "discovery-", 1) == discovery_id)
                        .map(|entry| entry.fingerprint.clone())
                });

            let Some(fingerprint) = candidate_fingerprint else {
                continue;
            };

            if let Some(existing_entry) = managed_agents
                .iter_mut()
                .find(|entry| entry.fingerprint == fingerprint)
            {
                existing_entry.enable();
                imported_fingerprints.push(existing_entry.fingerprint.clone());
                continue;
            }

            if let Some(agent) = discovered_agents
                .iter()
                .find(|entry| entry.fingerprint == fingerprint)
            {
                let managed_agent_id = discovery_id.replacen("discovery-", "managed-", 1);
                managed_agents.push(ManagedAgent {
                    id: AgentId::new(managed_agent_id),
                    fingerprint: agent.fingerprint.clone(),
                    alias: None,
                    enabled: true,
                    hidden: false,
                    imported_at: agent.detected_at.clone(),
                    source: "auto-imported".into(),
                    agent_type: Some(agent.agent_type.clone()),
                    root_path: Some(PathBuf::from(&agent.root_path)),
                });
                imported_fingerprints.push(agent.fingerprint.clone());
            }
        }

        self.save_managed_agents(&managed_agents)?;

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
        &self,
        managed_agent_id: String,
        scan_targets: Vec<ScanTargetDto>,
    ) -> Result<RemoveAgentResultDto, ServiceError> {
        let discovered_agents = self.discovered_agents_for_scan(scan_targets);
        let managed_agents = self.load_managed_agents();

        let removed_agent_id = merge_resolved_agents(&discovered_agents, &managed_agents)
            .into_iter()
            .find(|agent| agent.managed_agent_id.as_deref() == Some(managed_agent_id.as_str()))
            .map(|agent| agent.id)
            .unwrap_or_default();

        let next_managed_agents: Vec<_> = managed_agents
            .into_iter()
            .filter(|entry| entry.id.as_str() != managed_agent_id)
            .collect();

        self.save_managed_agents(&next_managed_agents)?;

        Ok(RemoveAgentResultDto {
            removed_agent_id,
            resolved_agents: merge_resolved_agents(&discovered_agents, &next_managed_agents),
        })
    }

    pub fn delete_agent(
        &self,
        managed_agent_id: String,
        scan_targets: Vec<ScanTargetDto>,
    ) -> Result<DeleteAgentResultDto, ServiceError> {
        let discovered_agents = self.discovered_agents_for_scan(scan_targets);
        let managed_agents = self.load_managed_agents();

        let deleted_agent_id = merge_resolved_agents(&discovered_agents, &managed_agents)
            .into_iter()
            .find(|agent| agent.managed_agent_id.as_deref() == Some(managed_agent_id.as_str()))
            .map(|agent| agent.id)
            .unwrap_or_default();

        let next_managed_agents: Vec<_> = managed_agents
            .into_iter()
            .filter(|entry| entry.id.as_str() != managed_agent_id)
            .collect();

        self.save_managed_agents(&next_managed_agents)?;

        Ok(DeleteAgentResultDto {
            deleted_agent_id,
            resolved_agents: merge_resolved_agents(&discovered_agents, &next_managed_agents),
        })
    }

    pub fn create_agent(&self, draft: ManualAgentDraftDto) -> Result<CreateAgentResultDto, ServiceError> {
        let mut managed_agents = self.load_managed_agents();
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
        let fingerprint = format!("{}-{}", draft.agent_type, id_suffix);
        let managed_agent_id = format!("managed-{}", id_suffix);

        managed_agents.retain(|entry| entry.fingerprint != fingerprint);
        managed_agents.push(ManagedAgent {
            id: AgentId::new(managed_agent_id.clone()),
            fingerprint: fingerprint.clone(),
            alias: Some(draft.name.trim().into()),
            enabled: true,
            hidden: false,
            imported_at: "2026-03-25T10:30:00Z".into(),
            source: "manual-imported".into(),
            agent_type: Some(draft.agent_type.clone()),
            root_path: Some(PathBuf::from(draft.root_path.trim())),
        });

        self.save_managed_agents(&managed_agents)?;

        let resolved_agents = merge_resolved_agents(&discovered_agents, &managed_agents);
        let agent = resolved_agents
            .iter()
            .find(|entry| entry.managed_agent_id.as_deref() == Some(managed_agent_id.as_str()))
            .cloned()
            .ok_or_else(|| ServiceError::Internal("Failed to create managed agent.".to_string()))?;

        Ok(CreateAgentResultDto {
            agent,
            resolved_agents,
        })
    }

    fn discovered_agents_for_scan(&self, scan_targets: Vec<ScanTargetDto>) -> Vec<DiscoveredAgentDto> {
        agent_type_scanner::scan_discovered_agents(scan_targets)
    }

    fn load_managed_agents(&self) -> Vec<ManagedAgent> {
        self.agent_repo.find_all().unwrap_or_default()
    }

    fn save_managed_agents(&self, agents: &[ManagedAgent]) -> Result<(), ServiceError> {
        self.agent_repo.save_all(agents).map_err(ServiceError::from)
    }
}

// Convert domain ManagedAgent to DTO
fn to_dto(agent: &ManagedAgent) -> ManagedAgentDto {
    ManagedAgentDto {
        managed_agent_id: agent.id.as_str().to_string(),
        fingerprint: agent.fingerprint.clone(),
        agent_type: agent.agent_type.clone(),
        alias: agent.alias.clone(),
        enabled: agent.enabled,
        hidden: agent.hidden,
        imported_at: agent.imported_at.clone(),
        source: agent.source.clone(),
        root_path: agent
            .root_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
    }
}

fn role_for_agent_type(agent_type: &str) -> String {
    match agent_type {
        "cursor" => "AI coding assistant",
        "claude" => "CLI coding assistant",
        "codex" => "Terminal coding assistant",
        "antigravity" => "Workflow automation assistant",
        _ => "Managed agent",
    }
    .into()
}

fn discovered_summary(agent_type: &str) -> String {
    match agent_type {
        "cursor" => "Detected Cursor in the user directory and ready to import into AgentDock.",
        "claude" => {
            "Detected Claude Code in the user directory and ready to import into AgentDock."
        }
        "codex" => "Detected Codex CLI in the user directory and ready to import into AgentDock.",
        "antigravity" => {
            "Detected Antigravity in the user directory and ready to import into AgentDock."
        }
        _ => "Detected in the user directory and ready to import into AgentDock.",
    }
    .into()
}

fn manual_resolved_agent(agent: &ManagedAgent) -> ResolvedAgentDto {
    let id_suffix = agent
        .id
        .as_str()
        .strip_prefix("managed-")
        .unwrap_or(agent.id.as_str());
    let agent_type = agent.agent_type.as_deref().unwrap_or_else(|| {
        agent
            .fingerprint
            .split_once('-')
            .map(|(agent_type, _)| agent_type)
            .unwrap_or("claude")
    });
    let name = agent.alias.clone().unwrap_or_else(|| {
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
    let is_managed = !agent.hidden;

    ResolvedAgentDto {
        id: format!("agent-{}", id_suffix),
        discovery_id: format!("discovery-{}", id_suffix),
        fingerprint: agent.fingerprint.clone(),
        agent_type: agent_type.into(),
        name,
        alias: agent.alias.clone(),
        role: "Manually managed agent".into(),
        root_path: agent
            .root_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".into()),
        managed: is_managed,
        managed_agent_id: Some(agent.id.as_str().to_string()),
        enabled: if is_managed { agent.enabled } else { false },
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
            command: 0,
            mcp: 0,
            subagent: 0,
        },
        last_scanned_at: agent.imported_at.clone(),
    }
}

fn merge_resolved_agents(
    discovered_agents: &[DiscoveredAgentDto],
    managed_agents: &[ManagedAgent],
) -> Vec<ResolvedAgentDto> {
    let discovered_fingerprints: HashSet<_> = discovered_agents
        .iter()
        .map(|agent| agent.fingerprint.as_str())
        .collect();
    let mut resolved_agents: Vec<_> = discovered_agents
        .iter()
        .map(|agent| {
            let managed = managed_agents.iter().find(|entry| {
                entry.matches(
                    &agent.fingerprint,
                    &agent.agent_type,
                    Path::new(&agent.root_path),
                )
            });

            ResolvedAgentDto {
                id: agent.discovery_id.replacen("discovery-", "agent-", 1),
                discovery_id: agent.discovery_id.clone(),
                fingerprint: agent.fingerprint.clone(),
                agent_type: agent.agent_type.clone(),
                name: agent.display_name.clone(),
                alias: managed.and_then(|entry| entry.alias.clone()),
                role: role_for_agent_type(&agent.agent_type),
                root_path: agent.root_path.clone(),
                managed: managed.map(|entry| !entry.hidden).unwrap_or(false),
                managed_agent_id: managed.map(|entry| entry.id.as_str().to_string()),
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
                    discovered_summary(&agent.agent_type)
                },
                group_id: "assistant".into(),
                resource_counts: agent.resource_counts.clone(),
                last_scanned_at: agent.detected_at.clone(),
            }
        })
        .collect();

    for entry in managed_agents {
        let already_resolved = discovered_agents.iter().any(|agent| {
            entry.matches(
                &agent.fingerprint,
                &agent.agent_type,
                Path::new(&agent.root_path),
            )
        });
        if discovered_fingerprints.contains(entry.fingerprint.as_str()) || already_resolved {
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
                agent_type: agent.agent_type.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::agents::{DiscoveredAgentDto, AgentResourceCountsDto};

    fn mock_discovered_agent(id: &str, fingerprint: &str) -> DiscoveredAgentDto {
        DiscoveredAgentDto {
            discovery_id: format!("discovery-{}", id),
            fingerprint: fingerprint.to_string(),
            agent_type: "claude".into(),
            display_name: format!("Agent {}", id),
            root_path: format!("/path/to/{}", id),
            status: "discovered".into(),
            resource_counts: AgentResourceCountsDto {
                skill: 0,
                command: 0,
                mcp: 0,
                subagent: 0,
            },
            detected_at: "2026-04-21T00:00:00Z".into(),
            reason: None,
        }
    }

    #[test]
    fn test_merge_resolved_agents_basic() {
        let discovered = vec![
            mock_discovered_agent("1", "fp-1"),
            mock_discovered_agent("2", "fp-2"),
        ];

        // No managed agents
        let managed = vec![];

        let resolved = merge_resolved_agents(&discovered, &managed);

        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].fingerprint, "fp-1");
        assert_eq!(resolved[0].managed, false);
        assert_eq!(resolved[1].fingerprint, "fp-2");
    }

    #[test]
    fn test_merge_resolved_agents_with_managed() {
        let discovered = vec![
            mock_discovered_agent("1", "fp-1"),
            mock_discovered_agent("2", "fp-2"),
        ];

        let managed = vec![
            ManagedAgent {
                id: AgentId::new("managed-1".into()),
                fingerprint: "fp-1".into(),
                alias: Some("My Special Agent".into()),
                enabled: true,
                hidden: false,
                imported_at: "2026-04-21T00:00:00Z".into(),
                source: "test".into(),
                agent_type: Some("claude".into()),
                root_path: Some(PathBuf::from("/path/to/1")),
            }
        ];

        let resolved = merge_resolved_agents(&discovered, &managed);

        assert_eq!(resolved.len(), 2);

        let r1 = resolved.iter().find(|r| r.fingerprint == "fp-1").unwrap();
        assert_eq!(r1.managed, true);
        assert_eq!(r1.alias.as_deref(), Some("My Special Agent"));
        assert_eq!(r1.enabled, true);

        let r2 = resolved.iter().find(|r| r.fingerprint == "fp-2").unwrap();
        assert_eq!(r2.managed, false);
    }
}
