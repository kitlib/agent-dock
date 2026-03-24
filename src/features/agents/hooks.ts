import { useMemo, useState } from "react";
import { marketplaceItems } from "@/features/marketplace/mock";
import { agentGroups, agents, resourcesByKind } from "./mock";
import type { AgentDiscoveryItem, AgentResource, InstallStateLabel, ResourceKind } from "./types";

function getInstallState(resource: AgentResource): "enabled" | "installed" {
  return resource.enabled ? "enabled" : "installed";
}

function toDiscoveryItem(resource: AgentResource): AgentDiscoveryItem {
  return {
    ...resource,
    origin: "local",
    installState: getInstallState(resource),
    sourceLabel: "local",
    version: undefined,
    author: undefined,
    downloads: undefined,
    description: resource.summary,
    highlights: [],
    usageLabel: resource.usageCount,
  };
}

function toMarketplaceDiscoveryItem(item: (typeof marketplaceItems)[number]): AgentDiscoveryItem {
  return {
    id: item.id,
    kind: item.kind,
    name: item.name,
    summary: item.summary,
    updatedAt: item.updatedAt,
    origin: "marketplace",
    installState: item.installState === "install" ? "available" : item.installState,
    sourceLabel: item.source,
    version: item.version,
    author: item.author,
    downloads: item.downloads,
    description: item.description,
    highlights: item.highlights,
    usageLabel: undefined,
  };
}

function getSearchScore(item: AgentDiscoveryItem, keyword: string) {
  if (!keyword) {
    return item.origin === "local" ? 2 : 1;
  }

  const normalizedName = item.name.toLowerCase();
  const normalizedSummary = item.summary.toLowerCase();
  const normalizedSource = item.sourceLabel.toLowerCase();
  const normalizedAuthor = item.author?.toLowerCase() ?? "";

  let score = 0;

  if (normalizedName === keyword) score += 12;
  else if (normalizedName.startsWith(keyword)) score += 8;
  else if (normalizedName.includes(keyword)) score += 5;

  if (normalizedSummary.includes(keyword)) score += 3;
  if (normalizedSource.includes(keyword)) score += 2;
  if (normalizedAuthor.includes(keyword)) score += 2;
  if (item.origin === "local") score += 1;
  if (item.installState === "enabled" || item.installState === "installed") score += 1;

  return score;
}

export function useAgentsPrototype() {
  const [selectedGroupId, setSelectedGroupId] = useState(agentGroups[0]?.id ?? "all");
  const [selectedAgentId, setSelectedAgentId] = useState(agents[0]?.id ?? "");
  const [activeKind, setActiveKind] = useState<ResourceKind>("skill");
  const [search, setSearch] = useState("");
  const [selectedResourceId, setSelectedResourceId] = useState("");
  const [checkedIds, setCheckedIds] = useState<string[]>([]);
  const [marketplaceInstallStates, setMarketplaceInstallStates] = useState<
    Record<string, InstallStateLabel>
  >(
    Object.fromEntries(
      marketplaceItems.map((item) => [
        item.id,
        item.installState === "install" ? "available" : item.installState,
      ])
    )
  );

  const filteredAgents = useMemo(() => {
    return agents.filter((agent) => {
      const matchGroup = selectedGroupId === "all" || agent.groupId === selectedGroupId;
      const keyword = search.trim().toLowerCase();
      const matchSearch =
        keyword.length === 0 ||
        agent.name.toLowerCase().includes(keyword) ||
        agent.role.toLowerCase().includes(keyword) ||
        agent.summary.toLowerCase().includes(keyword);
      return matchGroup && matchSearch;
    });
  }, [search, selectedGroupId]);

  const selectedAgent =
    filteredAgents.find((agent) => agent.id === selectedAgentId) ?? filteredAgents[0] ?? null;

  const filteredResources = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    const localItems = resourcesByKind[activeKind].map(toDiscoveryItem);
    const marketplaceResults: AgentDiscoveryItem[] = marketplaceItems
      .filter((item) => item.kind === activeKind)
      .map((item) => {
        const discoveryItem = toMarketplaceDiscoveryItem(item);
        return {
          ...discoveryItem,
          installState: marketplaceInstallStates[item.id] ?? discoveryItem.installState,
        } as AgentDiscoveryItem;
      });

    return [...localItems, ...marketplaceResults]
      .filter((item) => {
        if (!keyword) return true;
        return getSearchScore(item, keyword) > 0;
      })
      .sort((left, right) => {
        const scoreDiff = getSearchScore(right, keyword) - getSearchScore(left, keyword);
        if (scoreDiff !== 0) return scoreDiff;
        return right.updatedAt.localeCompare(left.updatedAt);
      });
  }, [activeKind, marketplaceInstallStates, search]);

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
    agentGroups,
    filteredAgents,
    selectedGroupId,
    setSelectedGroupId,
    selectedAgent,
    selectedAgentId,
    setSelectedAgentId,
    activeKind,
    selectKind,
    search,
    setSearch,
    filteredResources,
    selectedResource,
    selectedResourceId,
    selectResource,
    checkedIds,
    toggleChecked,
    clearChecked,
    updateMarketplaceInstallState,
  };
}
