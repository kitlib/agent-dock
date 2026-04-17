import type { MarketplaceItem } from "@/features/marketplace/types";
import type {
  AgentDiscoveryItem,
  AgentResource,
  AgentResourceView,
  MarketplaceDiscoveryItem,
  MarketplaceInstallStateLabel,
  ResourceKind,
} from "@/features/agents/types";

type DiscoveryFilterOptions = {
  includeMarketplaceWhenEmpty?: boolean;
};

function sanitizeMarketplaceSkillId(skillId: string): string {
  const sanitized = skillId
    .split("")
    .map((char) => (/^[a-zA-Z0-9_-]$/.test(char) ? char : "-"))
    .join("")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");

  return sanitized || "marketplace-skill";
}

function lastPathSegment(path: string | undefined): string {
  if (!path) {
    return "";
  }

  const segments = path.split(/[\\/]+/).filter(Boolean);
  return segments[segments.length - 1] ?? "";
}

function getMarketplaceString(
  resource: AgentResourceView,
  key: "marketplaceRemoteId" | "marketplaceSource"
): string | undefined {
  if (resource.kind !== "skill" || resource.origin !== "local") {
    return undefined;
  }

  const value = resource[key];
  return typeof value === "string" ? value : undefined;
}

function matchesMarketplaceSkill(item: MarketplaceItem, resource: AgentResourceView): boolean {
  if (
    item.kind !== "skill" ||
    !item.skillId ||
    resource.kind !== "skill" ||
    resource.origin !== "local"
  ) {
    return false;
  }

  const remoteId = getMarketplaceString(resource, "marketplaceRemoteId");
  const source = getMarketplaceString(resource, "marketplaceSource");
  if (remoteId === item.skillId && source === item.source) {
    return true;
  }

  const expectedDirectoryName = sanitizeMarketplaceSkillId(item.skillId);
  const directoryName =
    lastPathSegment(resource.relativePath) || lastPathSegment(resource.skillPath);

  return directoryName === expectedDirectoryName;
}

function isInstalledMarketplaceSkill(
  item: MarketplaceItem,
  localItems: AgentResourceView[],
  ownerAgentId: string | null
): boolean {
  if (item.kind !== "skill" || !item.skillId) {
    return false;
  }

  return localItems.some((resource) => {
    if (resource.origin !== "local" || resource.kind !== "skill") {
      return false;
    }

    if (ownerAgentId != null && resource.ownerAgentId !== ownerAgentId) {
      return false;
    }

    return matchesMarketplaceSkill(item, resource);
  });
}

function getInstallState(resource: AgentResource): "enabled" | "installed" {
  return resource.enabled ? "enabled" : "installed";
}

export function toLocalDiscoveryItem(
  resource: AgentResource,
  ownerAgentId: string | null,
  managed: boolean
): AgentResourceView {
  const sourceLabel = "sourceLabel" in resource ? resource.sourceLabel : undefined;
  const resolvedOwnerAgentId = "ownerAgentId" in resource ? resource.ownerAgentId : undefined;

  return {
    ...resource,
    origin: "local",
    installState: getInstallState(resource),
    sourceLabel: sourceLabel ?? "local",
    version: undefined,
    author: undefined,
    installs: undefined,
    description: resource.summary,
    highlights: [],
    usageLabel: resource.usageCount,
    ownerAgentId: resolvedOwnerAgentId ?? ownerAgentId,
    managed,
  };
}

export function toMarketplaceDiscoveryItem(item: MarketplaceItem): MarketplaceDiscoveryItem {
  return {
    id: item.id,
    kind: item.kind,
    name: item.name,
    updatedAt: item.updatedAt,
    origin: "marketplace",
    installState: item.installState === "install" ? "available" : item.installState,
    sourceLabel: item.source,
    skillId: item.skillId,
    version: item.version,
    author: item.author,
    installs: item.installs,
    description: item.description,
    highlights: item.highlights,
    url: item.url,
    usageLabel: undefined,
  };
}

export function getSearchScore(item: AgentDiscoveryItem, keyword: string) {
  if (!keyword) {
    return item.origin === "local" ? 2 : 1;
  }

  const normalizedName = item.name.toLowerCase();
  const normalizedDescription =
    item.origin === "marketplace" ? item.description.toLowerCase() : item.summary.toLowerCase();
  const normalizedSource = item.sourceLabel.toLowerCase();
  const normalizedAuthor = item.author?.toLowerCase() ?? "";

  let score = 0;

  if (normalizedName === keyword) score += 12;
  else if (normalizedName.startsWith(keyword)) score += 8;
  else if (normalizedName.includes(keyword)) score += 5;

  if (normalizedDescription.includes(keyword)) score += 3;
  if (normalizedSource.includes(keyword)) score += 2;
  if (normalizedAuthor.includes(keyword)) score += 2;
  if (score === 0) return 0;
  if (item.origin === "local") score += 1;
  if (item.installState === "enabled" || item.installState === "installed") score += 1;

  return score;
}

export function filterDiscoveryItems(
  items: AgentDiscoveryItem[],
  keyword: string,
  options: DiscoveryFilterOptions = {}
) {
  const { includeMarketplaceWhenEmpty = true } = options;

  return items.filter((item) => {
    if (!keyword) {
      return includeMarketplaceWhenEmpty || item.origin === "local";
    }

    return getSearchScore(item, keyword) > 0;
  });
}

export function sortDiscoveryItems(items: AgentDiscoveryItem[], keyword: string) {
  return [...items].sort((left, right) => {
    if (keyword && left.origin !== right.origin) {
      return left.origin === "local" ? -1 : 1;
    }

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
    .filter((item) => !isInstalledMarketplaceSkill(item, localItems, ownerAgentId))
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
