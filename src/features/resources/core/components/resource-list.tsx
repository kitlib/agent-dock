import type { DragEvent } from "react";
import { Download, MoreHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn, formatInstallCount } from "@/lib/utils";
import type {
  AgentDiscoveryItem,
  LocalSkillCopySource,
  SkillResource,
} from "@/features/agents/types";
import { getLocalSkillToggleTarget } from "@/features/home/local-skill-toggle";
import { installStateKey } from "@/features/shared/constants";

type AgentResourceListProps = {
  checkedIds: string[];
  filteredResources: AgentDiscoveryItem[];
  showOriginBadge: boolean;
  isAllAgentsView: boolean;
  onCopySkill: (source: LocalSkillCopySource) => void;
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
  onInstallMarketplaceItem: (resource: AgentDiscoveryItem) => Promise<void>;
  selectedResourceId: string;
  t: (key: string, options?: Record<string, unknown>) => string;
};

function renderDiscoveryMeta(
  resource: AgentDiscoveryItem,
  showOriginBadge: boolean,
  isAllAgentsView: boolean,
  t: AgentResourceListProps["t"],
  active: boolean,
  formattedMarketplaceInstalls?: string
) {
  const isLocalSkill = resource.origin === "local" && resource.kind === "skill";
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
      {showOriginBadge ? <span className={badgeClassName}>{originLabel}</span> : null}
      {isAllAgentsView && isLocalSkill && resource.agentName ? (
        <span className={cn("rounded", badgeClassName)}>{resource.agentName}</span>
      ) : null}
      {isLocalSkill ? (
        !resource.enabled ? (
          <span className="rounded border border-amber-500/30 bg-amber-500/10 px-1.5 py-0.5 text-[9px] leading-3 text-amber-700 dark:text-amber-300">
            {t("prototype.actions.disabled")}
          </span>
        ) : null
      ) : null}
      {resource.origin === "marketplace" ? (
        <>
          <span>{resource.sourceLabel}</span>
          <span className="inline-flex items-center gap-1 leading-none">
            <Download className="relative top-[-0.5px] h-3 w-3 shrink-0" />
            {formattedMarketplaceInstalls ?? resource.installs}
          </span>
        </>
      ) : null}
    </div>
  );
}
function renderResourceAction(
  resource: AgentDiscoveryItem,
  onCopySkill: AgentResourceListProps["onCopySkill"],
  onDeleteLocalSkill: AgentResourceListProps["onDeleteLocalSkill"],
  onOpenSkillEntryFile: AgentResourceListProps["onOpenSkillEntryFile"],
  onOpenSkillFolder: AgentResourceListProps["onOpenSkillFolder"],
  onSetLocalSkillEnabled: AgentResourceListProps["onSetLocalSkillEnabled"],
  onInstallMarketplaceItem: AgentResourceListProps["onInstallMarketplaceItem"],
  t: AgentResourceListProps["t"]
) {
  if (resource.origin === "marketplace") {
    return (
      <DropdownMenuItem onClick={() => void onInstallMarketplaceItem(resource)}>
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
  const canCopySkill =
    isSkill &&
    skillPath.length > 0 &&
    entryFilePath.length > 0 &&
    (skillResource.ownerAgentId?.length ?? 0) > 0;
  const skillToggleTarget = getLocalSkillToggleTarget(resource);

  return (
    <>
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
      {isSkill && canEditSkill ? (
        <DropdownMenuItem onClick={() => void onOpenSkillEntryFile(skillPath, entryFilePath)}>
          {t("prototype.actions.edit")}
        </DropdownMenuItem>
      ) : null}
      {isSkill && canCopySkill ? (
        <DropdownMenuItem
          onClick={() =>
            onCopySkill({
              id: skillResource.id,
              name: skillResource.name,
              ownerAgentId: skillResource.ownerAgentId ?? "",
              sourceKind: skillResource.sourceKind ?? "skills",
              relativePath: skillResource.relativePath ?? "",
              skillPath,
              entryFilePath,
            })
          }
        >
          {t("prototype.actions.copy")}
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
      {isSkill && canDeleteSkill ? (
        <DropdownMenuItem
          className="text-destructive focus:text-destructive"
          onClick={() => void onDeleteLocalSkill(skillPath, entryFilePath, skillResource.id)}
        >
          {t("prototype.actions.delete")}
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

function getMarketplaceActionButtonClassName(active: boolean): string {
  return cn(
    "shrink-0",
    active ? "border-background/60 bg-background/90 hover:bg-background text-foreground" : ""
  );
}

export function AgentResourceList({
  checkedIds,
  filteredResources,
  showOriginBadge,
  isAllAgentsView,
  onCopySkill,
  onDeleteLocalSkill,
  onDragStart,
  onOpenSkillEntryFile,
  onOpenSkillFolder,
  onSelectResource,
  onSetLocalSkillEnabled,
  onToggleChecked,
  onInstallMarketplaceItem,
  selectedResourceId,
  t,
}: AgentResourceListProps) {
  const { i18n } = useTranslation();

  return (
    <div className="space-y-1">
      {filteredResources.map((resource) => {
        const active = isSelectedResource(resource.id, selectedResourceId);
        const isMarketplaceResource = resource.origin === "marketplace";
        const formattedInstalls =
          resource.origin === "marketplace"
            ? formatInstallCount(resource.installs, i18n.language)
            : null;

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
                {resource.origin === "local" ? (
                  <div className={getSummaryClassName(active)}>{resource.summary}</div>
                ) : null}
                {renderDiscoveryMeta(
                  resource,
                  showOriginBadge,
                  isAllAgentsView,
                  t,
                  active,
                  formattedInstalls ?? undefined
                )}
              </div>
              {isMarketplaceResource ? (
                <Button
                  variant={active ? "secondary" : "outline"}
                  size="xs"
                  className={cn("h-7 px-2 text-xs", getMarketplaceActionButtonClassName(active))}
                  onClick={(event) => {
                    event.stopPropagation();
                    void onInstallMarketplaceItem(resource);
                  }}
                >
                  {t(installStateKey[resource.installState])}
                </Button>
              ) : (
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
                      onCopySkill,
                      onDeleteLocalSkill,
                      onOpenSkillEntryFile,
                      onOpenSkillFolder,
                      onSetLocalSkillEnabled,
                      onInstallMarketplaceItem,
                      t
                    )}
                  </DropdownMenuContent>
                </DropdownMenu>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}
