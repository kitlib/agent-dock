import { useMemo, useState } from "react";
import { marketplaceItems } from "@/features/marketplace/mock";
import {
  buildDiscoveryItems,
  createMarketplaceInstallStateMap,
  filterDiscoveryItems,
  sortDiscoveryItems,
} from "./discovery";
import { resourcesByKind } from "./resource-catalog";
import type { AgentDiscoveryItem, MarketplaceInstallStateLabel, ResourceKind } from "./types";

export function useResourceDiscovery(search: string) {
  const [activeKind, setActiveKind] = useState<ResourceKind>("skill");
  const [selectedResourceId, setSelectedResourceId] = useState("");
  const [checkedIds, setCheckedIds] = useState<string[]>([]);
  const [marketplaceInstallStates, setMarketplaceInstallStates] = useState<
    Record<string, MarketplaceInstallStateLabel>
  >(() => createMarketplaceInstallStateMap(marketplaceItems));

  const normalizedSearch = search.trim().toLowerCase();

  const discoveryItems = useMemo(() => {
    return buildDiscoveryItems(
      activeKind,
      resourcesByKind,
      marketplaceItems,
      marketplaceInstallStates,
      null,
      false
    );
  }, [activeKind, marketplaceInstallStates]);

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

  return {
    activeKind,
    checkedIds,
    clearChecked,
    filteredResources,
    selectKind,
    selectResource,
    selectedResource,
    selectedResourceId,
    toggleChecked,
    updateMarketplaceInstallState,
  };
}
