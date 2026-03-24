import type { DragEvent } from "react";
import { Download, MoreHorizontal } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";
import { installStateKey, kindIcons } from "./constants";
import type { AgentDiscoveryItem } from "./types";

type AgentResourceListProps = {
  checkedIds: string[];
  filteredResources: AgentDiscoveryItem[];
  onDragStart: (event: DragEvent<HTMLDivElement>, resourceId: string) => void;
  onSelectResource: (resource: AgentDiscoveryItem) => void;
  onToggleChecked: (id: string) => void;
  onUpdateMarketplaceInstallState: (id: string) => void;
  selectedResourceId: string;
  t: (key: string, options?: Record<string, unknown>) => string;
};

function renderDiscoveryMeta(resource: AgentDiscoveryItem, t: AgentResourceListProps["t"]) {
  return (
    <div className="text-muted-foreground mt-1 flex flex-wrap items-center gap-2 text-xs">
      <span className="bg-muted rounded px-2 py-1">
        {resource.origin === "local"
          ? t("prototype.badges.local")
          : t("prototype.badges.marketplace")}
      </span>
      <span className="bg-muted rounded px-2 py-1">
        {t(installStateKey[resource.installState])}
      </span>
      {resource.origin === "local" && resource.usageLabel !== undefined ? (
        <span>{t("prototype.detail.usedBy", { count: resource.usageLabel })}</span>
      ) : null}
      {resource.origin === "marketplace" ? (
        <>
          <span>{resource.author}</span>
          <span>
            <Download className="mr-1 inline h-3 w-3" />
            {resource.downloads}
          </span>
        </>
      ) : null}
    </div>
  );
}

export function AgentResourceList({
  checkedIds,
  filteredResources,
  onDragStart,
  onSelectResource,
  onToggleChecked,
  onUpdateMarketplaceInstallState,
  selectedResourceId,
  t,
}: AgentResourceListProps) {
  return (
    <div className="space-y-1">
      {filteredResources.map((resource) => {
        const Icon = kindIcons[resource.kind];
        const active = resource.id === selectedResourceId;

        return (
          <div
            key={resource.id}
            draggable={resource.origin === "local"}
            onDragStart={(event) => onDragStart(event, resource.id)}
            onClick={() => onSelectResource(resource)}
            className={cn(
              "group border-border/70 hover:bg-accent/40 rounded-lg border px-3 py-2 transition-colors",
              active ? "bg-accent border-primary/40" : "bg-background"
            )}
          >
            <div className="flex items-start gap-3">
              {resource.origin === "local" ? (
                <Checkbox
                  checked={checkedIds.includes(resource.id)}
                  onCheckedChange={() => onToggleChecked(resource.id)}
                  onClick={(event) => event.stopPropagation()}
                  className="mt-1"
                  aria-label={resource.name}
                />
              ) : null}
              <Icon className="text-muted-foreground mt-0.5 h-4 w-4 shrink-0" />
              <div className="min-w-0 flex-1">
                <div className="flex items-center justify-between gap-3">
                  <div className="truncate text-sm font-medium">{resource.name}</div>
                  <div className="text-muted-foreground text-xs">{resource.updatedAt}</div>
                </div>
                <div className="text-muted-foreground mt-1 line-clamp-1 text-xs">
                  {resource.summary}
                </div>
                {renderDiscoveryMeta(resource, t)}
              </div>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    onClick={(event) => event.stopPropagation()}
                  >
                    <MoreHorizontal className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  {resource.origin === "local" ? (
                    <>
                      <DropdownMenuItem>{t("prototype.actions.enable")}</DropdownMenuItem>
                      <DropdownMenuItem>{t("prototype.actions.disable")}</DropdownMenuItem>
                    </>
                  ) : (
                    <DropdownMenuItem onClick={() => onUpdateMarketplaceInstallState(resource.id)}>
                      {t(installStateKey[resource.installState])}
                    </DropdownMenuItem>
                  )}
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </div>
        );
      })}
    </div>
  );
}
