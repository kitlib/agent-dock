import { AgentResourceDetail } from "./resource-detail";
import { installStateKey } from "./constants";
import type { AgentDiscoveryItem, AgentSummary } from "./types";

type AgentDetailPanelProps = {
  selectedAgent: AgentSummary | null;
  selectedResource: AgentDiscoveryItem | null;
  onUpdateMarketplaceInstallState: (id: string) => void;
  t: (key: string, options?: Record<string, unknown>) => string;
};

export function AgentDetailPanel({
  selectedAgent,
  selectedResource,
  onUpdateMarketplaceInstallState,
  t,
}: AgentDetailPanelProps) {
  return (
    <div className="bg-muted/20 flex h-full min-w-0 flex-col overflow-hidden">
      <div className="border-b p-4">
        <div className="text-lg font-semibold break-words">
          {selectedResource?.name ?? t("prototype.detail.title")}
        </div>
        <div className="text-muted-foreground mt-1 text-sm">
          {selectedResource?.summary ?? selectedAgent?.summary ?? t("prototype.emptySelection")}
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
          <div className="text-muted-foreground mt-3 text-xs">
            {selectedAgent.name} · {selectedAgent.role}
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
        ) : (
          <div className="text-muted-foreground flex h-full items-center justify-center text-sm">
            {t("prototype.emptySelection")}
          </div>
        )}
      </div>
    </div>
  );
}
