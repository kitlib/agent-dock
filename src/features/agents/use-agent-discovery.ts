import { useEffect, useState } from "react";
import { agentTypeMeta } from "./agent-meta";
import { listManagedAgents, listResolvedAgents } from "./api";
import type { AgentDiscoveryState, DiscoveredAgent, ManagedAgent, ResolvedAgentView } from "./types";

const scanTargets = Object.values(agentTypeMeta).map((meta) => ({
  agentType: meta.agentType,
  name: meta.name,
  rootPath: meta.directory.replace(/\/$/, ""),
}));

export function useAgentDiscovery() {
  const [discoveryState, setDiscoveryState] = useState<AgentDiscoveryState>({
    initialized: false,
    scanning: true,
    refreshing: false,
    error: null,
    lastScannedAt: null,
  });
  const [discoveredAgents, setDiscoveredAgents] = useState<DiscoveredAgent[]>([]);
  const [managedAgents, setManagedAgents] = useState<ManagedAgent[]>([]);
  const [resolvedAgents, setResolvedAgents] = useState<ResolvedAgentView[]>([]);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      setDiscoveryState((current) => ({ ...current, scanning: true, error: null }));

      try {
        const [nextManaged, nextResolved] = await Promise.all([
          listManagedAgents(),
          listResolvedAgents(scanTargets),
        ]);
        const nextDiscovered = nextResolved.map((agent) => ({
          discoveryId: agent.discoveryId,
          fingerprint: agent.fingerprint,
          agentType: agent.agentType,
          displayName: agent.name,
          rootPath: agent.rootPath,
          status: agent.status,
          reason: undefined,
          resourceCounts: agent.resourceCounts,
          detectedAt: agent.lastScannedAt,
        }));
        if (cancelled) {
          return;
        }

        setDiscoveredAgents(nextDiscovered);
        setManagedAgents(nextManaged);
        setResolvedAgents(nextResolved);
        setDiscoveryState({
          initialized: true,
          scanning: false,
          refreshing: false,
          error: null,
          lastScannedAt: nextResolved[0]?.lastScannedAt ?? nextDiscovered[0]?.detectedAt ?? null,
        });
      } catch (error) {
        if (cancelled) {
          return;
        }

        setDiscoveryState({
          initialized: true,
          scanning: false,
          refreshing: false,
          error: error instanceof Error ? error.message : "Failed to load agent discovery.",
          lastScannedAt: null,
        });
      }
    };

    void load();

    return () => {
      cancelled = true;
    };
  }, []);

  return {
    discoveredAgents,
    discoveryState,
    managedAgents,
    resolvedAgents,
    setDiscoveryState,
    setManagedAgents,
    setResolvedAgents,
  };
}
