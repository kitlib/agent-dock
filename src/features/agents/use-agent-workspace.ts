import { useMemo, useState } from "react";
import { marketplaceItems } from "@/features/marketplace/mock";
import {
  buildDiscoveryItems,
  createMarketplaceInstallStateMap,
  filterDiscoveryItems,
  sortDiscoveryItems,
} from "./discovery";
import { resourcesByKind } from "./mock";
import { useAgentDiscovery } from "./use-agent-discovery";
import { useAgentManagement } from "./use-agent-management";
import type {
  AgentDiscoveryItem,
  CreateAgentResult,
  DeleteAgentResult,
  ImportAgentsResult,
  MarketplaceInstallStateLabel,
  RemoveAgentResult,
  ResourceKind,
} from "./types";

type WorkspaceMode = "browse" | "adding";

export function useAgentWorkspace() {
  const [workspaceMode, setWorkspaceMode] = useState<WorkspaceMode>("browse");
  const [search, setSearch] = useState("");
  const [selectedAgentId, setSelectedAgentId] = useState("");
  const [activeKind, setActiveKind] = useState<ResourceKind>("skill");
  const [selectedResourceId, setSelectedResourceId] = useState("");
  const [checkedIds, setCheckedIds] = useState<string[]>([]);
  const [marketplaceInstallStates, setMarketplaceInstallStates] = useState<
    Record<string, MarketplaceInstallStateLabel>
  >(() => createMarketplaceInstallStateMap(marketplaceItems));

  const {
    conflicts,
    discoveredAgents,
    discoveryState,
    managedAgents,
    resolvedAgents,
    setConflicts,
    setDiscoveryState,
    setManagedAgents,
    setResolvedAgents,
  } = useAgentDiscovery();

  const { importAgent, setAgentEnabled } = useAgentManagement({
    setConflicts,
    setDiscoveryState,
    setManagedAgents,
    setResolvedAgents,
  });

  const syncImportedAgents = ({ resolvedAgents }: ImportAgentsResult) => {
    setResolvedAgents(resolvedAgents);
    setManagedAgents(
      resolvedAgents
        .filter((agent) => agent.managed)
        .map((agent) => ({
          managedAgentId: agent.managedAgentId ?? `managed-${agent.id}`,
          fingerprint: agent.fingerprint,
          alias: agent.alias,
          enabled: agent.enabled,
          hidden: agent.hidden,
          importedAt: agent.lastScannedAt,
          source: "manual-imported" as const,
        }))
    );
    setSelectedResourceId("");
    setCheckedIds([]);
    setWorkspaceMode("adding");
  };

  const syncCreatedAgent = ({ agent, resolvedAgents }: CreateAgentResult) => {
    setResolvedAgents(resolvedAgents);
    setManagedAgents(
      resolvedAgents
        .filter((entry) => entry.managed)
        .map((entry) => ({
          managedAgentId: entry.managedAgentId ?? `managed-${entry.id}`,
          fingerprint: entry.fingerprint,
          alias: entry.alias,
          enabled: entry.enabled,
          hidden: entry.hidden,
          importedAt: entry.lastScannedAt,
          source: "manual-imported" as const,
        }))
    );
    setSelectedAgentId(agent.id);
    setSelectedResourceId("");
    setCheckedIds([]);
    setWorkspaceMode("browse");
  };

  const syncRemovedAgent = ({ removedAgentId, resolvedAgents }: RemoveAgentResult) => {
    setResolvedAgents(resolvedAgents);
    setManagedAgents(
      resolvedAgents
        .filter((entry) => entry.managed)
        .map((entry) => ({
          managedAgentId: entry.managedAgentId ?? `managed-${entry.id}`,
          fingerprint: entry.fingerprint,
          alias: entry.alias,
          enabled: entry.enabled,
          hidden: entry.hidden,
          importedAt: entry.lastScannedAt,
          source: "manual-imported" as const,
        }))
    );
    if (removedAgentId && selectedAgentId === removedAgentId) {
      setSelectedAgentId(resolvedAgents.find((entry) => entry.managed && !entry.hidden)?.id ?? "");
    }
    setSelectedResourceId("");
    setCheckedIds([]);
    setWorkspaceMode("adding");
  };

  const syncDeletedAgent = ({ deletedAgentId, resolvedAgents }: DeleteAgentResult) => {
    setResolvedAgents(resolvedAgents);
    setManagedAgents(
      resolvedAgents
        .filter((entry) => entry.managed)
        .map((entry) => ({
          managedAgentId: entry.managedAgentId ?? `managed-${entry.id}`,
          fingerprint: entry.fingerprint,
          alias: entry.alias,
          enabled: entry.enabled,
          hidden: entry.hidden,
          importedAt: entry.lastScannedAt,
          source: "manual-imported" as const,
        }))
    );
    if (deletedAgentId && selectedAgentId === deletedAgentId) {
      setSelectedAgentId(resolvedAgents.find((entry) => entry.managed && !entry.hidden)?.id ?? "");
    }
    setSelectedResourceId("");
    setCheckedIds([]);
    setWorkspaceMode("adding");
  };

  const normalizedSearch = search.trim().toLowerCase();

  const managedAgentsForRail = useMemo(() => {
    return resolvedAgents.filter((agent) => {
      if (!agent.managed) {
        return false;
      }

      const matchSearch =
        normalizedSearch.length === 0 ||
        agent.name.toLowerCase().includes(normalizedSearch) ||
        agent.role.toLowerCase().includes(normalizedSearch) ||
        agent.summary.toLowerCase().includes(normalizedSearch) ||
        agent.rootPath.toLowerCase().includes(normalizedSearch);

      return !agent.hidden && matchSearch;
    });
  }, [normalizedSearch, resolvedAgents]);

  const selectedAgent =
    managedAgentsForRail.find((agent) => agent.id === selectedAgentId) ??
    managedAgentsForRail[0] ??
    null;

  const discoveryItems = useMemo(() => {
    return buildDiscoveryItems(
      activeKind,
      resourcesByKind,
      marketplaceItems,
      marketplaceInstallStates,
      selectedAgent?.id ?? null,
      selectedAgent?.managed ?? false
    );
  }, [activeKind, marketplaceInstallStates, selectedAgent]);

  const filteredResources = useMemo(() => {
    return sortDiscoveryItems(
      filterDiscoveryItems(discoveryItems, normalizedSearch),
      normalizedSearch
    );
  }, [discoveryItems, normalizedSearch]);

  const selectedResource =
    filteredResources.find((resource) => resource.id === selectedResourceId) ??
    filteredResources[0] ??
    null;

  const toggleChecked = (id: string) => {
    const item = filteredResources.find((resource) => resource.id === id);
    if (!item || item.origin !== "local") return;

    setCheckedIds((current) =>
      current.includes(id) ? current.filter((entry) => entry !== id) : [...current, id]
    );
  };

  const clearChecked = () => setCheckedIds([]);

  const selectKind = (kind: ResourceKind) => {
    setActiveKind(kind);
    setSelectedResourceId("");
    setCheckedIds([]);
  };

  const selectResource = (resource: AgentDiscoveryItem | null) => {
    setSelectedResourceId(resource?.id ?? "");
  };

  const updateMarketplaceInstallState = (id: string) => {
    setMarketplaceInstallStates((current) => {
      const nextState =
        current[id] === "update" || current[id] === "available" ? "installed" : current[id];
      return { ...current, [id]: nextState };
    });
  };

  const selectAgent = (id: string) => {
    setWorkspaceMode("browse");
    setSelectedAgentId(id);
    setSelectedResourceId("");
    setCheckedIds([]);
  };

  const enterAddingMode = () => {
    setWorkspaceMode("adding");
    setSelectedResourceId("");
    setCheckedIds([]);
  };

  const exitAddingMode = () => {
    setWorkspaceMode("browse");
    setSelectedResourceId("");
    setCheckedIds([]);
  };

  return {
    activeKind,
    checkedIds,
    clearChecked,
    conflicts,
    discoveredAgents,
    discoveryState,
    filteredAgents: managedAgentsForRail,
    filteredResources,
    importAgent,
    managedAgents,
    managedAgentsForView: resolvedAgents,
    onCreateAgentSuccess: syncCreatedAgent,
    onDeleteAgentSuccess: syncDeletedAgent,
    onImportAgentsSuccess: syncImportedAgents,
    onRemoveAgentSuccess: syncRemovedAgent,
    search,
    selectKind,
    selectResource,
    selectedAgent,
    selectedAgentId: selectedAgent?.id ?? selectedAgentId,
    selectedResource,
    selectedResourceId: selectedResource?.id ?? selectedResourceId,
    setAgentEnabled,
    setSearch,
    setSelectedAgentId: selectAgent,
    toggleChecked,
    updateMarketplaceInstallState,
    workspaceMode,
    enterAddingMode,
    exitAddingMode,
  };
}
