import { useEffect, useMemo, useState } from "react";
import { marketplaceItems } from "@/features/marketplace/mock";
import {
  buildDiscoveryItems,
  createMarketplaceInstallStateMap,
  filterDiscoveryItems,
  sortDiscoveryItems,
} from "@/features/resources/core/discovery";
import { resourcesByKind } from "@/features/resources/core/resource-catalog";
import type {
  AgentDiscoveryItem,
  MarketplaceInstallStateLabel,
  ResourceKind,
  ResolvedAgentView,
  SkillResource,
} from "@/features/agents/types";

function clearUnavailableCheckedIds(ids: string[], resources: AgentDiscoveryItem[]) {
  return ids.filter((id) => resources.some((resource) => resource.id === id));
}

export function useResourceBrowser(
  search: string,
  selectedAgent: ResolvedAgentView | null,
  skills: SkillResource[]
) {
  const [activeKind, setActiveKind] = useState<ResourceKind>("skill");
  const [selectedResourceId, setSelectedResourceId] = useState("");
  const [checkedIds, setCheckedIds] = useState<string[]>([]);
  const [marketplaceInstallStates, setMarketplaceInstallStates] = useState<
    Record<string, MarketplaceInstallStateLabel>
  >(() => createMarketplaceInstallStateMap(marketplaceItems));

  const normalizedSearch = search.trim().toLowerCase();

  const localResources = useMemo(
    () => ({
      ...resourcesByKind,
      skill: skills,
    }),
    [skills]
  );

  const discoveryItems = useMemo(() => {
    return buildDiscoveryItems(
      activeKind,
      localResources,
      marketplaceItems,
      marketplaceInstallStates,
      selectedAgent?.id ?? null,
      selectedAgent?.managed ?? false
    );
  }, [activeKind, localResources, marketplaceInstallStates, selectedAgent?.id, selectedAgent?.managed]);

  const filteredResources = useMemo(() => {
    const includeMarketplaceWhenEmpty = activeKind !== "skill" || normalizedSearch.length > 0;

    return sortDiscoveryItems(
      filterDiscoveryItems(discoveryItems, normalizedSearch, { includeMarketplaceWhenEmpty }),
      normalizedSearch
    );
  }, [activeKind, discoveryItems, normalizedSearch]);

  useEffect(() => {
    setCheckedIds((current) => clearUnavailableCheckedIds(current, filteredResources));

    if (filteredResources.some((resource) => resource.id === selectedResourceId)) {
      return;
    }

    setSelectedResourceId("");
  }, [filteredResources, selectedResourceId]);

  const selectedResourceBase =
    filteredResources.find((resource) => resource.id === selectedResourceId) ??
    filteredResources[0] ??
    null;

  const selectedResource = selectedResourceBase;

  const toggleChecked = (id: string) => {
    const item = filteredResources.find((resource) => resource.id === id);
    if (!item || item.origin !== "local") {
      return;
    }

    setCheckedIds((current) =>
      current.includes(id) ? current.filter((entry) => entry !== id) : [...current, id]
    );
  };

  const clearChecked = () => setCheckedIds([]);

  const toggleAllChecked = (ids: string[]) => {
    setCheckedIds(ids);
  };

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
    selectedResourceBase,
    selectedResourceId: selectedResource?.id ?? selectedResourceId,
    toggleChecked,
    toggleAllChecked,
    updateMarketplaceInstallState,
  };
}
