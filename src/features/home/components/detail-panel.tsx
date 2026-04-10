import { useState } from "react";
import { Button } from "@/components/ui/button";
import { AgentIcon } from "@/features/agents/components/agent-icon";
import type { AgentDiscoveryItem, AgentSummary } from "@/features/agents/types";
import { getLocalSkillToggleTarget } from "@/features/home/local-skill-toggle";
import { AgentResourceDetail } from "@/features/resources/core/components/resource-detail";
import { installStateKey } from "@/features/shared/constants";

function compactHomePath(path: string | undefined): string | undefined {
  if (!path) {
    return path;
  }

  return path
    .replace(/^[A-Za-z]:[\\/]Users[\\/][^\\/]+/i, "~")
    .replace(/\.disabled$/i, "");
}

function isLocalSkillResource(selectedResource: AgentDiscoveryItem | null): boolean {
  return Boolean(
    selectedResource && selectedResource.kind === "skill" && selectedResource.origin === "local"
  );
}

function getSkillTitle(selectedResource: AgentDiscoveryItem | null): string | undefined {
  if (!isLocalSkillResource(selectedResource)) {
    return selectedResource?.name;
  }

  return (
    selectedResource.frontmatter?.name?.toString() ??
    selectedResource.frontmatter?.title?.toString() ??
    selectedResource.name
  );
}

function getOpenPath(selectedResource: AgentDiscoveryItem | null): string {
  if (!isLocalSkillResource(selectedResource)) {
    return "";
  }

  return selectedResource.skillPath ?? selectedResource.entryFilePath ?? "";
}


type AgentDetailPanelProps = {
  emptyDescription?: string;
  emptyTitle?: string;
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
  emptyDescription,
  emptyTitle,
  onOpenSkillFolder,
  onRefreshAgents,
  onSetLocalSkillEnabled,
  onUpdateMarketplaceInstallState,
  selectedAgent,
  selectedResource,
  t,
}: AgentDetailPanelProps) {
  const [isUpdatingSkillEnabled, setIsUpdatingSkillEnabled] = useState(false);
  const openPath = getOpenPath(selectedResource);
  const isLocalSkill = isLocalSkillResource(selectedResource);
  const skillToggleTarget = getLocalSkillToggleTarget(selectedResource);
  const title =
    getSkillTitle(selectedResource) ??
    selectedAgent?.alias ??
    selectedAgent?.name ??
    emptyTitle ??
    t("prototype.detail.title");
  const description =
    selectedResource?.summary ??
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

  return (
    <div className="bg-muted/20 flex h-full min-w-0 flex-col overflow-hidden">
      <div className="border-b p-4">
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0 flex-1 text-lg font-semibold break-words">
            {title}
            {skillToggleTarget && !skillToggleTarget.enabled && (
              <span className="ml-2 rounded bg-muted px-2 py-0.5 text-xs font-normal text-muted-foreground">
                {t("prototype.actions.disable")}
              </span>
            )}
          </div>
          {skillToggleTarget ? (
            <Button
              variant="outline"
              size="sm"
              className="shrink-0"
              disabled={isUpdatingSkillEnabled}
              onClick={handleToggleSkill}
            >
              {skillToggleTarget.enabled
                ? t("prototype.actions.disable")
                : t("prototype.actions.enable")}
            </Button>
          ) : null}
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
        {selectedAgent && !selectedResource ? (
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
      <div className="flex-1 overflow-auto p-4">
        {selectedResource ? (
          <AgentResourceDetail
            resource={selectedResource}
            onUpdateMarketplaceInstallState={onUpdateMarketplaceInstallState}
            t={t}
          />
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
