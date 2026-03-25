import { useEffect, useState } from "react";
import {
  listAgentConflicts,
  listDiscoveredAgents,
  listManagedAgents,
  listResolvedAgents,
} from "./api";
import type {
  AgentConflict,
  AgentDiscoveryState,
  DiscoveredAgent,
  ManagedAgent,
  ResolvedAgentView,
} from "./types";

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
  const [conflicts, setConflicts] = useState<AgentConflict[]>([]);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      setDiscoveryState((current) => ({ ...current, scanning: true, error: null }));

      try {
        const [nextDiscovered, nextManaged, nextResolved, nextConflicts] = await Promise.all([
          listDiscoveredAgents(),
          listManagedAgents(),
          listResolvedAgents(),
          listAgentConflicts(),
        ]);

        if (cancelled) {
          return;
        }

        setDiscoveredAgents(nextDiscovered);
        setManagedAgents(nextManaged);
        setResolvedAgents(nextResolved);
        setConflicts(nextConflicts);
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
    conflicts,
    discoveredAgents,
    discoveryState,
    managedAgents,
    resolvedAgents,
    setConflicts,
    setDiscoveryState,
    setManagedAgents,
    setResolvedAgents,
  };
}
