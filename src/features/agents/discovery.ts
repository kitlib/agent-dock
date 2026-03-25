import type { MarketplaceItem } from "@/features/marketplace/types";
import type {
  AgentDiscoveryItem,
  AgentResource,
  AgentResourceView,
  MarketplaceDiscoveryItem,
  MarketplaceInstallStateLabel,
  ResourceKind,
} from "./types";

function getInstallState(resource: AgentResource): "enabled" | "installed" {
  return resource.enabled ? "enabled" : "installed";
}

export function toLocalDiscoveryItem(
  resource: AgentResource,
  ownerAgentId: string | null,
  managed: boolean
): AgentResourceView {
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
    ownerAgentId,
    managed,
    configPath: undefined,
    conflictState: undefined,
  };
}

export function toMarketplaceDiscoveryItem(item: MarketplaceItem): MarketplaceDiscoveryItem {
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

export function getSearchScore(item: AgentDiscoveryItem, keyword: string) {
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

export function filterDiscoveryItems(items: AgentDiscoveryItem[], keyword: string) {
  return items.filter((item) => {
    if (!keyword) return true;
    return getSearchScore(item, keyword) > 0;
  });
}

export function sortDiscoveryItems(items: AgentDiscoveryItem[], keyword: string) {
  return [...items].sort((left, right) => {
    const scoreDiff = getSearchScore(right, keyword) - getSearchScore(left, keyword);
    if (scoreDiff !== 0) return scoreDiff;
    return right.updatedAt.localeCompare(left.updatedAt);
  });
}

export function buildDiscoveryItems(
  kind: ResourceKind,
  localResources: Record<ResourceKind, AgentResource[]>,
  marketplaceItems: MarketplaceItem[],
  marketplaceInstallStates: Record<string, MarketplaceInstallStateLabel>,
  ownerAgentId: string | null,
  managed: boolean
) {
  const localItems = localResources[kind].map((resource) =>
    toLocalDiscoveryItem(resource, ownerAgentId, managed)
  );
  const marketplaceResults = marketplaceItems
    .filter((item) => item.kind === kind)
    .map((item) => {
      const discoveryItem = toMarketplaceDiscoveryItem(item);
      return {
        ...discoveryItem,
        installState: marketplaceInstallStates[item.id] ?? discoveryItem.installState,
      } satisfies MarketplaceDiscoveryItem;
    });

  return [...localItems, ...marketplaceResults];
}

export function createMarketplaceInstallStateMap(items: MarketplaceItem[]) {
  return Object.fromEntries(
    items.map((item) => [
      item.id,
      item.installState === "install" ? "available" : item.installState,
    ])
  ) as Record<string, MarketplaceInstallStateLabel>;
}
