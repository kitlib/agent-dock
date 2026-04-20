import { useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { InlineMetaBadge } from "@/components/inline-meta-badge";
import { AgentIcon } from "@/features/agents/components/agent-icon";
import type {
  AgentDiscoveryItem,
  AgentSummary,
  LocalDiscoveryItem,
  SkillResource,
} from "@/features/agents/types";
import {
  getLocalSkillDeleteTarget,
  getLocalSkillToggleTarget,
} from "@/features/home/local-skill-toggle";
import { AgentResourceDetail } from "@/features/resources/core/components/resource-detail";
import { installStateKey } from "@/features/shared/constants";
import { formatInstallCount } from "@/lib/utils";

function compactHomePath(path: string | undefined): string | undefined {
  if (!path) {
    return path;
  }

  return path.replace(/^[A-Za-z]:[\\/]Users[\\/][^\\/]+/i, "~").replace(/\.disabled$/i, "");
}

function isLocalSkillResource(selectedResource: AgentDiscoveryItem | null): boolean {
  return Boolean(
    selectedResource && selectedResource.kind === "skill" && selectedResource.origin === "local"
  );
}

function getLocalSkillResource(
  selectedResource: AgentDiscoveryItem | null
): (LocalDiscoveryItem & { kind: "skill"; origin: "local" }) | null {
  if (!isLocalSkillResource(selectedResource)) {
    return null;
  }

  return selectedResource as LocalDiscoveryItem & { kind: "skill"; origin: "local" };
}

function getSkillTitle(selectedResource: AgentDiscoveryItem | null): string | undefined {
  const skill = getLocalSkillResource(selectedResource);
  if (!skill) {
    return selectedResource?.name;
  }

  return skill.name;
}

function getOpenPath(selectedResource: AgentDiscoveryItem | null): string {
  const skill = getLocalSkillResource(selectedResource);
  if (!skill) {
    return "";
  }

  return skill.skillPath ?? skill.entryFilePath ?? "";
}

function getLocalMcpResource(
  selectedResource: AgentDiscoveryItem | null
): (LocalDiscoveryItem & { kind: "mcp"; origin: "local" }) | null {
  if (!selectedResource || selectedResource.origin !== "local" || selectedResource.kind !== "mcp") {
    return null;
  }

  return selectedResource as LocalDiscoveryItem & { kind: "mcp"; origin: "local" };
}

function getSelectedResourceSourceValue(
  selectedResource: AgentDiscoveryItem | null,
  t: AgentDetailPanelProps["t"]
): string | null {
  if (!selectedResource || selectedResource.origin !== "local") {
    return null;
  }

  if (
    selectedResource.kind === "skill" &&
    "marketplaceSource" in selectedResource &&
    selectedResource.marketplaceSource
  ) {
    return t("prototype.badges.marketplace");
  }

  return selectedResource.sourceLabel ?? null;
}

function getLocalMarketplaceUrl(selectedResource: AgentDiscoveryItem | null): string | null {
  if (
    !selectedResource ||
    selectedResource.origin !== "local" ||
    selectedResource.kind !== "skill" ||
    !("marketplaceSource" in selectedResource) ||
    !("marketplaceRemoteId" in selectedResource) ||
    !selectedResource.marketplaceSource ||
    !selectedResource.marketplaceRemoteId
  ) {
    return null;
  }

  return `https://skills.sh/${selectedResource.marketplaceSource}/${selectedResource.marketplaceRemoteId}`;
}

function getMarketplaceRepositoryUrl(selectedResource: AgentDiscoveryItem | null): string | null {
  if (
    !selectedResource ||
    selectedResource.origin !== "marketplace" ||
    !selectedResource.sourceLabel
  ) {
    return null;
  }

  return `https://github.com/${selectedResource.sourceLabel}`;
}

type AgentDetailPanelProps = {
  allAgentsDescription?: string;
  allAgentsSkillCount?: number;
  allAgentsTitle?: string;
  emptyDescription?: string;
  emptyTitle?: string;
  isAllAgentsView?: boolean;
  isMarketplaceDetailLoading?: boolean;
  isLocalMarketplaceDetailLoading?: boolean;
  onDeleteLocalSkill?: (
    skillPath: string,
    entryFilePath: string,
    skillId?: string
  ) => Promise<void>;
  onDeleteLocalMcp?: (agentType: string, configPath: string, serverName: string) => Promise<void>;
  onEditLocalMcp?: (resource: AgentDiscoveryItem) => void;
  onInspectMcp?: (mcp: LocalDiscoveryItem & { kind: "mcp"; origin: "local" }) => void;
  onOpenSkillEntryFile?: (skillPath: string, entryFilePath: string) => Promise<void>;
  onOpenSkillFolder: (skillPath: string) => void;
  onOpenMcpConfigFile?: (configPath: string) => Promise<void>;
  onOpenMcpConfigFolder?: (configPath: string) => void;
  onRefreshAgents?: () => void;
  onSetLocalSkillEnabled?: (
    skillPath: string,
    entryFilePath: string,
    enabled: boolean,
    skillId?: string
  ) => Promise<void>;
  onInstallMarketplaceItem: (resource: AgentDiscoveryItem) => Promise<void>;
  onUpdateLocalMarketplaceSkill?: (
    resource: LocalDiscoveryItem & { kind: "skill"; origin: "local" }
  ) => Promise<void>;
  selectedAgent: AgentSummary | null;
  selectedResource: AgentDiscoveryItem | null;
  t: (key: string, options?: Record<string, unknown>) => string;
};

function getAgentSkillAndCommandCount(selectedAgent: AgentSummary | null): number {
  if (!selectedAgent) {
    return 0;
  }

  return selectedAgent.resourceCounts.skill + selectedAgent.resourceCounts.command;
}

export function AgentDetailPanel({
  allAgentsDescription,
  allAgentsSkillCount = 0,
  allAgentsTitle,
  emptyDescription,
  emptyTitle,
  isAllAgentsView = false,
  isMarketplaceDetailLoading = false,
  isLocalMarketplaceDetailLoading = false,
  onDeleteLocalSkill,
  onDeleteLocalMcp,
  onEditLocalMcp,
  onOpenSkillEntryFile,
  onOpenSkillFolder,
  onOpenMcpConfigFile,
  onOpenMcpConfigFolder,
  onRefreshAgents,
  onSetLocalSkillEnabled,
  onInstallMarketplaceItem,
  onUpdateLocalMarketplaceSkill,
  onInspectMcp,
  selectedAgent,
  selectedResource,
  t,
}: AgentDetailPanelProps) {
  const { i18n } = useTranslation();
  const [isUpdatingSkillEnabled, setIsUpdatingSkillEnabled] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [isDeletingSkill, setIsDeletingSkill] = useState(false);
  const [isDeleteMcpDialogOpen, setIsDeleteMcpDialogOpen] = useState(false);
  const [isDeletingMcp, setIsDeletingMcp] = useState(false);
  const openPath = getOpenPath(selectedResource);
  const isLocalSkill = isLocalSkillResource(selectedResource);
  const localSkill = getLocalSkillResource(selectedResource);
  const localMcp = getLocalMcpResource(selectedResource);
  const canUpdateLocalMarketplaceSkill =
    localSkill?.marketplaceHasUpdate === true &&
    !isLocalMarketplaceDetailLoading &&
    onUpdateLocalMarketplaceSkill != null;
  const skillDeleteTarget = getLocalSkillDeleteTarget(selectedResource);
  const skillToggleTarget = getLocalSkillToggleTarget(selectedResource);
  const title =
    getSkillTitle(selectedResource) ??
    (isAllAgentsView ? allAgentsTitle : undefined) ??
    selectedAgent?.alias ??
    selectedAgent?.name ??
    emptyTitle ??
    t("prototype.detail.title");
  const description =
    selectedResource?.origin === "marketplace"
      ? null
      : (selectedResource?.summary ??
        (isAllAgentsView ? allAgentsDescription : undefined) ??
        selectedAgent?.summary ??
        emptyDescription ??
        t("prototype.emptySelection"));
  const skillAndCommandCount = getAgentSkillAndCommandCount(selectedAgent);
  const formattedMarketplaceInstalls =
    selectedResource?.origin === "marketplace"
      ? formatInstallCount(selectedResource.installs, i18n.language)
      : null;
  const localSourceValue = getSelectedResourceSourceValue(selectedResource, t);
  const localMarketplaceUrl = getLocalMarketplaceUrl(selectedResource);
  const marketplaceRepositoryUrl = getMarketplaceRepositoryUrl(selectedResource);

  const handleToggleSkill = async () => {
    if (!skillToggleTarget || !onSetLocalSkillEnabled) {
      return;
    }

    setIsUpdatingSkillEnabled(true);
    try {
      await onSetLocalSkillEnabled(
        skillToggleTarget.skillPath,
        skillToggleTarget.entryFilePath,
        !skillToggleTarget.enabled,
        skillToggleTarget.id
      );
    } catch (error) {
      console.error("Failed to update local skill enabled state:", error);
    } finally {
      setIsUpdatingSkillEnabled(false);
    }
  };

  const handleDeleteSkill = async () => {
    if (!skillDeleteTarget || !onDeleteLocalSkill) {
      return;
    }

    setIsDeletingSkill(true);
    try {
      await onDeleteLocalSkill(
        skillDeleteTarget.skillPath,
        skillDeleteTarget.entryFilePath,
        skillDeleteTarget.id
      );
      setIsDeleteDialogOpen(false);
    } catch (error) {
      console.error("Failed to delete local skill:", error);
    } finally {
      setIsDeletingSkill(false);
    }
  };

  const handleDeleteMcp = async () => {
    if (!localMcp?.agentType || !localMcp?.configPath || !onDeleteLocalMcp) {
      return;
    }

    setIsDeletingMcp(true);
    try {
      await onDeleteLocalMcp(localMcp.agentType, localMcp.configPath, localMcp.name);
      setIsDeleteMcpDialogOpen(false);
    } catch (error) {
      console.error("Failed to delete local MCP:", error);
    } finally {
      setIsDeletingMcp(false);
    }
  };

  return (
    <div className="bg-muted/20 flex h-full min-w-0 flex-col overflow-hidden">
      <div className="border-b p-4">
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0 flex-1 text-lg font-semibold break-words">
            {title}
            {skillToggleTarget && !skillToggleTarget.enabled && (
              <span className="ml-2 rounded border border-amber-500/30 bg-amber-500/10 px-1.5 py-0.5 text-[9px] leading-3 text-amber-700 dark:text-amber-300">
                {t("prototype.actions.disabled")}
              </span>
            )}
          </div>
          <div className="flex shrink-0 flex-wrap items-center justify-end gap-2">
            {selectedResource?.origin === "marketplace" ? (
              <>
                {selectedResource.url ? (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => void openUrl(selectedResource.url!)}
                  >
                    {t("prototype.actions.visit")}
                  </Button>
                ) : null}
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => void onInstallMarketplaceItem(selectedResource)}
                >
                  {t(installStateKey[selectedResource.installState])}
                </Button>
              </>
            ) : null}
            {canUpdateLocalMarketplaceSkill ? (
              <Button
                variant="outline"
                size="sm"
                onClick={() => localSkill && void onUpdateLocalMarketplaceSkill?.(localSkill)}
              >
                {t("prototype.actions.update")}
              </Button>
            ) : null}
            {isLocalSkill && openPath ? (
              <Button variant="outline" size="sm" onClick={() => onOpenSkillFolder(openPath)}>
                {t("prototype.actions.open")}
              </Button>
            ) : null}
            {isLocalSkill && onOpenSkillEntryFile ? (
              <Button
                variant="outline"
                size="sm"
                onClick={() => {
                  const skill = selectedResource as SkillResource & { origin: "local" };
                  void onOpenSkillEntryFile(openPath, skill.entryFilePath ?? "");
                }}
              >
                {t("prototype.actions.edit")}
              </Button>
            ) : null}
            {localMcp?.configPath && onOpenMcpConfigFile ? (
              <Button
                variant="outline"
                size="sm"
                onClick={() => void onOpenMcpConfigFile(localMcp.configPath!)}
              >
                {t("prototype.actions.openFile")}
              </Button>
            ) : null}
            {localMcp?.configPath && onInspectMcp ? (
              <Button variant="outline" size="sm" onClick={() => onInspectMcp?.(localMcp)}>
                {t("prototype.actions.inspect")}
              </Button>
            ) : null}
            {localMcp?.configPath && onOpenMcpConfigFile ? (
              <Button variant="outline" size="sm" onClick={() => onEditLocalMcp?.(localMcp)}>
                {t("prototype.actions.edit")}
              </Button>
            ) : null}
            {localMcp?.configPath && localMcp?.agentType && onDeleteLocalMcp ? (
              <Button
                variant="destructive"
                size="sm"
                onClick={() => setIsDeleteMcpDialogOpen(true)}
              >
                {t("prototype.actions.delete")}
              </Button>
            ) : null}
            {skillToggleTarget ? (
              <Button
                variant="outline"
                size="sm"
                disabled={isUpdatingSkillEnabled}
                onClick={handleToggleSkill}
              >
                {skillToggleTarget.enabled
                  ? t("prototype.actions.disable")
                  : t("prototype.actions.enable")}
              </Button>
            ) : null}
            {skillDeleteTarget ? (
              <Button variant="destructive" size="sm" onClick={() => setIsDeleteDialogOpen(true)}>
                {t("prototype.actions.delete")}
              </Button>
            ) : null}
          </div>
        </div>
        {description ? (
          <div className="text-muted-foreground mt-1 text-sm">{description}</div>
        ) : null}
        {selectedResource ? (
          <div className="text-muted-foreground mt-3 flex flex-wrap items-center gap-2 text-xs">
            {selectedResource.origin === "marketplace" ? (
              <>
                {marketplaceRepositoryUrl ? (
                  <button
                    type="button"
                    className="cursor-pointer transition-opacity hover:opacity-85"
                    onClick={() => void openUrl(marketplaceRepositoryUrl)}
                    title={marketplaceRepositoryUrl}
                  >
                    <InlineMetaBadge
                      label={t("prototype.detail.repository")}
                      value={selectedResource.sourceLabel}
                      tone="blue"
                    />
                  </button>
                ) : (
                  <InlineMetaBadge
                    label={t("prototype.detail.repository")}
                    value={selectedResource.sourceLabel}
                    tone="blue"
                  />
                )}
                <InlineMetaBadge
                  label={t("prototype.detail.installs")}
                  value={formattedMarketplaceInstalls ?? selectedResource.installs}
                  tone="amber"
                />
              </>
            ) : null}
            {selectedResource.origin === "local" && localSourceValue ? (
              localMarketplaceUrl ? (
                <button
                  type="button"
                  className="cursor-pointer transition-opacity hover:opacity-85"
                  onClick={() => void openUrl(localMarketplaceUrl)}
                  title={localMarketplaceUrl}
                >
                  <InlineMetaBadge
                    label={t("prototype.detail.source")}
                    value={localSourceValue}
                    tone="blue"
                  />
                </button>
              ) : (
                <InlineMetaBadge
                  label={t("prototype.detail.source")}
                  value={localSourceValue}
                  tone="blue"
                />
              )
            ) : null}
            {selectedResource.origin === "local" && selectedResource.kind !== "mcp" ? (
              <InlineMetaBadge
                label={t("prototype.detail.updatedAt")}
                value={selectedResource.updatedAt}
                tone="green"
              />
            ) : null}
            {isLocalSkill ? (
              <button
                type="button"
                className="cursor-pointer break-all transition-opacity hover:opacity-85"
                onClick={() => onOpenSkillFolder(openPath)}
                title={openPath}
              >
                <InlineMetaBadge
                  label={t("prototype.detail.rootPath")}
                  value={compactHomePath(openPath) ?? openPath}
                  tone="amber"
                  className="max-w-full"
                />
              </button>
            ) : null}
            {localMcp?.configPath && onOpenMcpConfigFolder ? (
              <button
                type="button"
                className="cursor-pointer break-all transition-opacity hover:opacity-85"
                onClick={() => onOpenMcpConfigFolder(localMcp.configPath!)}
                title={localMcp.configPath}
              >
                <InlineMetaBadge
                  label={t("prototype.detail.rootPath")}
                  value={compactHomePath(localMcp.configPath) ?? localMcp.configPath}
                  tone="amber"
                  className="max-w-full"
                />
              </button>
            ) : null}
          </div>
        ) : null}
        {isAllAgentsView && !selectedResource ? (
          <div className="mt-3 space-y-3 text-xs">
            <div className="text-muted-foreground flex flex-wrap items-center gap-2">
              <span className="bg-muted rounded px-2 py-1">{allAgentsTitle ?? title}</span>
              <span className="bg-muted rounded px-2 py-1">
                {t("prototype.tabs.skill")}: {allAgentsSkillCount}
              </span>
            </div>
            <div className="text-muted-foreground">
              {allAgentsDescription ?? t("prototype.detail.allAgentsDescription")}
            </div>
          </div>
        ) : null}
        {selectedAgent && !selectedResource && !isAllAgentsView ? (
          <div className="mt-3 space-y-3 text-xs">
            <div className="text-muted-foreground flex items-center gap-2">
              <AgentIcon agentType={selectedAgent.agentType} size={16} />
              <span>
                {selectedAgent.name} · {selectedAgent.role}
              </span>
            </div>
            <div className="text-muted-foreground flex flex-wrap items-center gap-2">
              <span className="bg-muted rounded px-2 py-1">{selectedAgent.statusLabel}</span>
              <span className="bg-muted rounded px-2 py-1">{selectedAgent.rootPath}</span>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              {onRefreshAgents ? (
                <Button variant="outline" size="sm" onClick={onRefreshAgents}>
                  {t("prototype.actions.retryScan")}
                </Button>
              ) : null}
            </div>
          </div>
        ) : null}
      </div>
      <Dialog open={isDeleteDialogOpen} onOpenChange={setIsDeleteDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("prototype.deleteSkill.title")}</DialogTitle>
            <DialogDescription>
              {t("prototype.deleteSkill.description", { name: title })}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsDeleteDialogOpen(false)}>
              {t("prototype.actions.cancel")}
            </Button>
            <Button
              variant="destructive"
              onClick={() => void handleDeleteSkill()}
              disabled={isDeletingSkill}
            >
              {t("prototype.actions.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <Dialog open={isDeleteMcpDialogOpen} onOpenChange={setIsDeleteMcpDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("prototype.deleteMcp.title")}</DialogTitle>
            <DialogDescription>
              {t("prototype.deleteMcp.description", { name: title })}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsDeleteMcpDialogOpen(false)}>
              {t("prototype.actions.cancel")}
            </Button>
            <Button
              variant="destructive"
              onClick={() => void handleDeleteMcp()}
              disabled={isDeletingMcp}
            >
              {t("prototype.actions.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <div className="flex-1 overflow-auto p-4">
        {selectedResource ? (
          <AgentResourceDetail
            isMarketplaceDetailLoading={isMarketplaceDetailLoading}
            resource={selectedResource}
            t={t}
          />
        ) : isAllAgentsView ? (
          <div className="space-y-4 text-sm">
            <div className="bg-background rounded-lg border p-4">
              <div className="font-medium">{allAgentsTitle ?? t("prototype.agents.all")}</div>
              <div className="text-muted-foreground mt-2 space-y-1 text-xs">
                <div>{allAgentsDescription ?? t("prototype.detail.allAgentsDescription")}</div>
              </div>
            </div>
            <div className="grid grid-cols-3 gap-3 text-xs">
              <div className="bg-background rounded-lg border p-3">
                <div className="text-muted-foreground">{t("prototype.tabs.skill")}</div>
                <div className="mt-1 text-lg font-semibold">{allAgentsSkillCount}</div>
              </div>
              <div className="bg-background rounded-lg border p-3">
                <div className="text-muted-foreground">{t("prototype.tabs.mcp")}</div>
                <div className="mt-1 text-lg font-semibold">-</div>
              </div>
              <div className="bg-background rounded-lg border p-3">
                <div className="text-muted-foreground">{t("prototype.tabs.subagent")}</div>
                <div className="mt-1 text-lg font-semibold">-</div>
              </div>
            </div>
          </div>
        ) : selectedAgent ? (
          <div className="space-y-4 text-sm">
            <div className="bg-background rounded-lg border p-4">
              <div className="font-medium">{t("prototype.detail.discoveryState")}</div>
              <div className="text-muted-foreground mt-2 space-y-1 text-xs">
                <div>{selectedAgent.summary}</div>
                <div>{selectedAgent.rootPath}</div>
              </div>
            </div>
            <div className="grid grid-cols-3 gap-3 text-xs">
              <div className="bg-background rounded-lg border p-3">
                <div className="text-muted-foreground">{t("prototype.tabs.skill")}</div>
                <div className="mt-1 text-lg font-semibold">{skillAndCommandCount}</div>
              </div>
              <div className="bg-background rounded-lg border p-3">
                <div className="text-muted-foreground">{t("prototype.tabs.mcp")}</div>
                <div className="mt-1 text-lg font-semibold">{selectedAgent.resourceCounts.mcp}</div>
              </div>
              <div className="bg-background rounded-lg border p-3">
                <div className="text-muted-foreground">{t("prototype.tabs.subagent")}</div>
                <div className="mt-1 text-lg font-semibold">
                  {selectedAgent.resourceCounts.subagent}
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div className="text-muted-foreground flex h-full items-center justify-center text-sm">
            {t("prototype.emptySelection")}
          </div>
        )}
      </div>
    </div>
  );
}
