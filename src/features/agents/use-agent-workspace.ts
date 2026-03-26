import { useMemo, useState } from "react";
import { marketplaceItems } from "@/features/marketplace/mock";
import {
  buildDiscoveryItems,
  createMarketplaceInstallStateMap,
  filterDiscoveryItems,
  sortDiscoveryItems,
} from "./discovery";
import { resourcesByKind } from "./resource-catalog";
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
  ResolvedAgentView,
} from "./types";

type WorkspaceMode = "browse" | "adding";

function buildManagedAgents(resolvedAgents: ResolvedAgentView[]) {
  return resolvedAgents
    .filter((agent) => agent.managed)
    .map((agent) => ({
      managedAgentId: agent.managedAgentId ?? `managed-${agent.id}`,
      fingerprint: agent.fingerprint,
      alias: agent.alias,
      enabled: agent.enabled,
      hidden: agent.hidden,
      importedAt: agent.lastScannedAt,
      source: "manual-imported" as const,
    }));
}

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

  const { discoveredAgents, discoveryState, managedAgents, resolvedAgents, setDiscoveryState, setManagedAgents, setResolvedAgents } =
    useAgentDiscovery();

  const { refreshAgents } = useAgentManagement({
    setDiscoveryState,
    setManagedAgents,
    setResolvedAgents,
  });

  const clearResourceSelection = () => {
    setSelectedResourceId("");
    setCheckedIds([]);
  };

  const setMode = (mode: WorkspaceMode) => {
    setWorkspaceMode(mode);
    clearResourceSelection();
  };

  const resetWorkspaceSelection = () => {
    setMode("adding");
  };

  const syncManagedState = (resolvedAgents: ResolvedAgentView[]) => {
    setResolvedAgents(resolvedAgents);
    setManagedAgents(buildManagedAgents(resolvedAgents));
    resetWorkspaceSelection();
  };

  const syncImportedAgents = ({ resolvedAgents }: ImportAgentsResult) => {
    syncManagedState(resolvedAgents);
  };

  const syncCreatedAgent = ({ resolvedAgents }: CreateAgentResult) => {
    syncManagedState(resolvedAgents);
  };

  const syncRemovedAgent = ({ removedAgentId, resolvedAgents }: RemoveAgentResult) => {
    syncManagedState(resolvedAgents);
    if (removedAgentId && selectedAgentId === removedAgentId) {
      setSelectedAgentId(resolvedAgents.find((entry) => entry.managed && !entry.hidden)?.id ?? "");
    }
  };

  const syncDeletedAgent = ({ deletedAgentId, resolvedAgents }: DeleteAgentResult) => {
    syncManagedState(resolvedAgents);
    if (deletedAgentId && selectedAgentId === deletedAgentId) {
      setSelectedAgentId(resolvedAgents.find((entry) => entry.managed && !entry.hidden)?.id ?? "");
    }
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
    clearResourceSelection();
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
    setSelectedAgentId(id);
    setMode("browse");
  };

  const enterAddingMode = () => {
    setMode("adding");
  };

  const exitAddingMode = () => {
    setMode("browse");
  };

  return {
    activeKind,
    checkedIds,
    clearChecked,
    discoveredAgents,
    discoveryState,
    filteredAgents: managedAgentsForRail,
    filteredResources,
    managedAgents,
    managedAgentsForView: resolvedAgents,
    onCreateAgentSuccess: syncCreatedAgent,
    onDeleteAgentSuccess: syncDeletedAgent,
    onImportAgentsSuccess: syncImportedAgents,
    onRemoveAgentSuccess: syncRemovedAgent,
    search,
    refreshAgents,
    selectKind,
    selectResource,
    selectedAgent,
    selectedAgentId: selectedAgent?.id ?? selectedAgentId,
    selectedResource,
    selectedResourceId: selectedResource?.id ?? selectedResourceId,
    setSearch,
    setSelectedAgentId: selectAgent,
    toggleChecked,
    updateMarketplaceInstallState,
    workspaceMode,
    enterAddingMode,
    exitAddingMode,
  };
}
