import { useEffect, useMemo, useRef, useState, type DragEvent } from "react";
import { Copy, Loader2, Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ButtonGroup } from "@/components/ui/button-group";
import { Input } from "@/components/ui/input";
import type {
  AgentDiscoveryItem,
  LocalSkillCopySource,
  ResourceKind,
  SkillResource,
} from "@/features/agents/types";
import { getLocalSkillToggleTarget } from "@/features/home/local-skill-toggle";
import { kindIcons } from "@/features/shared/constants";
import { AgentResourceList } from "@/features/resources/core/components/resource-list";

const resourceKinds: readonly ResourceKind[] = ["skill", "mcp", "subagent"];
const sourceFilters = ["all", "local", "marketplace"] as const;

type Translate = (key: string, options?: Record<string, unknown>) => string;
type ResourceSourceFilter = (typeof sourceFilters)[number];

function MarketplaceListSkeleton() {
  return (
    <div className="space-y-1">
      {Array.from({ length: 8 }, (_, index) => (
        <div
          key={index}
          className="rounded-lg border border-border/70 bg-background px-3 py-2 animate-pulse"
        >
          <div className="flex items-start gap-3">
            <div className="mt-1 h-4 w-4 rounded-sm bg-muted" />
            <div className="min-w-0 flex-1">
              <div className="h-4 w-2/5 rounded bg-muted" />
              <div className="mt-2 flex items-center gap-2">
                <div className="h-3 w-20 rounded bg-muted" />
                <div className="h-3 w-24 rounded bg-muted" />
                <div className="h-3 w-16 rounded bg-muted" />
              </div>
            </div>
            <div className="h-7 w-16 rounded-md bg-muted" />
          </div>
        </div>
      ))}
    </div>
  );
}

function getSearchPlaceholder(activeKind: ResourceKind, t: Translate): string {
  if (activeKind === "skill") {
    return t("prototype.actions.searchSkillsPlaceholder");
  }

  return t("prototype.actions.searchPlaceholder");
}

type AgentResourcePanelProps = {
  activeKind: ResourceKind;
  canImportMcp: boolean;
  checkedIds: string[];
  filteredResources: AgentDiscoveryItem[];
  isAllAgentsView: boolean;
  onClearChecked: () => void;
  onCopySkill: (source: LocalSkillCopySource) => void;
  onCopySkills: (sources: LocalSkillCopySource[]) => void;
  onDeleteLocalSkill: (skillPath: string, entryFilePath: string, skillId?: string) => Promise<void>;
  onDeleteLocalMcp: (agentType: string, configPath: string, serverName: string) => Promise<void>;
  onImportMcp: () => void;
  onToggleCheckedSkills: () => Promise<void>;
  onDragStart: (event: DragEvent<HTMLDivElement>, resourceId: string) => void;
  onOpenSkillEntryFile: (skillPath: string, entryFilePath: string) => Promise<void>;
  onOpenSkillFolder: (skillPath: string) => void;
  onOpenMcpConfigFile: (configPath: string) => Promise<void>;
  onOpenMcpConfigFolder: (configPath: string) => void;
  onSearchChange: (value: string) => void;
  onSelectKind: (kind: ResourceKind) => void;
  onSelectResource: (resource: AgentDiscoveryItem) => void;
  onSetLocalSkillEnabled: (
    skillPath: string,
    entryFilePath: string,
    enabled: boolean,
    skillId?: string
  ) => Promise<void>;
  onToggleChecked: (id: string) => void;
  onToggleAllChecked: (ids: string[]) => void;
  onInstallMarketplaceItem: (resource: AgentDiscoveryItem) => Promise<void>;
  isMarketplaceLoading: boolean;
  isMarketplaceLoadingMore: boolean;
  hasMoreMarketplaceItems: boolean;
  onLoadMoreMarketplaceItems: () => Promise<void>;
  marketplaceError: string | null;
  marketplaceTotalSkills: number | null;
  search: string;
  selectedResourceId: string;
  t: (key: string, options?: Record<string, unknown>) => string;
};

export function AgentResourcePanel({
  activeKind,
  canImportMcp,
  checkedIds,
  filteredResources,
  isAllAgentsView,
  onClearChecked,
  onCopySkill,
  onCopySkills,
  onDeleteLocalSkill,
  onDeleteLocalMcp,
  onImportMcp,
  onToggleCheckedSkills,
  onDragStart,
  onOpenSkillEntryFile,
  onOpenSkillFolder,
  onOpenMcpConfigFile,
  onOpenMcpConfigFolder,
  onSearchChange,
  onSelectKind,
  onSelectResource,
  onSetLocalSkillEnabled,
  onToggleChecked,
  onToggleAllChecked,
  onInstallMarketplaceItem,
  isMarketplaceLoading,
  isMarketplaceLoadingMore,
  hasMoreMarketplaceItems,
  onLoadMoreMarketplaceItems,
  marketplaceError,
  marketplaceTotalSkills,
  search,
  selectedResourceId,
  t,
}: AgentResourcePanelProps) {
  const [sourceFilter, setSourceFilter] = useState<ResourceSourceFilter>("all");
  const scrollContainerRef = useRef<HTMLDivElement | null>(null);
  const sourceCounts = useMemo(() => {
    const counts = filteredResources.reduce<Record<ResourceSourceFilter, number>>(
      (counts, resource) => {
        counts.all += 1;
        counts[resource.origin] += 1;
        return counts;
      },
      { all: 0, local: 0, marketplace: 0 }
    );

    if (activeKind === "skill" && marketplaceTotalSkills != null) {
      counts.marketplace = marketplaceTotalSkills;
      counts.all = counts.local + marketplaceTotalSkills;
    }

    return counts;
  }, [activeKind, filteredResources, marketplaceTotalSkills]);

  const visibleResources = useMemo(
    () =>
      filteredResources.filter((resource) =>
        sourceFilter === "all" ? true : resource.origin === sourceFilter
      ),
    [filteredResources, sourceFilter]
  );

  const visibleResourceMap = useMemo(
    () => new Map(visibleResources.map((resource) => [resource.id, resource])),
    [visibleResources]
  );

  const visibleCheckedIds = useMemo(
    () => checkedIds.filter((id) => visibleResourceMap.has(id)),
    [checkedIds, visibleResourceMap]
  );
  const visibleCheckedIdSet = useMemo(() => new Set(visibleCheckedIds), [visibleCheckedIds]);
  const visibleLocalResources = useMemo(
    () => visibleResources.filter((resource) => resource.origin === "local"),
    [visibleResources]
  );
  const selectedCheckedResources = useMemo(
    () =>
      visibleResources.filter(
        (resource) => visibleCheckedIdSet.has(resource.id) && resource.origin === "local"
      ),
    [visibleCheckedIdSet, visibleResources]
  );
  const hasToggleableSkill = selectedCheckedResources.some(
    (resource) => getLocalSkillToggleTarget(resource) != null
  );
  const hasEnabledSkill = selectedCheckedResources.some(
    (resource) => getLocalSkillToggleTarget(resource)?.enabled ?? false
  );
  const hasCopyableSkills = selectedCheckedResources.some((resource) => resource.kind === "skill");
  const shouldShowMarketplaceSkeleton = activeKind === "skill" && isMarketplaceLoading;
  const shouldHideSourceCount = (filter: ResourceSourceFilter) =>
    activeKind === "skill" &&
    isMarketplaceLoading &&
    (filter === "all" || filter === "marketplace") &&
    sourceCounts[filter] === 0;

  useEffect(() => {
    const container = scrollContainerRef.current;
    if (!container) {
      return;
    }

    const handleScroll = () => {
      if (
        activeKind !== "skill" ||
        sourceFilter === "local" ||
        !hasMoreMarketplaceItems ||
        isMarketplaceLoadingMore
      ) {
        return;
      }

      const remaining = container.scrollHeight - container.scrollTop - container.clientHeight;
      if (remaining > 240) {
        return;
      }

      void onLoadMoreMarketplaceItems();
    };

    container.addEventListener("scroll", handleScroll, { passive: true });
    handleScroll();

    return () => container.removeEventListener("scroll", handleScroll);
  }, [
    activeKind,
    hasMoreMarketplaceItems,
    isMarketplaceLoadingMore,
    onLoadMoreMarketplaceItems,
    sourceFilter,
    visibleResources.length,
  ]);

  useEffect(() => {
    if (visibleResources.length === 0) {
      return;
    }

    const selectedVisible = visibleResources.some((resource) => resource.id === selectedResourceId);
    if (selectedVisible) {
      return;
    }

    onSelectResource(visibleResources[0]!);
  }, [onSelectResource, selectedResourceId, visibleResources]);

  return (
    <section className="flex h-full min-w-0 flex-col overflow-hidden">
      <div className="border-b p-3">
        <div className="flex flex-wrap items-center gap-2">
          {resourceKinds.map((kind) => {
            const Icon = kindIcons[kind];
            const active = activeKind === kind;

            return (
              <Button
                key={kind}
                variant={active ? "default" : "outline"}
                size="sm"
                onClick={() => onSelectKind(kind)}
              >
                <Icon className="h-4 w-4" />
                {t(`prototype.tabs.${kind}`)}
              </Button>
            );
          })}
        </div>

        <div className="mt-3 flex flex-wrap items-center gap-2">
          <div className="relative min-w-[240px] flex-1">
            <Search className="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2" />
            {isMarketplaceLoading ? (
              <Loader2 className="text-muted-foreground absolute top-1/2 right-3 h-4 w-4 -translate-y-1/2 animate-spin" />
            ) : null}
            <Input
              value={search}
              onChange={(event) => onSearchChange(event.target.value)}
              className="pr-9 pl-9"
              placeholder={getSearchPlaceholder(activeKind, t)}
            />
          </div>
          {activeKind === "mcp" && canImportMcp ? (
            <Button variant="outline" size="sm" onClick={onImportMcp}>
              {t("prototype.actions.import")}
            </Button>
          ) : null}
        </div>

        {activeKind === "skill" && marketplaceError ? (
          <div className="mt-3 rounded-md border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-xs text-amber-800 dark:text-amber-200">
            {t("prototype.marketplace.loadFailed")}: {marketplaceError}
          </div>
        ) : null}
      </div>

      <div className="px-3 py-2">
        <ButtonGroup className="w-fit [&>*]:border">
          {sourceFilters.map((filter) => (
            <Button
              key={filter}
              variant={sourceFilter === filter ? "default" : "outline"}
              size="xs"
              onClick={() => setSourceFilter(filter)}
            >
              {filter === "all" ? t("prototype.tabs.all") : t(`prototype.badges.${filter}`)}
              {shouldHideSourceCount(filter) ? null : (
                <span className="ml-1 text-xs opacity-80">{sourceCounts[filter]}</span>
              )}
            </Button>
          ))}
        </ButtonGroup>
      </div>

      <div className="border-b" />

      {visibleCheckedIds.length > 0 ? (
        <div className="bg-muted/50 flex items-center justify-between border-b px-3 py-2 text-sm">
          <div className="flex items-center gap-2">
            <span>{t("prototype.actions.batchSelected", { count: visibleCheckedIds.length })}</span>
            <Button
              variant="outline"
              size="xs"
              onClick={() =>
                onToggleAllChecked(visibleLocalResources.map((resource) => resource.id))
              }
            >
              {t("prototype.actions.selectAll")}
            </Button>
            <Button variant="outline" size="xs" onClick={onClearChecked}>
              {t("prototype.actions.clear")}
            </Button>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="xs"
              disabled={!hasCopyableSkills}
              onClick={() => {
                const sources: LocalSkillCopySource[] = visibleCheckedIds
                  .map((id) => visibleResourceMap.get(id))
                  .filter(
                    (resource): resource is NonNullable<typeof resource> =>
                      resource != null && resource.origin === "local" && resource.kind === "skill"
                  )
                  .map((resource) => {
                    const skill = resource as SkillResource & { origin: "local" };
                    return {
                      id: skill.id,
                      name: skill.name,
                      ownerAgentId: skill.ownerAgentId ?? "",
                      sourceKind: skill.sourceKind ?? "skills",
                      relativePath: skill.relativePath ?? "",
                      skillPath: skill.skillPath ?? "",
                      entryFilePath: skill.entryFilePath ?? "",
                    };
                  });
                onCopySkills(sources);
              }}
            >
              <Copy className="h-3 w-3" />
              {t("prototype.actions.copy")}
            </Button>
            <Button
              variant="outline"
              size="xs"
              disabled={!hasToggleableSkill}
              onClick={() => void onToggleCheckedSkills()}
            >
              {hasEnabledSkill ? t("prototype.actions.disable") : t("prototype.actions.enable")}
            </Button>
          </div>
        </div>
      ) : null}

      <div ref={scrollContainerRef} className="flex-1 overflow-auto px-3 py-2">
        {shouldShowMarketplaceSkeleton ? (
          <div className="bg-background rounded-lg pb-2">
            <MarketplaceListSkeleton />
          </div>
        ) : visibleResources.length === 0 ? (
          <div className="text-muted-foreground flex h-full items-center justify-center text-sm">
            {search ? t("prototype.noResults") : t("prototype.emptyList")}
          </div>
        ) : (
          <div className="bg-background rounded-lg pb-2">
            <AgentResourceList
              checkedIds={checkedIds}
              filteredResources={visibleResources}
              showOriginBadge={sourceFilter === "all"}
              isAllAgentsView={isAllAgentsView}
              onCopySkill={onCopySkill}
              onDeleteLocalSkill={onDeleteLocalSkill}
              onDeleteLocalMcp={onDeleteLocalMcp}
              onDragStart={onDragStart}
              onOpenSkillEntryFile={onOpenSkillEntryFile}
              onOpenSkillFolder={onOpenSkillFolder}
              onOpenMcpConfigFile={onOpenMcpConfigFile}
              onOpenMcpConfigFolder={onOpenMcpConfigFolder}
              onSelectResource={onSelectResource}
              onSetLocalSkillEnabled={onSetLocalSkillEnabled}
              onToggleChecked={onToggleChecked}
              onInstallMarketplaceItem={onInstallMarketplaceItem}
              scrollContainerRef={scrollContainerRef}
              selectedResourceId={selectedResourceId}
              t={t}
            />
            {activeKind === "skill" && (isMarketplaceLoadingMore || hasMoreMarketplaceItems) ? (
              <div className="text-muted-foreground flex items-center justify-center gap-2 px-3 py-3 text-xs">
                {isMarketplaceLoadingMore ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : null}
                <span>
                  {isMarketplaceLoadingMore
                    ? t("prototype.actions.loadingMore")
                    : t("prototype.actions.scrollToLoadMore")}
                </span>
              </div>
            ) : null}
          </div>
        )}
      </div>
    </section>
  );
}
