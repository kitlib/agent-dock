import { invoke } from "@tauri-apps/api/core";
import { agentConflicts, discoveredAgents, managedAgents, resolvedAgents } from "./mock";
import {
  applyImportToResolvedAgents,
  buildScanCandidates,
  createManualResolvedAgent,
  deleteAgentFromResolvedAgents,
  removeManagedAgentFromResolvedAgents,
} from "./import-utils";
import type {
  AgentConflict,
  AgentResourceView,
  CreateAgentResult,
  DeleteAgentResult,
  DiscoveredAgent,
  ImportAgentsResult,
  ManagedAgent,
  ManualAgentDraft,
  RemoveAgentResult,
  ResolvedAgentView,
  ScannedAgentCandidate,
} from "./types";

const hasTauriRuntime = () =>
  typeof window !== "undefined" && typeof window.__TAURI_INTERNALS__ !== "undefined";

async function invokeOrFallback<T>(command: string, fallback: T, args?: Record<string, unknown>) {
  if (!hasTauriRuntime()) {
    return fallback;
  }

  return invoke<T>(command, args);
}

export async function listDiscoveredAgents() {
  return invokeOrFallback<DiscoveredAgent[]>("list_discovered_agents", discoveredAgents);
}

export async function listManagedAgents() {
  return invokeOrFallback<ManagedAgent[]>("list_managed_agents", managedAgents);
}

export async function listResolvedAgents() {
  return invokeOrFallback<ResolvedAgentView[]>("list_resolved_agents", resolvedAgents);
}

export async function listAgentConflicts() {
  return invokeOrFallback<AgentConflict[]>("list_agent_conflicts", agentConflicts);
}

export async function scanAgents() {
  return invokeOrFallback<ScannedAgentCandidate[]>(
    "scan_agents",
    buildScanCandidates(resolvedAgents)
  );
}

export async function refreshAgentDiscovery() {
  return invokeOrFallback<ResolvedAgentView[]>("refresh_agent_discovery", resolvedAgents);
}

export async function importAgents(candidateIds: string[]) {
  return invokeOrFallback<ImportAgentsResult>(
    "import_agents",
    applyImportToResolvedAgents(resolvedAgents, candidateIds),
    {
      candidateIds,
    }
  );
}

export async function createManualAgent(draft: ManualAgentDraft) {
  return invokeOrFallback<CreateAgentResult>(
    "create_agent",
    createManualResolvedAgent(resolvedAgents, draft),
    {
      draft,
    }
  );
}

export async function removeManagedAgent(managedAgentId: string) {
  return invokeOrFallback<RemoveAgentResult>(
    "remove_managed_agent",
    removeManagedAgentFromResolvedAgents(resolvedAgents, managedAgentId),
    {
      managedAgentId,
    }
  );
}

export async function deleteAgent(managedAgentId: string) {
  return invokeOrFallback<DeleteAgentResult>(
    "delete_agent",
    deleteAgentFromResolvedAgents(resolvedAgents, managedAgentId),
    {
      managedAgentId,
    }
  );
}

export async function importDiscoveredAgent(discoveryId: string) {
  return invokeOrFallback<ResolvedAgentView[]>("import_discovered_agent", resolvedAgents, {
    discoveryId,
  });
}

export async function setManagedAgentEnabled(agentId: string, enabled: boolean) {
  return invokeOrFallback<ResolvedAgentView[]>("set_managed_agent_enabled", resolvedAgents, {
    agentId,
    enabled,
  });
}

export async function listAgentResources() {
  return invokeOrFallback<AgentResourceView[]>("list_agent_resources", []);
}
