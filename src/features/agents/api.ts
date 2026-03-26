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

export async function deleteAgent(managedAgentId: string, scanTargets: ScanTarget[]) {
  return invoke<DeleteAgentResult>("delete_agent", {
    managedAgentId,
    scanTargets,
  });
}
