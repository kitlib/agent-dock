import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { marketplaceItems as mockMarketplaceItems } from "@/features/marketplace/mock";
import { useSkillsshMarketplaceQuery } from "@/features/marketplace/queries";
import {
  buildDiscoveryItems,
  createMarketplaceInstallStateMap,
  filterDiscoveryItems,
  sortDiscoveryItems,
} from "@/features/resources/core/discovery";
import { resourcesByKind } from "@/features/resources/core/resource-catalog";
import { formatInstallCount } from "@/lib/utils";
import type {
  AgentDiscoveryItem,
  MarketplaceInstallStateLabel,
  McpResource,
  ResourceKind,
  ResolvedAgentView,
  SkillResource,
} from "@/features/agents/types";

function clearUnavailableCheckedIds(ids: string[], resources: AgentDiscoveryItem[]) {
  const availableIds = new Set(resources.map((resource) => resource.id));
  return ids.filter((id) => availableIds.has(id));
}

export function useResourceBrowser(
  search: string,
  selectedAgent: ResolvedAgentView | null,
  skills: SkillResource[],
  mcps: McpResource[]
) {
  const { i18n } = useTranslation();
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
  const skillMarketplaceItems = skillMarketplaceQuery.data?.items ?? [];
  const effectiveMarketplaceItems = useMemo(() => {
    const nonSkillItems = mockMarketplaceItems.filter((item) => item.kind !== "skill");
    const skillItems =
      activeKind === "skill"
        ? skillMarketplaceItems
        : mockMarketplaceItems.filter((item) => item.kind === "skill");

    return [...skillItems, ...nonSkillItems];
  }, [activeKind, skillMarketplaceItems]);

  useEffect(() => {
    const defaultStates = createMarketplaceInstallStateMap(effectiveMarketplaceItems);
    setMarketplaceInstallStates(defaultStates);
  }, [effectiveMarketplaceItems, skills]);

  const localResources = useMemo(
    () => ({
      ...resourcesByKind,
      skill: skills,
      mcp: mcps,
    }),
    [mcps, skills]
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
    ).map((resource) =>
      resource.origin === "marketplace"
        ? {
            ...resource,
            formattedInstalls: formatInstallCount(resource.installs, i18n.language),
          }
        : resource
    );
  }, [discoveryItems, i18n.language, normalizedSearch]);

  useEffect(() => {
    setCheckedIds((current) => clearUnavailableCheckedIds(current, filteredResources));

    if (filteredResources.some((resource) => resource.id === selectedResourceId)) {
      return;
    }

    setSelectedResourceId("");
  }, [filteredResources, selectedResourceId]);

  const filteredResourceMap = useMemo(
    () => new Map(filteredResources.map((resource) => [resource.id, resource])),
    [filteredResources]
  );

  const selectedResourceBase =
    filteredResourceMap.get(selectedResourceId) ?? filteredResources[0] ?? null;

  const selectedResource = selectedResourceBase;

  const toggleChecked = (id: string) => {
    const item = filteredResourceMap.get(id);
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
    isMarketplaceLoading:
      skillMarketplaceQuery.isLoading ||
      (skillMarketplaceQuery.isFetching && !skillMarketplaceQuery.isFetchingNextPage),
    isMarketplaceLoadingMore: skillMarketplaceQuery.isFetchingNextPage,
    hasMoreMarketplaceItems: skillMarketplaceQuery.hasNextPage ?? false,
    loadMoreMarketplaceItems: async () => {
      if (!skillMarketplaceQuery.hasNextPage || skillMarketplaceQuery.isFetchingNextPage) {
        return;
      }

      await skillMarketplaceQuery.fetchNextPage();
    },
    marketplaceTotalSkills: skillMarketplaceQuery.data?.totalSkills ?? null,
    marketplaceError:
      skillMarketplaceQuery.error instanceof Error ? skillMarketplaceQuery.error.message : null,
  };
}
