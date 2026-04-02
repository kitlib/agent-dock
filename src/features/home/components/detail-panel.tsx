import { Button } from "@/components/ui/button";
import { AgentIcon } from "@/features/agents/components/agent-icon";
import { AgentResourceDetail } from "@/features/resources/core/components/resource-detail";
import { installStateKey } from "@/features/shared/constants";
import type { AgentDiscoveryItem, AgentSummary } from "@/features/agents/types";

function compactHomePath(path: string | undefined) {
  if (!path) {
    return path;
  }

  return path.replace(/^[A-Za-z]:[\\/]Users[\\/][^\\/]+/i, "~");
}

function getSkillTitle(selectedResource: AgentDiscoveryItem | null) {
  if (!selectedResource || selectedResource.kind !== "skill" || selectedResource.origin !== "local") {
    return selectedResource?.name;
  }

  return (
    selectedResource.frontmatter?.name?.toString() ??
    selectedResource.frontmatter?.title?.toString() ??
    selectedResource.name
  );
}

function getOpenPath(selectedResource: AgentDiscoveryItem | null) {
  if (!selectedResource || selectedResource.kind !== "skill" || selectedResource.origin !== "local") {
    return "";
  }

  return selectedResource.entryFilePath ?? selectedResource.skillPath ?? "";
}

type AgentDetailPanelProps = {
  emptyDescription?: string;
  emptyTitle?: string;
  onOpenSkillFolder: (skillPath: string) => void;
  onRefreshAgents?: () => void;
  onUpdateMarketplaceInstallState: (id: string) => void;
  selectedAgent: AgentSummary | null;
  selectedResource: AgentDiscoveryItem | null;
  t: (key: string, options?: Record<string, unknown>) => string;
};

export function AgentDetailPanel({
  emptyDescription,
  emptyTitle,
  onOpenSkillFolder,
  onRefreshAgents,
  onUpdateMarketplaceInstallState,
  selectedAgent,
  selectedResource,
  t,
}: AgentDetailPanelProps) {
  const openPath = getOpenPath(selectedResource);

  return (
    <div className="bg-muted/20 flex h-full min-w-0 flex-col overflow-hidden">
      <div className="border-b p-4">
        <div className="text-lg font-semibold break-words">
          {getSkillTitle(selectedResource) ??
            selectedAgent?.alias ??
            selectedAgent?.name ??
            emptyTitle ??
            t("prototype.detail.title")}
        </div>
        <div className="text-muted-foreground mt-1 text-sm">
          {selectedResource?.summary ??
            selectedAgent?.summary ??
            emptyDescription ??
            t("prototype.emptySelection")}
        </div>
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
            {selectedResource.origin === "local" && selectedResource.kind === "skill" ? (
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
            {!selectedResource ? (
              <div className="flex flex-wrap items-center gap-2">
                {onRefreshAgents ? (
                  <Button variant="outline" size="sm" onClick={() => void onRefreshAgents()}>
                    {t("prototype.actions.retryScan")}
                  </Button>
                ) : null}
              </div>
            ) : null}
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
                <div className="mt-1 text-lg font-semibold">
                  {selectedAgent.resourceCounts.skill}
                </div>
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
