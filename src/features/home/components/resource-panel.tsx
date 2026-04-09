import type { DragEvent } from "react";
import { Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { AgentDiscoveryItem, ResourceKind } from "@/features/agents/types";
import { getLocalSkillToggleTarget } from "@/features/home/local-skill-toggle";
import { kindIcons } from "@/features/shared/constants";
import { AgentResourceList } from "@/features/resources/core/components/resource-list";

const resourceKinds: readonly ResourceKind[] = ["skill", "mcp", "subagent"];

type Translate = (key: string, options?: Record<string, unknown>) => string;

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
  onClearChecked: () => void;
  onToggleCheckedSkills: () => Promise<void>;
  onDragStart: (event: DragEvent<HTMLDivElement>, resourceId: string) => void;
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
  onUpdateMarketplaceInstallState: (id: string) => void;
  search: string;
  totalCount: number;
  selectedResourceId: string;
  t: (key: string, options?: Record<string, unknown>) => string;
};

export function AgentResourcePanel({
  activeKind,
  checkedIds,
  filteredResources,
  onClearChecked,
  onToggleCheckedSkills,
  onDragStart,
  onOpenSkillFolder,
  onSearchChange,
  onSelectKind,
  onSelectResource,
  onSetLocalSkillEnabled,
  onToggleChecked,
  onToggleAllChecked,
  onUpdateMarketplaceInstallState,
  search,
  totalCount,
  selectedResourceId,
  t,
}: AgentResourcePanelProps) {
  const hasToggleableSkill = filteredResources.some((resource) => {
    if (!checkedIds.includes(resource.id)) {
      return false;
    }
    return getLocalSkillToggleTarget(resource) != null;
  });
  const hasEnabledSkill = filteredResources.some((resource) => {
    if (!checkedIds.includes(resource.id)) {
      return false;
    }

    return getLocalSkillToggleTarget(resource)?.enabled ?? false;
  });
  return (
    <section className="flex h-full min-w-0 flex-col overflow-hidden">
      <div className="border-b p-3">
        <div className="flex flex-wrap items-center justify-between gap-2">
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
          <div className="bg-muted text-muted-foreground hidden rounded-md px-2 py-1 text-xs lg:flex">
            {t("prototype.actions.totalCount", { count: totalCount })}
          </div>
        </div>
        <div className="mt-3 flex flex-wrap items-center gap-2">
          <div className="relative min-w-[240px] flex-1">
            <Search className="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2" />
            <Input
              value={search}
              onChange={(event) => onSearchChange(event.target.value)}
              className="pl-9"
              placeholder={getSearchPlaceholder(activeKind, t)}
            />
          </div>
        </div>
      </div>

      {checkedIds.length > 0 ? (
        <div className="bg-muted/50 flex items-center justify-between border-b px-3 py-2 text-sm">
          <div className="flex items-center gap-2">
            <span>{t("prototype.actions.batchSelected", { count: checkedIds.length })}</span>
            <Button
              variant="outline"
              size="xs"
              onClick={() => onToggleAllChecked(filteredResources.map((r) => r.id))}
            >
              {t("prototype.actions.selectAll")}
            </Button>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="xs"
              disabled={!hasToggleableSkill}
              onClick={() => void onToggleCheckedSkills()}
            >
              {hasEnabledSkill
                ? t("prototype.actions.disable")
                : t("prototype.actions.enable")}
            </Button>
            <Button variant="outline" size="xs" onClick={onClearChecked}>
              {t("prototype.actions.clear")}
            </Button>
          </div>
        </div>
      ) : null}

      <div className="flex-1 overflow-auto p-2">
        {filteredResources.length === 0 ? (
          <div className="text-muted-foreground flex h-full items-center justify-center text-sm">
            {search ? t("prototype.noResults") : t("prototype.emptyList")}
          </div>
        ) : (
          <AgentResourceList
            checkedIds={checkedIds}
            filteredResources={filteredResources}
            onDragStart={onDragStart}
            onOpenSkillFolder={onOpenSkillFolder}
            onSelectResource={onSelectResource}
            onSetLocalSkillEnabled={onSetLocalSkillEnabled}
            onToggleChecked={onToggleChecked}
            onUpdateMarketplaceInstallState={onUpdateMarketplaceInstallState}
            selectedResourceId={selectedResourceId}
            t={t}
          />
        )}
      </div>
    </section>
  );
}
