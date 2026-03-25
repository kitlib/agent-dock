import { Button } from "@/components/ui/button";
import { AgentProviderIcon } from "./provider-icon";
import { AgentResourceDetail } from "./resource-detail";
import { installStateKey } from "./constants";
import type { AgentConflict, AgentDiscoveryItem, AgentSummary } from "./types";

type AgentDetailPanelProps = {
  conflicts?: AgentConflict[];
  emptyDescription?: string;
  emptyTitle?: string;
  onImportAgent?: (discoveryId: string) => void;
  onSetAgentEnabled?: (agentId: string, enabled: boolean) => void;
  onUpdateMarketplaceInstallState: (id: string) => void;
  selectedAgent: AgentSummary | null;
  selectedResource: AgentDiscoveryItem | null;
  t: (key: string, options?: Record<string, unknown>) => string;
};

export function AgentDetailPanel({
  conflicts = [],
  emptyDescription,
  emptyTitle,
  onImportAgent,
  onSetAgentEnabled,
  onUpdateMarketplaceInstallState,
  selectedAgent,
  selectedResource,
  t,
}: AgentDetailPanelProps) {
  const relatedConflicts = selectedAgent
    ? conflicts.filter((conflict) => conflict.agentFingerprints.includes(selectedAgent.fingerprint))
    : [];

  return (
    <div className="bg-muted/20 flex h-full min-w-0 flex-col overflow-hidden">
      <div className="border-b p-4">
        <div className="text-lg font-semibold break-words">
          {selectedResource?.name ??
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
            <span className="bg-muted rounded px-2 py-1">
              {t(installStateKey[selectedResource.installState])}
            </span>
          </div>
        ) : null}
        {selectedAgent ? (
          <div className="mt-3 space-y-3 text-xs">
            <div className="text-muted-foreground flex items-center gap-2">
              <AgentProviderIcon provider={selectedAgent.provider} size={16} />
              <span>
                {selectedAgent.name} · {selectedAgent.role}
              </span>
            </div>
            <div className="text-muted-foreground flex flex-wrap items-center gap-2">
              <span className="bg-muted rounded px-2 py-1">{selectedAgent.sourceScope}</span>
              <span className="bg-muted rounded px-2 py-1">{selectedAgent.statusLabel}</span>
              <span className="bg-muted rounded px-2 py-1">{selectedAgent.rootPath}</span>
            </div>
            {!selectedResource ? (
              <div className="flex flex-wrap items-center gap-2">
                {!selectedAgent.managed && onImportAgent ? (
                  <Button size="sm" onClick={() => onImportAgent(selectedAgent.discoveryId)}>
                    {t("prototype.actions.import")}
                  </Button>
                ) : null}
                {selectedAgent.managed && onSetAgentEnabled ? (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => onSetAgentEnabled(selectedAgent.id, !selectedAgent.enabled)}
                  >
                    {selectedAgent.enabled
                      ? t("prototype.actions.disable")
                      : t("prototype.actions.enable")}
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
                {selectedAgent.configPath ? <div>{selectedAgent.configPath}</div> : null}
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
            {relatedConflicts.length > 0 ? (
              <section className="space-y-2">
                <div className="font-medium">{t("prototype.detail.conflicts")}</div>
                {relatedConflicts.map((conflict) => (
                  <div
                    key={conflict.id}
                    className="rounded-lg border border-amber-500/30 bg-amber-500/5 p-3 text-xs"
                  >
                    <div className="font-medium">{conflict.summary}</div>
                    <div className="text-muted-foreground mt-1">{conflict.type}</div>
                  </div>
                ))}
              </section>
            ) : null}
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
