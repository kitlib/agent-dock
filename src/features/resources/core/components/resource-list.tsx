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
import type { AgentDiscoveryItem, SkillResource } from "@/features/agents/types";
import { getLocalSkillToggleTarget } from "@/features/home/local-skill-toggle";
import { installStateKey } from "@/features/shared/constants";

type AgentResourceListProps = {
  checkedIds: string[];
  filteredResources: AgentDiscoveryItem[];
  isAllAgentsView: boolean;
  onDeleteLocalSkill: (skillPath: string, entryFilePath: string, skillId?: string) => Promise<void>;
  onDragStart: (event: DragEvent<HTMLDivElement>, resourceId: string) => void;
  onOpenSkillEntryFile: (skillPath: string, entryFilePath: string) => Promise<void>;
  onOpenSkillFolder: (skillPath: string) => void;
  onSelectResource: (resource: AgentDiscoveryItem) => void;
  onSetLocalSkillEnabled: (
    skillPath: string,
    entryFilePath: string,
    enabled: boolean,
    skillId?: string
  ) => Promise<void>;
  onToggleChecked: (id: string) => void;
  onUpdateMarketplaceInstallState: (id: string) => void;
  selectedResourceId: string;
  t: (key: string, options?: Record<string, unknown>) => string;
};

function renderDiscoveryMeta(
  resource: AgentDiscoveryItem,
  isAllAgentsView: boolean,
  t: AgentResourceListProps["t"],
  active: boolean
) {
  const isLocalSkill = resource.origin === "local" && resource.kind === "skill";
  const shouldShowInstallState = !isLocalSkill;
  const badgeClassName = active
    ? "border border-border/70 bg-background/85 px-1.5 py-0.5 text-[9px] leading-3 text-foreground"
    : "bg-muted px-1.5 py-0.5 text-[9px] leading-3 text-muted-foreground";
  const originLabel =
    resource.origin === "local" ? t("prototype.badges.local") : t("prototype.badges.marketplace");

  return (
    <div
      className={cn(
        "mt-1 flex flex-wrap items-center gap-2 text-xs",
        active ? "text-foreground/75" : "text-muted-foreground"
      )}
    >
      <span className={badgeClassName}>{originLabel}</span>
      {isAllAgentsView && isLocalSkill && resource.agentName ? (
        <span className={cn("rounded", badgeClassName)}>{resource.agentName}</span>
      ) : null}
      {isLocalSkill ? (
        <span
          className={cn(
            "rounded border px-1.5 py-0.5 text-[9px] leading-3",
            resource.enabled
              ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"
              : "border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300"
          )}
        >
          {resource.enabled ? t("prototype.actions.enabled") : t("prototype.actions.disabled")}
        </span>
      ) : null}
      {shouldShowInstallState ? (
        <span className={cn("rounded", badgeClassName)}>
          {t(installStateKey[resource.installState])}
        </span>
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
function renderResourceAction(
  resource: AgentDiscoveryItem,
  onDeleteLocalSkill: AgentResourceListProps["onDeleteLocalSkill"],
  onOpenSkillEntryFile: AgentResourceListProps["onOpenSkillEntryFile"],
  onOpenSkillFolder: AgentResourceListProps["onOpenSkillFolder"],
  onSetLocalSkillEnabled: AgentResourceListProps["onSetLocalSkillEnabled"],
  onUpdateMarketplaceInstallState: AgentResourceListProps["onUpdateMarketplaceInstallState"],
  t: AgentResourceListProps["t"]
) {
  if (resource.origin === "marketplace") {
    return (
      <DropdownMenuItem onClick={() => onUpdateMarketplaceInstallState(resource.id)}>
        {t(installStateKey[resource.installState])}
      </DropdownMenuItem>
    );
  }

  const isSkill = resource.kind === "skill";
  // Type assertion for skill-specific properties
  const skillResource = resource as SkillResource & { origin: "local" };
  const skillPath = isSkill ? (skillResource.skillPath?.trim() ?? "") : "";
  const entryFilePath = isSkill ? (skillResource.entryFilePath ?? "") : "";
  const canOpenSkillFolder = skillPath.length > 0;
  const canEditSkill = isSkill && skillPath.length > 0 && entryFilePath.length > 0;
  const canDeleteSkill = isSkill && skillPath.length > 0;
  const skillToggleTarget = getLocalSkillToggleTarget(resource);

  return (
    <>
      {isSkill && canEditSkill ? (
        <DropdownMenuItem onClick={() => void onOpenSkillEntryFile(skillPath, entryFilePath)}>
          {t("prototype.actions.edit")}
        </DropdownMenuItem>
      ) : null}
      {isSkill ? (
        <DropdownMenuItem
          disabled={!canOpenSkillFolder}
          onClick={() => {
            if (canOpenSkillFolder) {
              onOpenSkillFolder(skillPath);
            }
          }}
        >
          {t("prototype.actions.open")}
        </DropdownMenuItem>
      ) : null}
      {isSkill && canDeleteSkill ? (
        <DropdownMenuItem
          className="text-destructive focus:text-destructive"
          onClick={() => void onDeleteLocalSkill(skillPath, entryFilePath, skillResource.id)}
        >
          {t("prototype.actions.delete")}
        </DropdownMenuItem>
      ) : null}
      {skillToggleTarget ? (
        <DropdownMenuItem
          onClick={() =>
            void onSetLocalSkillEnabled(
              skillToggleTarget.skillPath,
              skillToggleTarget.entryFilePath,
              !skillToggleTarget.enabled,
              skillToggleTarget.id
            )
          }
        >
          {skillToggleTarget.enabled
            ? t("prototype.actions.disable")
            : t("prototype.actions.enable")}
        </DropdownMenuItem>
      ) : null}
    </>
  );
}

function isLocalResource(resource: AgentDiscoveryItem): boolean {
  return resource.origin === "local";
}

function isSelectedResource(resourceId: string, selectedResourceId: string): boolean {
  return resourceId === selectedResourceId;
}

function getCheckboxClassName(active: boolean): string {
  return cn(
    "mt-1",
    active &&
      "border-foreground/50 data-[state=checked]:border-foreground data-[state=checked]:bg-foreground data-[state=checked]:text-background"
  );
}

function getCardClassName(active: boolean): string {
  return cn(
    "group rounded-lg border border-border/70 px-3 py-2 transition-colors hover:bg-accent/40",
    active ? "border-primary/40 bg-accent/80 text-accent-foreground" : "bg-background"
  );
}

function getSummaryClassName(active: boolean): string {
  return cn("mt-1 line-clamp-1 text-xs", active ? "text-foreground/80" : "text-muted-foreground");
}

function getActionButtonClassName(active: boolean): string {
  return cn(active && "text-foreground/80 hover:text-foreground");
}

export function AgentResourceList({
  checkedIds,
  filteredResources,
  isAllAgentsView,
  onDeleteLocalSkill,
  onDragStart,
  onOpenSkillEntryFile,
  onOpenSkillFolder,
  onSelectResource,
  onSetLocalSkillEnabled,
  onToggleChecked,
  onUpdateMarketplaceInstallState,
  selectedResourceId,
  t,
}: AgentResourceListProps) {
  return (
    <div className="space-y-1">
      {filteredResources.map((resource) => {
        const active = isSelectedResource(resource.id, selectedResourceId);

        return (
          <div
            key={resource.id}
            draggable={isLocalResource(resource)}
            onDragStart={(event) => onDragStart(event, resource.id)}
            onClick={() => onSelectResource(resource)}
            className={getCardClassName(active)}
          >
            <div className="flex items-start gap-3">
              {isLocalResource(resource) ? (
                <Checkbox
                  checked={checkedIds.includes(resource.id)}
                  onCheckedChange={() => onToggleChecked(resource.id)}
                  onClick={(event) => event.stopPropagation()}
                  className={getCheckboxClassName(active)}
                  aria-label={resource.name}
                />
              ) : null}
              <div className="min-w-0 flex-1">
                <div className={cn("truncate text-sm font-medium", active && "text-foreground")}>
                  {resource.name}
                </div>
                <div className={getSummaryClassName(active)}>{resource.summary}</div>
                {renderDiscoveryMeta(resource, isAllAgentsView, t, active)}
              </div>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    className={getActionButtonClassName(active)}
                    onClick={(event) => event.stopPropagation()}
                  >
                    <MoreHorizontal className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  {renderResourceAction(
                    resource,
                    onDeleteLocalSkill,
                    onOpenSkillEntryFile,
                    onOpenSkillFolder,
                    onSetLocalSkillEnabled,
                    onUpdateMarketplaceInstallState,
                    t
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
