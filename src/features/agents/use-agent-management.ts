import { agentMeta } from "./agent-meta";
import { refreshAgentDiscovery } from "./api";
import type { AgentDiscoveryState, ManagedAgent, ResolvedAgentView } from "./types";

type UseAgentManagementParams = {
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

const scanTargets = Object.values(agentMeta).map((meta) => ({
  agent: meta.id,
  name: meta.name,
  rootPath: meta.directory.replace(/\/$/, ""),
}));

export function useAgentManagement({
  setDiscoveryState,
  setManagedAgents,
  setResolvedAgents,
}: UseAgentManagementParams) {
  const syncResolvedAgents = (agents: ResolvedAgentView[]) => {
    setResolvedAgents(agents);
    setManagedAgents(toManagedAgentsSnapshot(agents));
    setDiscoveryState((current: AgentDiscoveryState) => ({
      ...current,
      initialized: true,
      scanning: false,
      refreshing: false,
      error: null,
      lastScannedAt: agents[0]?.lastScannedAt ?? current.lastScannedAt,
    }));
  };

  const handleRefresh = async () => {
    setDiscoveryState((current: AgentDiscoveryState) => ({ ...current, refreshing: true, error: null }));

    try {
      const nextAgents = await refreshAgentDiscovery(scanTargets);
      syncResolvedAgents(nextAgents);
    } catch (error) {
      setDiscoveryState((current: AgentDiscoveryState) => ({
        ...current,
        refreshing: false,
        error: error instanceof Error ? error.message : "Failed to refresh agent discovery.",
      }));
    }
  };

  return {
    refreshAgents: handleRefresh,
  };
}
