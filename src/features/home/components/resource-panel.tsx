import { type DragEvent, useState } from "react";
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

function getSearchPlaceholder(activeKind: ResourceKind, t: Translate): string {
  if (activeKind === "skill") {
    return t("prototype.actions.searchSkillsPlaceholder");
  }

  return t("prototype.actions.searchPlaceholder");
}

type AgentResourcePanelProps = {
  activeKind: ResourceKind;
  checkedIds: string[];
  filteredResources: AgentDiscoveryItem[];
  isAllAgentsView: boolean;
  onClearChecked: () => void;
  onCopySkill: (source: LocalSkillCopySource) => void;
  onCopySkills: (sources: LocalSkillCopySource[]) => void;
  onDeleteLocalSkill: (skillPath: string, entryFilePath: string, skillId?: string) => Promise<void>;
  onToggleCheckedSkills: () => Promise<void>;
  onDragStart: (event: DragEvent<HTMLDivElement>, resourceId: string) => void;
  onOpenSkillEntryFile: (skillPath: string, entryFilePath: string) => Promise<void>;
  onOpenSkillFolder: (skillPath: string) => void;
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
  marketplaceError: string | null;
  search: string;
  selectedResourceId: string;
  t: (key: string, options?: Record<string, unknown>) => string;
};

export function AgentResourcePanel({
  activeKind,
  checkedIds,
  filteredResources,
  isAllAgentsView,
  onClearChecked,
  onCopySkill,
  onCopySkills,
  onDeleteLocalSkill,
  onToggleCheckedSkills,
  onDragStart,
  onOpenSkillEntryFile,
  onOpenSkillFolder,
  onSearchChange,
  onSelectKind,
  onSelectResource,
  onSetLocalSkillEnabled,
  onToggleChecked,
  onToggleAllChecked,
  onInstallMarketplaceItem,
  isMarketplaceLoading,
  marketplaceError,
  search,
  selectedResourceId,
  t,
}: AgentResourcePanelProps) {
  const [sourceFilter, setSourceFilter] = useState<ResourceSourceFilter>("all");
  const sourceCounts: Record<ResourceSourceFilter, number> = {
    all: filteredResources.length,
    local: filteredResources.filter((resource) => resource.origin === "local").length,
    marketplace: filteredResources.filter((resource) => resource.origin === "marketplace").length,
  };

  const visibleResources = filteredResources.filter((resource) => {
    if (sourceFilter === "all") {
      return true;
    }

    return resource.origin === sourceFilter;
  });

  const visibleCheckedIds = checkedIds.filter((id) =>
    visibleResources.some((resource) => resource.id === id)
  );
  const visibleLocalResources = visibleResources.filter((resource) => resource.origin === "local");
  const hasToggleableSkill = visibleResources.some((resource) => {
    if (!visibleCheckedIds.includes(resource.id)) {
      return false;
    }

    return getLocalSkillToggleTarget(resource) != null;
  });
  const hasEnabledSkill = visibleResources.some((resource) => {
    if (!visibleCheckedIds.includes(resource.id)) {
      return false;
    }

    return getLocalSkillToggleTarget(resource)?.enabled ?? false;
  });
  const hasCopyableSkills = visibleCheckedIds.some((id) => {
    const resource = visibleResources.find((candidate) => candidate.id === id);
    return resource?.origin === "local" && resource.kind === "skill";
  });

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
              <span className="ml-1 text-xs opacity-80">{sourceCounts[filter]}</span>
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
                  .map((id) => visibleResources.find((resource) => resource.id === id))
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

      <div className="flex-1 overflow-auto px-3 py-2">
        {visibleResources.length === 0 ? (
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
              onDragStart={onDragStart}
              onOpenSkillEntryFile={onOpenSkillEntryFile}
              onOpenSkillFolder={onOpenSkillFolder}
              onSelectResource={onSelectResource}
              onSetLocalSkillEnabled={onSetLocalSkillEnabled}
              onToggleChecked={onToggleChecked}
              onInstallMarketplaceItem={onInstallMarketplaceItem}
              selectedResourceId={selectedResourceId}
              t={t}
            />
          </div>
        )}
      </div>
    </section>
  );
}
