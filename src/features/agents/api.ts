import { invoke } from "@tauri-apps/api/core";
import { checkLocalMarketplaceSkillUpdate } from "@/features/marketplace/api";
import type {
  CopyLocalSkillsResult,
  CreateAgentResult,
  DeleteAgentResult,
  EditableLocalMcp,
  ImportAgentsResult,
  ImportLocalMcpResult,
  LocalMcpImportConflictStrategy,
  LocalSkillConflictResolution,
  LocalSkillCopySource,
  LocalSkillCopyTargetAgent,
  ManagedAgent,
  ManualAgentDraft,
  McpResource,
  McpScanTarget,
  McpServerCapabilities,
  PreviewLocalSkillCopyResult,
  RemoveAgentResult,
  ResolvedAgentView,
  ScanTarget,
  ScannedAgentCandidate,
  SkillResource,
  SkillScanTarget,
  UpdateLocalMcpInput,
  UpdateLocalMcpResult,
} from "./types";

function formatSkillScanTargets(scanTargets: SkillScanTarget[]) {
  return scanTargets.map((target) => ({
    agentId: target.agentId,
    agentType: target.agentType,
    rootPath: target.rootPath,
    displayName: target.displayName,
    source: target.source,
  }));
}

function formatSkillLog(skill: SkillResource) {
  return {
    id: skill.id,
    name: skill.name,
    ownerAgentId: skill.ownerAgentId ?? null,
    agentType: skill.agentType ?? null,
    agentName: skill.agentName ?? null,
    skillPath: skill.skillPath ?? null,
  };
}

type UpdateLocalMcpPayload = {
  nextServerName: string;
  transport: UpdateLocalMcpInput["transport"];
  command: UpdateLocalMcpInput["command"];
  args: UpdateLocalMcpInput["args"];
  env: UpdateLocalMcpInput["env"];
  url: UpdateLocalMcpInput["url"];
  headers: UpdateLocalMcpInput["headers"];
};

function buildLocalMcpPayload(nextServer: UpdateLocalMcpInput): UpdateLocalMcpPayload {
  return {
    nextServerName: nextServer.serverName,
    transport: nextServer.transport,
    command: nextServer.command,
    args: nextServer.args,
    env: nextServer.env,
    url: nextServer.url,
    headers: nextServer.headers,
  };
}

function formatMcpScanTargets(scanTargets: McpScanTarget[]) {
  return scanTargets.map((target) => ({
    agentId: target.agentId,
    rootPath: target.rootPath,
  }));
}

function formatMcpLog(mcp: McpResource) {
  return {
    id: mcp.id,
    name: mcp.name,
    ownerAgentId: mcp.ownerAgentId ?? null,
    transport: mcp.transport,
    command: mcp.command ? mcp.command.slice(0, 50) + (mcp.command.length > 50 ? "..." : "") : null,
    url: mcp.url ?? null,
  };
}

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
    scanTargets: formatSkillScanTargets(scanTargets),
  });

  const skills = await invoke<SkillResource[]>("list_local_skills", { scanTargets });

  console.log("[skills] list_local_skills response", {
    count: skills.length,
    skills: skills.map(formatSkillLog),
  });

  return skills;
}

export async function getLocalSkillDetail(scanTargets: SkillScanTarget[], skillId: string) {
  console.log("[skills] get_local_skill_detail request", {
    skillId,
    scanTargets: formatSkillScanTargets(scanTargets),
  });

  const detail = await invoke<SkillResource>("get_local_skill_detail", { scanTargets, skillId });

  console.log("[skills] get_local_skill_detail response", formatSkillLog(detail));

  return detail;
}

export async function listLocalMcps(scanTargets: McpScanTarget[]) {
  console.log("[mcp] list_local_mcps request", {
    scanTargets: formatMcpScanTargets(scanTargets),
  });

  try {
    const mcps = await invoke<McpResource[]>("list_local_mcps", { scanTargets });

    console.log("[mcp] list_local_mcps response", {
      count: mcps.length,
      mcps: mcps.map(formatMcpLog),
    });

    return mcps;
  } catch (error) {
    console.error("[mcp] list_local_mcps failed", {
      scanTargets: formatMcpScanTargets(scanTargets),
      error:
        error instanceof Error
          ? {
              message: error.message,
              stack: error.stack,
              name: error.name,
            }
          : error,
    });
    throw error;
  }
}

export async function openMcpConfigFolder(configPath: string) {
  return invoke<void>("open_mcp_config_folder", { configPath });
}

export async function openMcpConfigFile(configPath: string) {
  return invoke<void>("open_mcp_config_file", { configPath });
}

export async function getLocalMcpEditData(
  agentType: string,
  configPath: string,
  serverName: string,
  scope: string,
  projectPath?: string | null
) {
  return invoke<EditableLocalMcp>("get_local_mcp_edit_data", {
    agentType,
    configPath,
    serverName,
    scope,
    projectPath,
  });
}

export async function updateLocalMcp(
  agentType: string,
  configPath: string,
  serverName: string,
  scope: string,
  nextServer: UpdateLocalMcpInput,
  projectPath?: string | null
) {
  return invoke<UpdateLocalMcpResult>("update_local_mcp", {
    agentType,
    configPath,
    serverName,
    scope,
    projectPath,
    ...buildLocalMcpPayload(nextServer),
  });
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

export async function inspectMcpServer(config: EditableLocalMcp) {
  console.log("[MCP] Inspect server request:", {
    name: config.serverName,
    transport: config.transport,
    command: config.command ? `${config.command} ${config.args?.join(" ") || ""}` : null,
    url: config.url,
    hasEnv: config.env && Object.keys(config.env).length > 0,
    hasHeaders: config.headers && Object.keys(config.headers).length > 0,
  });

  try {
    const result = await invoke<McpServerCapabilities>("inspect_mcp_server", { config });
    console.log("[MCP] Inspect server response:", {
      name: config.serverName,
      tools: result.tools?.length || 0,
      toolNames: result.tools?.map((t) => t.name) || [],
    });
    return result;
  } catch (error) {
    console.error("[MCP] Inspect server failed:", {
      name: config.serverName,
      error:
        error instanceof Error
          ? {
              message: error.message,
              stack: error.stack,
              name: error.name,
            }
          : error,
      config,
    });
    throw error;
  }
}

export async function stopMcpInspector() {
  console.log("[MCP] Stop inspector request");

  try {
    await invoke<void>("stop_mcp_inspector");
    console.log("[MCP] Stop inspector success");
  } catch (error) {
    console.error("[MCP] Stop inspector failed:", {
      error:
        error instanceof Error
          ? {
              message: error.message,
              stack: error.stack,
              name: error.name,
            }
          : error,
    });
    throw error;
  }
}

export async function callMcpTool(
  config: EditableLocalMcp,
  toolName: string,
  parameters: Record<string, unknown>
) {
  return invoke<unknown>("call_mcp_tool", {
    request: {
      config,
      toolName,
      parameters,
    },
  });
}
