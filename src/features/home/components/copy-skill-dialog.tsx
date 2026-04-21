import { useEffect, useState } from "react";
import { Copy, AlertTriangle } from "lucide-react";
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
import { cn } from "@/lib/utils";
import type {
  LocalSkillCopySource,
  LocalSkillCopyTargetAgent,
  PreviewLocalSkillCopyResult,
  LocalSkillCopyConflict,
  LocalSkillConflictResolution,
  ResolvedAgentView,
} from "@/features/agents/types";

type CopySkillDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  sources: LocalSkillCopySource[];
  targetAgents: ResolvedAgentView[];
  onPreview: (
    sources: LocalSkillCopySource[],
    targetAgent: LocalSkillCopyTargetAgent
  ) => Promise<PreviewLocalSkillCopyResult>;
  onCopy: (
    sources: LocalSkillCopySource[],
    targetAgent: LocalSkillCopyTargetAgent,
    resolutions: LocalSkillConflictResolution[]
  ) => Promise<void>;
  t: (key: string, options?: Record<string, unknown>) => string;
};

type PreviewResultsByAgent = Record<string, PreviewLocalSkillCopyResult>;

function resolutionKey(agentId: string, skillId: string): string {
  return `${agentId}::${skillId}`;
}

function getAgentMetaText(agent: ResolvedAgentView): string {
  if (agent.alias && agent.alias !== agent.name) {
    return agent.alias;
  }

  const segments = agent.rootPath.split(/[/\\]+/).filter(Boolean);
  return segments[segments.length - 1] ?? agent.rootPath;
}

function toTargetAgent(agent: ResolvedAgentView): LocalSkillCopyTargetAgent {
  return {
    agentId: agent.id,
    agentType: agent.agentType,
    agentName: agent.name,
    rootPath: agent.rootPath,
  };
}

function ConflictItem({
  conflict,
  resolution,
  onResolve,
  t,
}: {
  conflict: LocalSkillCopyConflict;
  resolution: "overwrite" | "skip" | null;
  onResolve: (action: "overwrite" | "skip") => void;
  t: CopySkillDialogProps["t"];
}) {
  return (
    <div className="flex items-center justify-between gap-2 rounded-md border border-amber-500/30 bg-amber-500/10 px-3 py-2">
      <div className="flex items-center gap-2">
        <AlertTriangle className="h-4 w-4 text-amber-600" />
        <span className="text-sm font-medium">{conflict.skillName}</span>
      </div>
      <div className="flex gap-1">
        <Button
          variant={resolution === "overwrite" ? "default" : "outline"}
          size="xs"
          onClick={() => onResolve("overwrite")}
        >
          {t("prototype.actions.overwrite")}
        </Button>
        <Button
          variant={resolution === "skip" ? "default" : "outline"}
          size="xs"
          onClick={() => onResolve("skip")}
        >
          {t("prototype.actions.skip")}
        </Button>
      </div>
    </div>
  );
}

export function CopySkillDialog({
  open,
  onOpenChange,
  sources,
  targetAgents,
  onPreview,
  onCopy,
  t,
}: CopySkillDialogProps) {
  const [selectedAgentIds, setSelectedAgentIds] = useState<string[]>([]);
  const [previewResults, setPreviewResults] = useState<PreviewResultsByAgent>({});
  const [resolutions, setResolutions] = useState<Map<string, "overwrite" | "skip">>(new Map());
  const [previewingAgentIds, setPreviewingAgentIds] = useState<string[]>([]);
  const [isCopying, setIsCopying] = useState(false);
  const [applyToAll, setApplyToAll] = useState<"overwrite" | "skip" | null>(null);

  const filteredTargetAgents = targetAgents.filter(
    (agent) => !sources.some((s) => s.ownerAgentId === agent.id)
  );
  const selectedAgents = filteredTargetAgents.filter((agent) =>
    selectedAgentIds.includes(agent.id)
  );
  const isPreviewing = previewingAgentIds.length > 0;
  const allConflicts = selectedAgents.flatMap((agent) =>
    (previewResults[agent.id]?.conflicts ?? []).map((conflict) => ({
      agentId: agent.id,
      conflict,
    }))
  );
  const hasConflicts = allConflicts.length > 0;
  const allPreviewsReady =
    selectedAgentIds.length > 0 &&
    selectedAgentIds.every((agentId) => previewResults[agentId] != null);
  const allResolved =
    !hasConflicts ||
    allConflicts.every(({ agentId, conflict }) =>
      resolutions.has(resolutionKey(agentId, conflict.skillId))
    ) ||
    applyToAll !== null;

  const resetDialogState = () => {
    setSelectedAgentIds([]);
    setPreviewResults({});
    setResolutions(new Map());
    setPreviewingAgentIds([]);
    setIsCopying(false);
    setApplyToAll(null);
  };

  useEffect(() => {
    if (open) {
      return;
    }

    resetDialogState();
  }, [open]);

  useEffect(() => {
    if (open) {
      return;
    }

    resetDialogState();
  }, [open, sources]);

  const previewAgent = async (agentId: string) => {
    const agent = targetAgents.find((a) => a.id === agentId);
    if (!agent || sources.length === 0) {
      return;
    }

    setPreviewingAgentIds((current) => [...current, agentId]);
    try {
      const result = await onPreview(sources, toTargetAgent(agent));
      setPreviewResults((current) => ({ ...current, [agentId]: result }));
    } catch (error) {
      console.error("Failed to preview copy:", error);
    } finally {
      setPreviewingAgentIds((current) => current.filter((id) => id !== agentId));
    }
  };

  const handleSelectAgent = async (agentId: string) => {
    if (selectedAgentIds.includes(agentId)) {
      setSelectedAgentIds((current) => current.filter((id) => id !== agentId));
      setPreviewResults((current) => {
        const next = { ...current };
        delete next[agentId];
        return next;
      });
      setResolutions((current) => {
        const next = new Map(current);
        Array.from(next.keys()).forEach((key) => {
          if (key.startsWith(`${agentId}::`)) {
            next.delete(key);
          }
        });
        return next;
      });
      setPreviewingAgentIds((current) => current.filter((id) => id !== agentId));
      setApplyToAll(null);
      return;
    }

    setSelectedAgentIds((current) => [...current, agentId]);
    setApplyToAll(null);
    await previewAgent(agentId);
  };

  const handleResolve = (agentId: string, skillId: string, action: "overwrite" | "skip") => {
    const next = new Map(resolutions);
    next.set(resolutionKey(agentId, skillId), action);
    setResolutions(next);
  };

  const handleApplyToAll = (action: "overwrite" | "skip") => {
    setApplyToAll(action);
    const next = new Map();
    allConflicts.forEach(({ agentId, conflict }) =>
      next.set(resolutionKey(agentId, conflict.skillId), action)
    );
    setResolutions(next);
  };

  const handleCopy = async () => {
    if (!allPreviewsReady || selectedAgents.length === 0) return;

    setIsCopying(true);
    try {
      for (const agent of selectedAgents) {
        const previewResult = previewResults[agent.id];
        if (!previewResult) {
          continue;
        }

        const resolutionList: LocalSkillConflictResolution[] = previewResult.conflicts.map((c) => ({
          skillId: c.skillId,
          action: resolutions.get(resolutionKey(agent.id, c.skillId)) ?? "skip",
        }));

        await onCopy(sources, toTargetAgent(agent), resolutionList);
      }

      onOpenChange(false);
      resetDialogState();
    } catch (error) {
      console.error("Failed to copy:", error);
    } finally {
      setIsCopying(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md max-h-[90vh] flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Copy className="h-5 w-5" />
            {t("prototype.copySkill.title")}
          </DialogTitle>
          <DialogDescription>
            {sources.length > 0 ? t("prototype.copySkill.selectTarget") : t("prototype.emptyList")}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 overflow-y-auto flex-1">
          <div className="grid max-h-[220px] grid-cols-1 gap-2 overflow-auto sm:grid-cols-2">
            {filteredTargetAgents.map((agent) => {
              const isSelected = selectedAgentIds.includes(agent.id);

              return (
                <button
                  key={agent.id}
                  type="button"
                  className={cn(
                    "hover:bg-accent hover:text-accent-foreground rounded-lg border px-3 py-3 text-left transition-colors",
                    isSelected
                      ? "border-primary bg-accent text-accent-foreground"
                      : "bg-background border-border/70"
                  )}
                  onClick={() => void handleSelectAgent(agent.id)}
                >
                  <div className="flex items-start gap-3">
                    <div className="bg-muted/60 rounded-md p-1.5">
                      <AgentIcon agentType={agent.agentType} size={16} />
                    </div>
                    <div className="min-w-0 flex-1">
                      <div className="truncate text-sm font-medium">{agent.name}</div>
                      <div className="text-muted-foreground mt-1 truncate text-xs">
                        {getAgentMetaText(agent)}
                      </div>
                    </div>
                  </div>
                </button>
              );
            })}
          </div>

          <div className="min-h-[180px]">
            {isPreviewing ? (
              <div className="text-muted-foreground text-sm">
                {t("prototype.copySkill.previewingTargets")}
              </div>
            ) : selectedAgents.length > 0 ? (
              <div className="space-y-2">
                <div className="text-muted-foreground text-sm">
                  {t("prototype.copySkill.selectedTargets", { count: selectedAgents.length })}
                </div>

                {hasConflicts ? (
                  <div className="space-y-2">
                    <div className="flex gap-2">
                      <Button
                        variant="outline"
                        size="xs"
                        onClick={() => handleApplyToAll("overwrite")}
                      >
                        {t("prototype.actions.overwriteAll")}
                      </Button>
                      <Button variant="outline" size="xs" onClick={() => handleApplyToAll("skip")}>
                        {t("prototype.actions.skipAll")}
                      </Button>
                    </div>
                    <div className="max-h-[200px] overflow-auto rounded-md border p-2">
                      {selectedAgents.map((agent) => {
                        const previewResult = previewResults[agent.id];
                        if (!previewResult) {
                          return null;
                        }

                        return (
                          <div key={agent.id} className="space-y-2 py-2 first:pt-0 last:pb-0">
                            <div className="text-sm font-medium">
                              {agent.name} ({agent.agentType})
                            </div>
                            {previewResult.conflicts.length > 0 ? (
                              <div className="space-y-2">
                                {previewResult.conflicts.map((conflict) => (
                                  <ConflictItem
                                    key={`${agent.id}:${conflict.skillId}`}
                                    conflict={conflict}
                                    resolution={
                                      applyToAll ??
                                      resolutions.get(resolutionKey(agent.id, conflict.skillId)) ??
                                      null
                                    }
                                    onResolve={(action) =>
                                      handleResolve(agent.id, conflict.skillId, action)
                                    }
                                    t={t}
                                  />
                                ))}
                              </div>
                            ) : (
                              <div className="text-muted-foreground text-sm">
                                {t("prototype.copySkill.noConflicts")}
                              </div>
                            )}
                          </div>
                        );
                      })}
                    </div>
                  </div>
                ) : (
                  <div className="text-sm">{t("prototype.copySkill.noConflicts")}</div>
                )}
              </div>
            ) : null}
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("prototype.actions.cancel")}
          </Button>
          <Button
            disabled={!allPreviewsReady || !allResolved || isCopying || isPreviewing}
            onClick={() => void handleCopy()}
          >
            {t("prototype.actions.copy")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
