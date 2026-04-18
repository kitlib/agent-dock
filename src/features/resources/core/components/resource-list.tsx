import { memo, useEffect, useMemo, type DragEvent, type RefObject } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
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
import type {
  AgentDiscoveryItem,
  LocalSkillCopySource,
  McpResource,
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
  onDeleteLocalMcp: (agentType: string, configPath: string, serverName: string) => Promise<void>;
  onDragStart: (event: DragEvent<HTMLDivElement>, resourceId: string) => void;
  onOpenSkillEntryFile: (skillPath: string, entryFilePath: string) => Promise<void>;
  onOpenSkillFolder: (skillPath: string) => void;
  onOpenMcpConfigFile: (configPath: string) => Promise<void>;
  onOpenMcpConfigFolder: (configPath: string) => void;
  onSelectResource: (resource: AgentDiscoveryItem) => void;
  onSetLocalSkillEnabled: (
    skillPath: string,
    entryFilePath: string,
    enabled: boolean,
    skillId?: string
  ) => Promise<void>;
  onToggleChecked: (id: string) => void;
  onInstallMarketplaceItem: (resource: AgentDiscoveryItem) => Promise<void>;
  scrollContainerRef: RefObject<HTMLDivElement | null>;
  selectedResourceId: string;
  t: (key: string, options?: Record<string, unknown>) => string;
};

const VIRTUALIZATION_THRESHOLD = 100;
const VIRTUAL_ROW_GAP = 4;
const VIRTUAL_LOCAL_ROW_HEIGHT = 84;
const VIRTUAL_MARKETPLACE_ROW_HEIGHT = 64;

type DisplayResource = {
  resource: AgentDiscoveryItem;
  formattedMarketplaceInstalls?: string;
  isLocalResource: boolean;
  isMarketplaceResource: boolean;
  isLocalSkill: boolean;
  originLabel: string;
  agentBadgeLabel?: string;
  isDisabled: boolean;
  skillPath?: string;
  entryFilePath?: string;
  configPath?: string;
  copySource?: LocalSkillCopySource;
  skillToggleTarget?: ReturnType<typeof getLocalSkillToggleTarget>;
};

type AgentResourceRowProps = {
  active: boolean;
  checked: boolean;
  display: DisplayResource;
  onCopySkill: AgentResourceListProps["onCopySkill"];
  onDeleteLocalSkill: AgentResourceListProps["onDeleteLocalSkill"];
  onDeleteLocalMcp: AgentResourceListProps["onDeleteLocalMcp"];
  onDragStart: AgentResourceListProps["onDragStart"];
  onOpenSkillEntryFile: AgentResourceListProps["onOpenSkillEntryFile"];
  onOpenSkillFolder: AgentResourceListProps["onOpenSkillFolder"];
  onOpenMcpConfigFile: AgentResourceListProps["onOpenMcpConfigFile"];
  onOpenMcpConfigFolder: AgentResourceListProps["onOpenMcpConfigFolder"];
  onSelectResource: AgentResourceListProps["onSelectResource"];
  onSetLocalSkillEnabled: AgentResourceListProps["onSetLocalSkillEnabled"];
  onToggleChecked: AgentResourceListProps["onToggleChecked"];
  onInstallMarketplaceItem: AgentResourceListProps["onInstallMarketplaceItem"];
  showOriginBadge: boolean;
  t: AgentResourceListProps["t"];
};

function renderDiscoveryMeta(
  display: DisplayResource,
  showOriginBadge: boolean,
  active: boolean,
  formattedMarketplaceInstalls?: string
) {
  const badgeClassName = active
    ? "border border-border/70 bg-background/85 px-1.5 py-0.5 text-[9px] leading-3 text-foreground"
    : "bg-muted px-1.5 py-0.5 text-[9px] leading-3 text-muted-foreground";

  return (
    <div
      className={cn(
        "mt-1 flex flex-wrap items-center gap-2 text-xs",
        active ? "text-foreground/75" : "text-muted-foreground"
      )}
    >
      {showOriginBadge ? <span className={badgeClassName}>{display.originLabel}</span> : null}
      {display.agentBadgeLabel ? (
        <span className={cn("rounded", badgeClassName)}>{display.agentBadgeLabel}</span>
      ) : null}
      {display.isDisabled ? (
        <span className="rounded border border-amber-500/30 bg-amber-500/10 px-1.5 py-0.5 text-[9px] leading-3 text-amber-700 dark:text-amber-300">
          Disabled
        </span>
      ) : null}
      {display.isMarketplaceResource ? (
        <>
          <span>{display.resource.sourceLabel}</span>
          <span className="inline-flex items-center gap-1 leading-none">
            <Download className="relative top-[-0.5px] h-3 w-3 shrink-0" />
            {formattedMarketplaceInstalls ?? display.resource.installs}
          </span>
        </>
      ) : null}
    </div>
  );
}

function renderResourceAction(
  display: DisplayResource,
  onCopySkill: AgentResourceListProps["onCopySkill"],
  onDeleteLocalSkill: AgentResourceListProps["onDeleteLocalSkill"],
  onDeleteLocalMcp: AgentResourceListProps["onDeleteLocalMcp"],
  onOpenSkillEntryFile: AgentResourceListProps["onOpenSkillEntryFile"],
  onOpenSkillFolder: AgentResourceListProps["onOpenSkillFolder"],
  onOpenMcpConfigFile: AgentResourceListProps["onOpenMcpConfigFile"],
  onOpenMcpConfigFolder: AgentResourceListProps["onOpenMcpConfigFolder"],
  onSetLocalSkillEnabled: AgentResourceListProps["onSetLocalSkillEnabled"],
  onInstallMarketplaceItem: AgentResourceListProps["onInstallMarketplaceItem"],
  t: AgentResourceListProps["t"]
) {
  if (display.isMarketplaceResource) {
    return (
      <DropdownMenuItem onClick={() => void onInstallMarketplaceItem(display.resource)}>
        {t(installStateKey[display.resource.installState])}
      </DropdownMenuItem>
    );
  }

  const canOpenSkillFolder = (display.skillPath?.length ?? 0) > 0;
  const canEditSkill =
    display.isLocalSkill &&
    (display.skillPath?.length ?? 0) > 0 &&
    (display.entryFilePath?.length ?? 0) > 0;
  const canDeleteSkill = display.isLocalSkill && (display.skillPath?.length ?? 0) > 0;
  const canCopySkill = display.copySource != null;
  const canOpenMcpFolder = !display.isLocalSkill && (display.configPath?.length ?? 0) > 0;
  const localMcpResource =
    !display.isLocalSkill && display.resource.origin === "local" && display.resource.kind === "mcp"
      ? (display.resource as McpResource & { origin: "local" })
      : null;
  const canDeleteMcp =
    localMcpResource != null &&
    (display.configPath?.length ?? 0) > 0 &&
    (localMcpResource.agentType?.length ?? 0) > 0;

  return (
    <>
      {display.isLocalSkill ? (
        <DropdownMenuItem
          disabled={!canOpenSkillFolder}
          onClick={() => {
            if (canOpenSkillFolder && display.skillPath) {
              onOpenSkillFolder(display.skillPath);
            }
          }}
        >
          {t("prototype.actions.open")}
        </DropdownMenuItem>
      ) : null}
      {canEditSkill && display.skillPath && display.entryFilePath ? (
        <DropdownMenuItem
          onClick={() => void onOpenSkillEntryFile(display.skillPath!, display.entryFilePath!)}
        >
          {t("prototype.actions.edit")}
        </DropdownMenuItem>
      ) : null}
      {canOpenMcpFolder ? (
        <>
          <DropdownMenuItem onClick={() => onOpenMcpConfigFolder(display.configPath!)}>
            {t("prototype.actions.open")}
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => void onOpenMcpConfigFile(display.configPath!)}>
            {t("prototype.actions.edit")}
          </DropdownMenuItem>
          {canDeleteMcp ? (
            <DropdownMenuItem
              className="text-destructive focus:text-destructive"
              onClick={() =>
                void onDeleteLocalMcp(
                  localMcpResource?.agentType ?? "",
                  display.configPath!,
                  display.resource.name
                )
              }
            >
              {t("prototype.actions.delete")}
            </DropdownMenuItem>
          ) : null}
        </>
      ) : null}
      {canCopySkill ? (
        <DropdownMenuItem onClick={() => onCopySkill(display.copySource!)}>
          {t("prototype.actions.copy")}
        </DropdownMenuItem>
      ) : null}
      {display.skillToggleTarget ? (
        <DropdownMenuItem
          onClick={() =>
            void onSetLocalSkillEnabled(
              display.skillToggleTarget!.skillPath,
              display.skillToggleTarget!.entryFilePath,
              !display.skillToggleTarget!.enabled,
              display.skillToggleTarget!.id
            )
          }
        >
          {display.skillToggleTarget.enabled
            ? t("prototype.actions.disable")
            : t("prototype.actions.enable")}
        </DropdownMenuItem>
      ) : null}
      {canDeleteSkill && display.skillPath ? (
        <DropdownMenuItem
          className="text-destructive focus:text-destructive"
          onClick={() =>
            void onDeleteLocalSkill(
              display.skillPath!,
              display.entryFilePath ?? "",
              display.resource.id
            )
          }
        >
          {t("prototype.actions.delete")}
        </DropdownMenuItem>
      ) : null}
    </>
  );
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

const AgentResourceRow = memo(function AgentResourceRow({
  active,
  checked,
  display,
  onCopySkill,
  onDeleteLocalSkill,
  onDeleteLocalMcp,
  onDragStart,
  onOpenSkillEntryFile,
  onOpenSkillFolder,
  onOpenMcpConfigFile,
  onOpenMcpConfigFolder,
  onSelectResource,
  onSetLocalSkillEnabled,
  onToggleChecked,
  onInstallMarketplaceItem,
  showOriginBadge,
  t,
}: AgentResourceRowProps) {
  const { resource, formattedMarketplaceInstalls, isLocalResource, isMarketplaceResource } =
    display;

  return (
    <div
      draggable={isLocalResource}
      onDragStart={(event) => onDragStart(event, resource.id)}
      onClick={() => onSelectResource(resource)}
      className={getCardClassName(active)}
    >
      <div className="flex items-start gap-3">
        {isLocalResource ? (
          <Checkbox
            checked={checked}
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
            display,
            showOriginBadge,
            active,
            formattedMarketplaceInstalls
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
                display,
                onCopySkill,
                onDeleteLocalSkill,
                onDeleteLocalMcp,
                onOpenSkillEntryFile,
                onOpenSkillFolder,
                onOpenMcpConfigFile,
                onOpenMcpConfigFolder,
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
});

function buildDisplayResource(
  resource: AgentDiscoveryItem,
  showOriginBadge: boolean,
  isAllAgentsView: boolean,
  t: AgentResourceListProps["t"]
): DisplayResource {
  const isLocalResource = resource.origin === "local";
  const isMarketplaceResource = resource.origin === "marketplace";
  const isLocalSkill = isLocalResource && resource.kind === "skill";
  const originLabel = isLocalResource
    ? t("prototype.badges.local")
    : t("prototype.badges.marketplace");
  const formattedMarketplaceInstalls = resource.formattedInstalls;

  if (!isLocalSkill) {
    const mcpResource =
      isLocalResource && resource.kind === "mcp" ? (resource as McpResource & { origin: "local" }) : null;
    return {
      resource,
      formattedMarketplaceInstalls,
      isLocalResource,
      isMarketplaceResource,
      isLocalSkill,
      originLabel,
      agentBadgeLabel:
        showOriginBadge && isAllAgentsView && "agentName" in resource ? resource.agentName : undefined,
      isDisabled: false,
      configPath: mcpResource?.configPath ?? "",
    };
  }

  const skillResource = resource as SkillResource & { origin: "local" };
  const skillPath = skillResource.skillPath?.trim() ?? "";
  const entryFilePath = skillResource.entryFilePath ?? "";
  const copySource =
    skillPath.length > 0 &&
    entryFilePath.length > 0 &&
    (skillResource.ownerAgentId?.length ?? 0) > 0
      ? {
          id: skillResource.id,
          name: skillResource.name,
          ownerAgentId: skillResource.ownerAgentId ?? "",
          sourceKind: skillResource.sourceKind ?? "skills",
          relativePath: skillResource.relativePath ?? "",
          skillPath,
          entryFilePath,
        }
      : undefined;

  return {
    resource,
    formattedMarketplaceInstalls,
    isLocalResource,
    isMarketplaceResource,
    isLocalSkill,
    originLabel,
    agentBadgeLabel:
      showOriginBadge && isAllAgentsView && resource.agentName ? resource.agentName : undefined,
    isDisabled: !resource.enabled,
    skillPath,
    entryFilePath,
    copySource,
    skillToggleTarget: getLocalSkillToggleTarget(resource),
  };
}

function estimateDisplayResourceHeight(display: DisplayResource | undefined): number {
  if (!display) {
    return VIRTUAL_LOCAL_ROW_HEIGHT;
  }

  return display.isMarketplaceResource ? VIRTUAL_MARKETPLACE_ROW_HEIGHT : VIRTUAL_LOCAL_ROW_HEIGHT;
}

export function AgentResourceList({
  checkedIds,
  filteredResources,
  showOriginBadge,
  isAllAgentsView,
  onCopySkill,
  onDeleteLocalSkill,
  onDeleteLocalMcp,
  onDragStart,
  onOpenSkillEntryFile,
  onOpenSkillFolder,
  onOpenMcpConfigFile,
  onOpenMcpConfigFolder,
  onSelectResource,
  onSetLocalSkillEnabled,
  onToggleChecked,
  onInstallMarketplaceItem,
  scrollContainerRef,
  selectedResourceId,
  t,
}: AgentResourceListProps) {
  const checkedIdSet = useMemo(() => new Set(checkedIds), [checkedIds]);
  const displayResources = useMemo(
    () =>
      filteredResources.map((resource) =>
        buildDisplayResource(resource, showOriginBadge, isAllAgentsView, t)
      ),
    [filteredResources, showOriginBadge, isAllAgentsView, t]
  );
  const shouldVirtualize = filteredResources.length >= VIRTUALIZATION_THRESHOLD;

  // TanStack Virtual exposes imperative instance methods by design.
  // This integration stays local to the list component and does not rely on React Compiler memoization.
  // eslint-disable-next-line react-hooks/incompatible-library
  const rowVirtualizer = useVirtualizer({
    count: displayResources.length,
    estimateSize: (index) => estimateDisplayResourceHeight(displayResources[index]),
    gap: VIRTUAL_ROW_GAP,
    getItemKey: (index) => displayResources[index]?.resource.id ?? index,
    getScrollElement: () => scrollContainerRef.current,
    overscan: 4,
    useFlushSync: false,
  });
  const virtualRows = rowVirtualizer.getVirtualItems();

  useEffect(() => {
    if (!shouldVirtualize) {
      return;
    }

    rowVirtualizer.measure();
  }, [displayResources, rowVirtualizer, shouldVirtualize]);

  useEffect(() => {
    if (!shouldVirtualize) {
      return;
    }

    const selectedIndex = filteredResources.findIndex(
      (resource) => resource.id === selectedResourceId
    );
    if (selectedIndex < 0) {
      return;
    }

    rowVirtualizer.scrollToIndex(selectedIndex, { align: "auto" });
  }, [filteredResources, rowVirtualizer, selectedResourceId, shouldVirtualize]);

  if (!shouldVirtualize) {
    return (
      <div className="space-y-1">
        {displayResources.map((display) => (
          <AgentResourceRow
            key={display.resource.id}
            active={display.resource.id === selectedResourceId}
            checked={checkedIdSet.has(display.resource.id)}
            display={display}
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
            showOriginBadge={showOriginBadge}
            t={t}
          />
        ))}
      </div>
    );
  }

  return (
    <div
      className="relative w-full"
      style={{
        height: `${rowVirtualizer.getTotalSize()}px`,
      }}
    >
      {virtualRows.map((virtualRow) => {
        const item = displayResources[virtualRow.index];
        if (!item) {
          return null;
        }
        const display = item;
        const { resource } = display;

        return (
          <div
            key={virtualRow.key}
            data-index={virtualRow.index}
            ref={rowVirtualizer.measureElement}
            className="absolute top-0 left-0 w-full"
            style={{
              transform: `translateY(${virtualRow.start}px)`,
            }}
          >
            <AgentResourceRow
              active={resource.id === selectedResourceId}
              checked={checkedIdSet.has(resource.id)}
              display={display}
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
              showOriginBadge={showOriginBadge}
              t={t}
            />
          </div>
        );
      })}
    </div>
  );
}
