import { importDiscoveredAgent, refreshAgentDiscovery, setManagedAgentEnabled } from "./api";
import type { AgentConflict, AgentDiscoveryState, ManagedAgent, ResolvedAgentView } from "./types";

type UseAgentManagementParams = {
  setConflicts: React.Dispatch<React.SetStateAction<AgentConflict[]>>;
  setDiscoveryState: React.Dispatch<React.SetStateAction<AgentDiscoveryState>>;
  setManagedAgents: React.Dispatch<React.SetStateAction<ManagedAgent[]>>;
  setResolvedAgents: React.Dispatch<React.SetStateAction<ResolvedAgentView[]>>;
};

function toManagedAgentsSnapshot(agents: ResolvedAgentView[]): ManagedAgent[] {
  return agents
    .filter((agent) => agent.managed)
    .map((agent) => ({
      managedAgentId: agent.managedAgentId ?? `managed-${agent.id}`,
      fingerprint: agent.fingerprint,
      alias: agent.alias,
      enabled: agent.enabled,
      hidden: agent.hidden,
      importedAt: agent.lastScannedAt,
      source: agent.managedAgentId ? "manual-imported" : "auto-imported",
    }));
}

export function useAgentManagement({
  setConflicts,
  setDiscoveryState,
  setManagedAgents,
  setResolvedAgents,
}: UseAgentManagementParams) {
  const syncResolvedAgents = (agents: ResolvedAgentView[]) => {
    setResolvedAgents(agents);
    setManagedAgents(toManagedAgentsSnapshot(agents));
    setConflicts(
      agents
        .filter((agent) => agent.conflictIds.length > 0)
        .map((agent) => ({
          id: `${agent.id}-conflict`,
          type: "same-provider-multi-source",
          severity: agent.health === "error" ? "error" : "warning",
          summary: agent.summary,
          agentFingerprints: [agent.fingerprint],
        }))
    );
    setDiscoveryState((current) => ({
      ...current,
      initialized: true,
      scanning: false,
      refreshing: false,
      error: null,
      lastScannedAt: agents[0]?.lastScannedAt ?? current.lastScannedAt,
    }));
  };

  const handleRefresh = async () => {
    setDiscoveryState((current) => ({ ...current, refreshing: true, error: null }));

    try {
      const nextAgents = await refreshAgentDiscovery();
      syncResolvedAgents(nextAgents);
    } catch (error) {
      setDiscoveryState((current) => ({
        ...current,
        refreshing: false,
        error: error instanceof Error ? error.message : "Failed to refresh agent discovery.",
      }));
    }
  };

  const handleImport = async (discoveryId: string) => {
    const nextAgents = await importDiscoveredAgent(discoveryId);
    syncResolvedAgents(nextAgents);
  };

  const handleSetEnabled = async (agentId: string, enabled: boolean) => {
    const nextAgents = await setManagedAgentEnabled(agentId, enabled);
    syncResolvedAgents(nextAgents);
  };

  return {
    importAgent: handleImport,
    refreshAgents: handleRefresh,
    setAgentEnabled: handleSetEnabled,
  };
}
