import { useEffect, useState } from "react";
import { Check, LoaderCircle, Plus, RefreshCw, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
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
import { AgentProviderIcon } from "./provider-icon";
import { useAgentImport } from "./use-agent-import";
import type {
  AgentManagementCard,
  CreateAgentResult,
  DeleteAgentResult,
  ImportAgentsResult,
  ManualAgentDraft,
  RemoveAgentResult,
} from "./types";

const providerOptions: ManualAgentDraft["provider"][] = [
  "claude",
  "cursor",
  "codex",
  "antigravity",
];

type AgentImportPanelProps = {
  managedAgentsForView: CreateAgentResult["resolvedAgents"];
  onCreateSuccess: (result: CreateAgentResult) => void;
  onDeleteSuccess: (result: DeleteAgentResult) => void;
  onImportSuccess: (result: ImportAgentsResult) => void;
  onRemoveSuccess: (result: RemoveAgentResult) => void;
  t: (key: string, options?: Record<string, unknown>) => string;
};

function candidateStateLabel(candidate: AgentManagementCard, t: AgentImportPanelProps["t"]) {
  if (candidate.state === "conflict") {
    return candidate.reason ?? t("prototype.detail.conflicts");
  }

  if (candidate.state === "unreadable") {
    return candidate.reason ?? candidate.state;
  }

  return null;
}

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
    await submitManualAdd();
    setIsManualDialogOpen(false);
  };

  return (
    <div className="flex h-full min-w-0 flex-col overflow-hidden p-4">
      <div className="mb-3 flex items-start justify-between gap-3">
        <div>
          <div className="text-lg font-semibold">{t("prototype.actions.add")}</div>
          <div className="text-muted-foreground mt-1 text-sm">
            {isScanning ? t("prototype.detail.scanningAgents") : t("prototype.detail.scanOrCreate")}
          </div>
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
                <DialogDescription>{t("prototype.detail.scanOrCreate")}</DialogDescription>
              </DialogHeader>

              <div className="grid grid-cols-2 gap-2.5">
                <label className="col-span-2 flex flex-col gap-1.5 text-sm">
                  <span>{t("prototype.detail.selectProvider")}</span>
                  <Select
                    value={manualDraft.provider}
                    onValueChange={(value) =>
                      updateManualDraft("provider", value as ManualAgentDraft["provider"])
                    }
                  >
                    <SelectTrigger className="w-full">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {providerOptions.map((provider) => (
                        <SelectItem key={provider} value={provider}>
                          {provider}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </label>

                <label className="flex flex-col gap-1.5 text-sm">
                  <span>{t("prototype.detail.name")}</span>
                  <Input
                    value={manualDraft.name}
                    onChange={(event) => updateManualDraft("name", event.target.value)}
                    placeholder="Claude Code"
                  />
                </label>

                <label className="col-span-2 flex flex-col gap-1.5 text-sm">
                  <span>{t("prototype.detail.rootPath")}</span>
                  <Input
                    value={manualDraft.rootPath}
                    onChange={(event) => updateManualDraft("rootPath", event.target.value)}
                    placeholder=".claude/"
                  />
                </label>

                <label className="col-span-2 flex flex-col gap-1.5 text-sm">
                  <span>{t("prototype.detail.config")}</span>
                  <Input
                    value={manualDraft.configPath}
                    onChange={(event) => updateManualDraft("configPath", event.target.value)}
                    placeholder=".claude/settings.json"
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

          <div className="max-h-[62vh] overflow-auto p-2">
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
              <div className="grid grid-cols-1 gap-2 sm:grid-cols-2 xl:grid-cols-4">
                {managementCards.map((candidate) => {
                  const isReady = candidate.state === "ready";
                  const isImported = candidate.state === "imported";
                  const stateLabel = candidateStateLabel(candidate, t);
                  return (
                    <button
                      type="button"
                      key={candidate.id}
                      onClick={(event) => {
                        event.preventDefault();
                        event.stopPropagation();
                        void toggleCandidate(candidate);
                      }}
                      disabled={isImporting || (!isReady && !isImported)}
                      className={cn(
                        "bg-card w-full rounded-lg border p-2 text-left transition-all",
                        isImported || isReady
                          ? "hover:border-primary/40 hover:bg-accent/40 focus-visible:ring-ring cursor-pointer focus-visible:ring-2 focus-visible:outline-none"
                          : "cursor-default",
                        isImported
                          ? "border-emerald-500/40 bg-emerald-500/10"
                          : isReady
                            ? "border-sky-500/40 bg-sky-500/10"
                            : "border-border bg-muted/60"
                      )}
                    >
                      <div className="flex items-start gap-2.5">
                        <div className="bg-background flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border">
                          <AgentProviderIcon provider={candidate.provider} size={18} />
                        </div>
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-2">
                            <div className="min-w-0 flex-1 truncate text-sm font-medium">
                              {candidate.displayName}
                            </div>
                            <div className="flex shrink-0 items-center gap-1">
                              {isImported ? (
                                <span className="flex h-5 w-5 items-center justify-center rounded-full bg-emerald-500 text-white">
                                  <Check className="h-3.5 w-3.5" />
                                </span>
                              ) : null}
                              {candidate.deletable ? (
                                <Button
                                  type="button"
                                  variant="ghost"
                                  size="icon-sm"
                                  className="text-destructive hover:text-destructive shrink-0 hover:bg-red-500/10"
                                  onClick={(event) => {
                                    event.preventDefault();
                                    event.stopPropagation();
                                    void deleteCandidate(candidate);
                                  }}
                                  title="Delete"
                                  disabled={isImporting}
                                >
                                  <Trash2 className="h-3.5 w-3.5" />
                                </Button>
                              ) : null}
                            </div>
                          </div>
                          <div className="text-muted-foreground mt-1 text-[11px]">
                            {candidate.rootPath}
                          </div>
                          <div className="text-muted-foreground mt-1 flex flex-wrap items-center gap-1.5 text-xs">
                            <span className="bg-muted rounded px-1.5 py-0.5 text-[10px]">
                              {candidate.sourceScope}
                            </span>
                            <span className="bg-muted rounded px-1.5 py-0.5 text-[10px]">
                              {candidate.resourceCounts.skill} Skills
                            </span>
                            <span className="bg-muted rounded px-1.5 py-0.5 text-[10px]">
                              {candidate.resourceCounts.mcp} MCP
                            </span>
                            <span className="bg-muted rounded px-1.5 py-0.5 text-[10px]">
                              {candidate.resourceCounts.subagent} Subagents
                            </span>
                          </div>
                          {stateLabel ? (
                            <div className="text-muted-foreground mt-1 text-[11px]">
                              {stateLabel}
                            </div>
                          ) : null}
                        </div>
                      </div>
                    </button>
                  );
                })}
              </div>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
