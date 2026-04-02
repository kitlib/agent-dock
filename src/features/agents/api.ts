import { invoke } from "@tauri-apps/api/core";
import type {
  CreateAgentResult,
  DeleteAgentResult,
  ImportAgentsResult,
  ManagedAgent,
  ManualAgentDraft,
  RemoveAgentResult,
  ResolvedAgentView,
  ScanTarget,
  ScannedAgentCandidate,
  SkillResource,
  SkillScanTarget,
} from "./types";

export async function listManagedAgents() {
  return invoke<ManagedAgent[]>("list_managed_agents");
}

export async function listResolvedAgents(scanTargets: ScanTarget[]) {
  return invoke<ResolvedAgentView[]>("list_resolved_agents", { scanTargets });
}

export async function scanAgents(scanTargets: ScanTarget[]) {
  return invoke<ScannedAgentCandidate[]>("scan_agents", { scanTargets });
}

export async function refreshAgentDiscovery(scanTargets: ScanTarget[]) {
  return invoke<ResolvedAgentView[]>("refresh_agent_discovery", { scanTargets });
}

export async function importAgents(candidateIds: string[], scanTargets: ScanTarget[]) {
  return invoke<ImportAgentsResult>("import_agents", {
    candidateIds,
    scanTargets,
  });
}

export async function createManualAgent(draft: ManualAgentDraft) {
  return invoke<CreateAgentResult>("create_agent", {
    draft,
  });
}

export async function removeManagedAgent(managedAgentId: string, scanTargets: ScanTarget[]) {
  return invoke<RemoveAgentResult>("remove_managed_agent", {
    managedAgentId,
    scanTargets,
  });
}

export async function listLocalSkills(scanTargets: SkillScanTarget[]) {
  console.log("[skills] list_local_skills request", {
    scanTargets: scanTargets.map((target) => ({
      agentId: target.agentId,
      agentType: target.agentType,
      rootPath: target.rootPath,
      displayName: target.displayName,
      source: target.source,
    })),
  });

  const skills = await invoke<SkillResource[]>("list_local_skills", { scanTargets });

  console.log("[skills] list_local_skills response", {
    count: skills.length,
    skills: skills.map((skill) => ({
      id: skill.id,
      name: skill.name,
      ownerAgentId: skill.ownerAgentId ?? null,
      agentType: skill.agentType ?? null,
      agentName: skill.agentName ?? null,
      skillPath: skill.skillPath ?? null,
    })),
  });

  return skills;
}

export async function getLocalSkillDetail(scanTargets: SkillScanTarget[], skillId: string) {
  console.log("[skills] get_local_skill_detail request", {
    skillId,
    scanTargets: scanTargets.map((target) => ({
      agentId: target.agentId,
      agentType: target.agentType,
      rootPath: target.rootPath,
      displayName: target.displayName,
      source: target.source,
    })),
  });

  const detail = await invoke<SkillResource>("get_local_skill_detail", { scanTargets, skillId });

  console.log("[skills] get_local_skill_detail response", {
    skillId: detail.id,
    ownerAgentId: detail.ownerAgentId ?? null,
    agentType: detail.agentType ?? null,
    agentName: detail.agentName ?? null,
    skillPath: detail.skillPath ?? null,
  });

  return detail;
}

export async function openSkillFolder(skillPath: string) {
  return invoke<void>("open_skill_folder", { skillPath });
}

export async function deleteAgent(managedAgentId: string, scanTargets: ScanTarget[]) {
  return invoke<DeleteAgentResult>("delete_agent", {
    managedAgentId,
    scanTargets,
  });
}
