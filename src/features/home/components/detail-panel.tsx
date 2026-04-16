import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { AgentIcon } from "@/features/agents/components/agent-icon";
import type { AgentDiscoveryItem, AgentSummary, SkillResource } from "@/features/agents/types";
import {
  getLocalSkillDeleteTarget,
  getLocalSkillToggleTarget,
} from "@/features/home/local-skill-toggle";
import { AgentResourceDetail } from "@/features/resources/core/components/resource-detail";
import { installStateKey } from "@/features/shared/constants";

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
): (SkillResource & { origin: "local" }) | null {
  if (!isLocalSkillResource(selectedResource)) {
    return null;
  }

  return selectedResource as SkillResource & { origin: "local" };
}

function getSkillTitle(selectedResource: AgentDiscoveryItem | null): string | undefined {
  const skill = getLocalSkillResource(selectedResource);
  if (!skill) {
    return selectedResource?.name;
  }

  return skill.frontmatter?.name?.toString() ?? skill.frontmatter?.title?.toString() ?? skill.name;
}

function getOpenPath(selectedResource: AgentDiscoveryItem | null): string {
  const skill = getLocalSkillResource(selectedResource);
  if (!skill) {
    return "";
  }

  return skill.skillPath ?? skill.entryFilePath ?? "";
}

type AgentDetailPanelProps = {
  allAgentsDescription?: string;
  allAgentsSkillCount?: number;
  allAgentsTitle?: string;
  emptyDescription?: string;
  emptyTitle?: string;
  isAllAgentsView?: boolean;
  onDeleteLocalSkill?: (
    skillPath: string,
    entryFilePath: string,
    skillId?: string
  ) => Promise<void>;
  onOpenSkillEntryFile?: (skillPath: string, entryFilePath: string) => Promise<void>;
  onOpenSkillFolder: (skillPath: string) => void;
  onRefreshAgents?: () => void;
  onSetLocalSkillEnabled?: (
    skillPath: string,
    entryFilePath: string,
    enabled: boolean,
    skillId?: string
  ) => Promise<void>;
  onUpdateMarketplaceInstallState: (id: string) => void;
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
  onDeleteLocalSkill,
  onOpenSkillEntryFile,
  onOpenSkillFolder,
  onRefreshAgents,
  onSetLocalSkillEnabled,
  onUpdateMarketplaceInstallState,
  selectedAgent,
  selectedResource,
  t,
}: AgentDetailPanelProps) {
  const [isUpdatingSkillEnabled, setIsUpdatingSkillEnabled] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [isDeletingSkill, setIsDeletingSkill] = useState(false);
  const openPath = getOpenPath(selectedResource);
  const isLocalSkill = isLocalSkillResource(selectedResource);
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
    selectedResource?.summary ??
    (isAllAgentsView ? allAgentsDescription : undefined) ??
    selectedAgent?.summary ??
    emptyDescription ??
    t("prototype.emptySelection");
  const skillAndCommandCount = getAgentSkillAndCommandCount(selectedAgent);

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
            {skillDeleteTarget ? (
              <Button variant="destructive" size="sm" onClick={() => setIsDeleteDialogOpen(true)}>
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
          </div>
        </div>
        <div className="text-muted-foreground mt-1 text-sm">{description}</div>
        {selectedResource ? (
          <div className="text-muted-foreground mt-3 flex flex-wrap items-center gap-2 text-xs">
            <span className="bg-muted rounded px-2 py-1">
              {selectedResource.origin === "local"
                ? t("prototype.badges.local")
                : t("prototype.badges.marketplace")}
            </span>
            {selectedResource.origin === "marketplace" ? (
              <span className="bg-muted rounded px-2 py-1">
                {t(installStateKey[selectedResource.installState])}
              </span>
            ) : null}
            <span className="bg-muted rounded px-2 py-1">
              {t("prototype.detail.updatedAt")}: {selectedResource.updatedAt}
            </span>
            {isLocalSkill ? (
              <button
                type="button"
                className="bg-muted hover:bg-accent hover:text-accent-foreground cursor-pointer rounded px-2 py-1 break-all transition-colors"
                onClick={() => onOpenSkillFolder(openPath)}
              >
                {compactHomePath(openPath)}
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
      <div className="flex-1 overflow-auto p-4">
        {selectedResource ? (
          <AgentResourceDetail
            resource={selectedResource}
            onUpdateMarketplaceInstallState={onUpdateMarketplaceInstallState}
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
