import { memo, useEffect, useState } from "react";
import { Check, LoaderCircle, Plus, RefreshCw, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { cn } from "@/lib/utils";
import { AgentIcon } from "@/features/agents/components/agent-icon";
import { agentTypeMeta } from "@/features/agents/agent-meta";
import { useAgentImport } from "@/features/agents/use-agent-import";
import type {
  AgentManagementCard,
  CreateAgentResult,
  DeleteAgentResult,
  ImportAgentsResult,
  AgentTypeId,
  ManualAgentDraft,
  RemoveAgentResult,
} from "@/features/agents/types";

const agentTypeOptions = Object.keys(agentTypeMeta) as AgentTypeId[];

type AgentImportPanelProps = {
  managedAgentsForView: CreateAgentResult["resolvedAgents"];
  onCreateSuccess: (result: CreateAgentResult) => void;
  onDeleteSuccess: (result: DeleteAgentResult) => void;
  onImportSuccess: (result: ImportAgentsResult) => void;
  onRemoveSuccess: (result: RemoveAgentResult) => void;
  t: (key: string, options?: Record<string, unknown>) => string;
};

type AgentCardItemProps = {
  candidate: AgentManagementCard;
  isImporting: boolean;
  onToggle: (candidate: AgentManagementCard) => Promise<void>;
  onDelete: (candidate: AgentManagementCard) => Promise<void>;
};

function candidateStateLabel(candidate: AgentManagementCard): string | null {
  if (candidate.state === "unreadable") {
    return candidate.reason ?? candidate.state;
  }

  return null;
}

function candidateSkillResourceCount(candidate: AgentManagementCard): number {
  const { skill, command } = candidate.resourceCounts;

  return skill + command;
}

const AgentManagementCardItem = memo(function AgentManagementCardItem({
  candidate,
  isImporting,
  onToggle,
  onDelete,
}: AgentCardItemProps) {
  const isReady = candidate.state === "ready";
  const isImported = candidate.state === "imported";
  const canToggle = isImported || isReady;
  const stateLabel = candidateStateLabel(candidate);

  return (
    <div
      role={canToggle ? "button" : undefined}
      tabIndex={canToggle ? 0 : undefined}
      onClick={(event) => {
        if (!canToggle) {
          return;
        }
        event.preventDefault();
        event.stopPropagation();
        void onToggle(candidate);
      }}
      onKeyDown={(event) => {
        if (!canToggle) {
          return;
        }
        if (event.key !== "Enter" && event.key !== " ") {
          return;
        }
        event.preventDefault();
        event.stopPropagation();
        void onToggle(candidate);
      }}
      aria-disabled={isImporting || !canToggle}
      className={cn(
        "w-full rounded-lg border p-2 text-left transition-all",
        canToggle
          ? "border-border bg-card hover:border-border hover:bg-accent/20 focus-visible:ring-ring cursor-pointer focus-visible:ring-2 focus-visible:outline-none"
          : "border-border bg-muted/60 cursor-default"
      )}
    >
      <div className="flex items-stretch gap-2.5">
        <div className="bg-background flex h-9 w-9 shrink-0 items-center justify-center self-start rounded-lg border">
          <AgentIcon agentType={candidate.agentType} size={18} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm leading-5 font-medium">{candidate.displayName}</div>
          <div className="text-muted-foreground mt-1 text-[11px]">{candidate.rootPath}</div>
          <div className="text-muted-foreground mt-1 flex flex-wrap items-center gap-1.5 text-xs">
            <span className="bg-muted rounded px-1.5 py-0.5 text-[10px]">
              {candidateSkillResourceCount(candidate)} Skills
            </span>
            <span className="bg-muted rounded px-1.5 py-0.5 text-[10px]">
              {candidate.resourceCounts.mcp} MCP
            </span>
            <span className="bg-muted rounded px-1.5 py-0.5 text-[10px]">
              {candidate.resourceCounts.subagent} Subagents
            </span>
          </div>
          {stateLabel ? (
            <div className="text-muted-foreground mt-1 text-[11px]">{stateLabel}</div>
          ) : null}
        </div>
        <div className="flex min-h-full shrink-0 flex-col items-center justify-between self-stretch">
          <div className="flex h-5 w-5 items-center justify-center">
            {isImported ? (
              <span className="flex h-5 w-5 items-center justify-center rounded-full bg-emerald-500 text-white">
                <Check className="h-3.5 w-3.5" />
              </span>
            ) : null}
          </div>
          <div className="flex h-7 w-7 items-center justify-center">
            {candidate.deletable ? (
              <Button
                type="button"
                variant="ghost"
                size="icon-sm"
                className="h-7 w-7 shrink-0 rounded-md border border-red-500/50 p-0 text-red-600 hover:text-red-600 dark:border-red-400/60 dark:text-red-400 dark:hover:text-red-400"
                onClick={(event) => {
                  event.preventDefault();
                  event.stopPropagation();
                  void onDelete(candidate);
                }}
                title="Delete"
                disabled={isImporting}
              >
                <Trash2 className="h-3.5 w-3.5" />
              </Button>
            ) : null}
          </div>
        </div>
      </div>
    </div>
  );
});

export function AgentImportPanel({
  managedAgentsForView,
  onCreateSuccess,
  onDeleteSuccess,
  onImportSuccess,
  onRemoveSuccess,
  t,
}: AgentImportPanelProps) {
  const [isManualDialogOpen, setIsManualDialogOpen] = useState(false);
  const {
    canSubmitManual,
    enterImporting,
    isCreatingManually,
    isImporting,
    isScanning,
    managementCards,
    manualDraft,
    resetImportState,
    resetManualDraft,
    runScan,
    scanError,
    submitManualAdd,
    toggleCandidate,
    deleteCandidate,
    updateManualDraft,
  } = useAgentImport({
    managedAgentsForView,
    onCreateSuccess,
    onDeleteSuccess,
    onImportSuccess,
    onRemoveSuccess,
  });
  const selectedAgentTypeMeta = agentTypeMeta[manualDraft.agentType];

  useEffect(() => {
    void enterImporting();

    return () => {
      resetImportState();
      setIsManualDialogOpen(false);
    };
  }, [enterImporting, resetImportState]);

  const handleManualDialogChange = (open: boolean) => {
    setIsManualDialogOpen(open);
    if (!open) {
      resetManualDraft();
    }
  };

  const openManualDialog = () => {
    resetManualDraft();
    setIsManualDialogOpen(true);
  };

  const handleSubmitManualAdd = async () => {
    try {
      await submitManualAdd();
      setIsManualDialogOpen(false);
    } catch {
      // Keep dialog open so the user can retry after the error is shown elsewhere.
    }
  };

  return (
    <div className="flex h-full min-w-0 flex-col overflow-hidden p-4">
      <div className="mb-3 flex items-start justify-between gap-3">
        <div>
          <div className="text-lg font-semibold">{t("prototype.actions.add")}</div>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={openManualDialog}>
            <Plus className="h-4 w-4" />
            {t("prototype.actions.manualAdd")}
          </Button>
          <Button variant="outline" size="sm" onClick={() => void runScan()} disabled={isScanning}>
            {isScanning ? (
              <LoaderCircle className="h-4 w-4 animate-spin" />
            ) : (
              <RefreshCw className="h-4 w-4" />
            )}
            {t("prototype.actions.retryScan")}
          </Button>
        </div>
      </div>

      <div className="grid min-h-0 flex-1 overflow-hidden">
        <section className="min-h-0 rounded-lg border">
          <Dialog open={isManualDialogOpen} onOpenChange={handleManualDialogChange}>
            <DialogContent className="sm:max-w-xl">
              <DialogHeader>
                <DialogTitle>{t("prototype.actions.manualAdd")}</DialogTitle>
              </DialogHeader>

              <div className="grid grid-cols-2 gap-2.5">
                <label className="col-span-2 flex flex-col gap-1.5 text-sm">
                  <span>{t("prototype.detail.selectAgentType")}</span>
                  <Select
                    value={manualDraft.agentType}
                    onValueChange={(value) =>
                      updateManualDraft("agentType", value as ManualAgentDraft["agentType"])
                    }
                  >
                    <SelectTrigger className="w-full">
                      <SelectValue>
                        <div className="flex items-center gap-2">
                          <AgentIcon agentType={manualDraft.agentType} size={16} />
                          <span>{selectedAgentTypeMeta.name}</span>
                        </div>
                      </SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                      {agentTypeOptions.map((agentType) => (
                        <SelectItem key={agentType} value={agentType}>
                          <div className="flex items-center gap-2">
                            <AgentIcon agentType={agentType} size={16} />
                            <span>{agentTypeMeta[agentType].name}</span>
                          </div>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </label>

                <label className="col-span-2 flex flex-col gap-1.5 text-sm">
                  <span>{t("prototype.detail.name")}</span>
                  <Input
                    value={manualDraft.name}
                    onChange={(event) => updateManualDraft("name", event.target.value)}
                    placeholder={selectedAgentTypeMeta.name}
                  />
                </label>

                <label className="col-span-2 flex flex-col gap-1.5 text-sm">
                  <span>{t("prototype.detail.rootPath")}</span>
                  <Input
                    value={manualDraft.rootPath}
                    onChange={(event) => updateManualDraft("rootPath", event.target.value)}
                    placeholder={selectedAgentTypeMeta.directory}
                  />
                </label>
              </div>

              <DialogFooter>
                <Button variant="outline" onClick={() => handleManualDialogChange(false)}>
                  {t("prototype.actions.cancel")}
                </Button>
                <Button
                  onClick={() => void handleSubmitManualAdd()}
                  disabled={!canSubmitManual || isCreatingManually}
                >
                  {isCreatingManually ? (
                    <LoaderCircle className="h-4 w-4 animate-spin" />
                  ) : (
                    <Plus className="h-4 w-4" />
                  )}
                  {t("prototype.actions.manualAdd")}
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>

          <div className="h-full overflow-auto p-2">
            {scanError ? (
              <div className="text-destructive rounded-lg border border-dashed p-4 text-sm">
                {scanError}
              </div>
            ) : null}

            {!scanError && managementCards.length === 0 ? (
              <div className="text-muted-foreground flex h-40 items-center justify-center text-sm">
                {isScanning ? t("prototype.detail.scanningAgents") : t("prototype.emptyList")}
              </div>
            ) : (
              <div className="grid grid-cols-2 gap-2 xl:grid-cols-3 2xl:grid-cols-4">
                {managementCards.map((candidate) => (
                  <AgentManagementCardItem
                    key={candidate.id}
                    candidate={candidate}
                    isImporting={isImporting}
                    onToggle={toggleCandidate}
                    onDelete={deleteCandidate}
                  />
                ))}
              </div>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
