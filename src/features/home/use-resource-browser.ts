import { useEffect, useMemo, useState } from "react";
import { marketplaceItems as mockMarketplaceItems } from "@/features/marketplace/mock";
import { useSkillsshMarketplaceQuery } from "@/features/marketplace/queries";
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
  >({});

  const normalizedSearch = search.trim().toLowerCase();
  const skillMarketplaceQuery = useSkillsshMarketplaceQuery(
    normalizedSearch,
    activeKind === "skill"
  );
  const effectiveMarketplaceItems = useMemo(() => {
    const nonSkillItems = mockMarketplaceItems.filter((item) => item.kind !== "skill");
    const skillItems =
      activeKind === "skill"
        ? (skillMarketplaceQuery.data ?? [])
        : mockMarketplaceItems.filter((item) => item.kind === "skill");

    return [...skillItems, ...nonSkillItems];
  }, [activeKind, skillMarketplaceQuery.data]);

  useEffect(() => {
    const defaultStates = createMarketplaceInstallStateMap(effectiveMarketplaceItems);
    setMarketplaceInstallStates(defaultStates);
  }, [effectiveMarketplaceItems, skills]);

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
      effectiveMarketplaceItems,
      marketplaceInstallStates,
      selectedAgent?.id ?? null,
      selectedAgent?.managed ?? false
    );
  }, [
    activeKind,
    localResources,
    marketplaceInstallStates,
    selectedAgent?.id,
    selectedAgent?.managed,
    effectiveMarketplaceItems,
  ]);

  const filteredResources = useMemo(() => {
    const includeMarketplaceWhenEmpty = true;

    return sortDiscoveryItems(
      filterDiscoveryItems(discoveryItems, normalizedSearch, { includeMarketplaceWhenEmpty }),
      normalizedSearch
    );
  }, [discoveryItems, normalizedSearch]);

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
      const currentState = current[id] ?? "available";
      const nextState =
        currentState === "update" || currentState === "available" ? "installed" : currentState;
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
    isMarketplaceLoading: skillMarketplaceQuery.isFetching,
    marketplaceError:
      skillMarketplaceQuery.error instanceof Error ? skillMarketplaceQuery.error.message : null,
  };
}
