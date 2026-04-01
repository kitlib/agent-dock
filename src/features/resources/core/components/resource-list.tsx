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
import { installStateKey, kindIcons } from "@/features/shared/constants";
import type { AgentDiscoveryItem } from "@/features/agents/types";

type AgentResourceListProps = {
  checkedIds: string[];
  filteredResources: AgentDiscoveryItem[];
  onDragStart: (event: DragEvent<HTMLDivElement>, resourceId: string) => void;
  onOpenSkillFolder: (skillPath: string) => void;
  onSelectResource: (resource: AgentDiscoveryItem) => void;
  onToggleChecked: (id: string) => void;
  onUpdateMarketplaceInstallState: (id: string) => void;
  selectedResourceId: string;
  t: (key: string, options?: Record<string, unknown>) => string;
};

function renderDiscoveryMeta(
  resource: AgentDiscoveryItem,
  t: AgentResourceListProps["t"],
  active: boolean
) {
  const shouldShowInstallState = !(resource.origin === "local" && resource.kind === "skill");
  const badgeClassName = active
    ? "bg-background/85 text-foreground border-border/70 border"
    : "bg-muted text-muted-foreground";

  return (
    <div
      className={cn(
        "mt-1 flex flex-wrap items-center gap-2 text-xs",
        active ? "text-foreground/75" : "text-muted-foreground"
      )}
    >
      <span className={cn("rounded px-2 py-1", badgeClassName)}>
        {resource.origin === "local"
          ? t("prototype.badges.local")
          : t("prototype.badges.marketplace")}
      </span>
      {shouldShowInstallState ? (
        <span className={cn("rounded px-2 py-1", badgeClassName)}>
          {t(installStateKey[resource.installState])}
        </span>
      ) : null}
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
  onOpenSkillFolder,
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
        const shouldShowOpenSkillFolder = resource.origin === "local" && resource.kind === "skill";
        const skillPath = shouldShowOpenSkillFolder ? resource.skillPath?.trim() ?? "" : "";
        const disableOpenSkillFolder = skillPath.length === 0;

        return (
          <div
            key={resource.id}
            draggable={resource.origin === "local"}
            onDragStart={(event) => onDragStart(event, resource.id)}
            onClick={() => onSelectResource(resource)}
            className={cn(
              "group border-border/70 hover:bg-accent/40 rounded-lg border px-3 py-2 transition-colors",
              active ? "bg-accent/80 border-primary/40 text-accent-foreground" : "bg-background"
            )}
          >
            <div className="flex items-start gap-3">
              {resource.origin === "local" ? (
                <Checkbox
                  checked={checkedIds.includes(resource.id)}
                  onCheckedChange={() => onToggleChecked(resource.id)}
                  onClick={(event) => event.stopPropagation()}
                  className={cn(
                    "mt-1",
                    active &&
                      "border-foreground/50 data-[state=checked]:border-foreground data-[state=checked]:bg-foreground data-[state=checked]:text-background"
                  )}
                  aria-label={resource.name}
                />
              ) : null}
              <Icon
                className={cn(
                  "mt-0.5 h-4 w-4 shrink-0",
                  active ? "text-foreground/80" : "text-muted-foreground"
                )}
              />
              <div className="min-w-0 flex-1">
                <div className="flex items-center justify-between gap-3">
                  <div className={cn("truncate text-sm font-medium", active && "text-foreground")}>
                    {resource.name}
                  </div>
                  <div className={cn("text-xs", active ? "text-foreground/70" : "text-muted-foreground")}>
                    {resource.updatedAt}
                  </div>
                </div>
                <div
                  className={cn(
                    "mt-1 line-clamp-1 text-xs",
                    active ? "text-foreground/80" : "text-muted-foreground"
                  )}
                >
                  {resource.summary}
                </div>
                {renderDiscoveryMeta(resource, t, active)}
              </div>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    className={cn(active && "text-foreground/80 hover:text-foreground")}
                    onClick={(event) => event.stopPropagation()}
                  >
                    <MoreHorizontal className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  {resource.origin === "local" ? (
                    <>
                      {shouldShowOpenSkillFolder ? (
                        <DropdownMenuItem
                          disabled={disableOpenSkillFolder}
                          onClick={() => {
                            if (!disableOpenSkillFolder) {
                              onOpenSkillFolder(skillPath);
                            }
                          }}
                        >
                          {t("prototype.actions.open")}
                        </DropdownMenuItem>
                      ) : null}
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
