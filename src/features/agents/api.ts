import { invoke } from "@tauri-apps/api/core";
import { checkLocalMarketplaceSkillUpdate } from "@/features/marketplace/api";
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
  McpResource,
  McpScanTarget,
  ImportLocalMcpResult,
  LocalMcpImportConflictStrategy,
  LocalSkillCopySource,
  LocalSkillCopyTargetAgent,
  PreviewLocalSkillCopyResult,
  LocalSkillConflictResolution,
  CopyLocalSkillsResult,
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

export async function listLocalMcps(scanTargets: McpScanTarget[]) {
  return invoke<McpResource[]>("list_local_mcps", { scanTargets });
}

export async function openMcpConfigFolder(configPath: string) {
  return invoke<void>("open_mcp_config_folder", { configPath });
}

export async function openMcpConfigFile(configPath: string) {
  return invoke<void>("open_mcp_config_file", { configPath });
}

export async function deleteLocalMcp(agentType: string, configPath: string, serverName: string) {
  return invoke<void>("delete_local_mcp", { agentType, configPath, serverName });
}

export async function importLocalMcpJson(
  agentType: string,
  rootPath: string,
  jsonPayload: string,
  conflictStrategy: LocalMcpImportConflictStrategy
) {
  return invoke<ImportLocalMcpResult>("import_local_mcp_json", {
    agentType,
    rootPath,
    jsonPayload,
    conflictStrategy,
  });
}

export async function setLocalSkillEnabled(
  skillPath: string,
  entryFilePath: string,
  enabled: boolean
) {
  return invoke<void>("set_local_skill_enabled", { skillPath, entryFilePath, enabled });
}

export async function openSkillFolder(skillPath: string) {
  return invoke<void>("open_skill_folder", { skillPath });
}

export async function openSkillEntryFile(skillPath: string, entryFilePath: string) {
  return invoke<void>("open_skill_entry_file", { skillPath, entryFilePath });
}

export async function deleteLocalSkill(skillPath: string, entryFilePath: string) {
  return invoke<void>("delete_local_skill", { skillPath, entryFilePath });
}

export async function deleteAgent(managedAgentId: string, scanTargets: ScanTarget[]) {
  return invoke<DeleteAgentResult>("delete_agent", {
    managedAgentId,
    scanTargets,
  });
}

export async function previewLocalSkillCopy(
  sources: LocalSkillCopySource[],
  targetAgent: LocalSkillCopyTargetAgent
) {
  return invoke<PreviewLocalSkillCopyResult>("preview_local_skill_copy", {
    sources,
    targetAgent,
  });
}

export async function copyLocalSkills(
  sources: LocalSkillCopySource[],
  targetAgent: LocalSkillCopyTargetAgent,
  resolutions: LocalSkillConflictResolution[]
) {
  return invoke<CopyLocalSkillsResult>("copy_local_skills", {
    sources,
    targetAgent,
    resolutions,
  });
}

export async function checkMarketplaceSkillUpdate(skillPath: string, entryFilePath: string) {
  return checkLocalMarketplaceSkillUpdate(skillPath, entryFilePath);
}
