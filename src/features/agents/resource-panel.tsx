import { Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { kindIcons } from "./constants";
import type { AgentDiscoveryItem, ResourceKind } from "./types";
import { AgentResourceList } from "./resource-list";
import type { DragEvent } from "react";

type AgentResourcePanelProps = {
  activeKind: ResourceKind;
  checkedIds: string[];
  filteredResources: AgentDiscoveryItem[];
  isRailCollapsed: boolean;
  onClearChecked: () => void;
  onDragStart: (event: DragEvent<HTMLDivElement>, resourceId: string) => void;
  onSearchChange: (value: string) => void;
  onSelectKind: (kind: ResourceKind) => void;
  onSelectResource: (resource: AgentDiscoveryItem) => void;
  onToggleChecked: (id: string) => void;
  onUpdateMarketplaceInstallState: (id: string) => void;
  search: string;
  selectedAgentName?: string;
  selectedResourceId: string;
  t: (key: string, options?: Record<string, unknown>) => string;
};

export function AgentResourcePanel({
  activeKind,
  checkedIds,
  filteredResources,
  isRailCollapsed,
  onClearChecked,
  onDragStart,
  onSearchChange,
  onSelectKind,
  onSelectResource,
  onToggleChecked,
  onUpdateMarketplaceInstallState,
  search,
  selectedAgentName,
  selectedResourceId,
  t,
}: AgentResourcePanelProps) {
  return (
    <section className="flex h-full min-w-0 flex-col overflow-hidden">
      <div className="border-b p-3">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex flex-wrap items-center gap-2">
            {(["skill", "mcp", "subagent"] as ResourceKind[]).map((kind) => {
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
          {selectedAgentName && !isRailCollapsed ? (
            <div className="bg-muted text-muted-foreground hidden rounded-md px-2 py-1 text-xs lg:flex">
              {selectedAgentName}
            </div>
          ) : null}
        </div>
        <div className="mt-3 flex flex-wrap items-center gap-2">
          <div className="relative min-w-[240px] flex-1">
            <Search className="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2" />
            <Input
              value={search}
              onChange={(event) => onSearchChange(event.target.value)}
              className="pl-9"
              placeholder={t("prototype.actions.searchPlaceholder")}
            />
          </div>
        </div>
      </div>

      {checkedIds.length > 0 ? (
        <div className="bg-muted/50 flex items-center justify-between border-b px-3 py-2 text-sm">
          <span>{t("prototype.actions.batchSelected", { count: checkedIds.length })}</span>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="xs">
              {t("prototype.actions.disable")}
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
            onSelectResource={onSelectResource}
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
