import { useEffect, useState } from "react";
import { Download } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  RadioGroup,
  RadioGroupItem,
} from "@/components/ui/radio-group";
import { AgentIcon } from "@/features/agents/components/agent-icon";
import type { ResolvedAgentView } from "@/features/agents/types";
import type { MarketplaceInstallMethod } from "@/features/marketplace/types";
import { cn } from "@/lib/utils";

type MarketplaceInstallAgentDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  targetAgents: ResolvedAgentView[];
  initialSelectedAgentId?: string | null;
  onConfirm: (agent: ResolvedAgentView, installMethod: MarketplaceInstallMethod) => Promise<void>;
  t: (key: string, options?: Record<string, unknown>) => string;
};

function getAgentMetaText(agent: ResolvedAgentView): string {
  if (agent.alias && agent.alias !== agent.name) {
    return agent.alias;
  }

  const segments = agent.rootPath.split(/[/\\]+/).filter(Boolean);
  return segments[segments.length - 1] ?? agent.rootPath;
}

export function MarketplaceInstallAgentDialog({
  open,
  onOpenChange,
  targetAgents,
  initialSelectedAgentId,
  onConfirm,
  t,
}: MarketplaceInstallAgentDialogProps) {
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);
  const [installMethod, setInstallMethod] = useState<MarketplaceInstallMethod>("skillsh");
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (open) {
      setSelectedAgentId(initialSelectedAgentId ?? null);
      setInstallMethod("skillsh");
      return;
    }

    setSelectedAgentId(null);
    setInstallMethod("skillsh");
    setIsSubmitting(false);
  }, [initialSelectedAgentId, open]);

  const selectedAgent = targetAgents.find((agent) => agent.id === selectedAgentId) ?? null;

  async function handleConfirm(): Promise<void> {
    if (!selectedAgent) {
      return;
    }

    setIsSubmitting(true);
    try {
      await onConfirm(selectedAgent, installMethod);
      onOpenChange(false);
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Download className="h-5 w-5" />
            {t("prototype.marketplace.installToAgentTitle")}
          </DialogTitle>
          <DialogDescription>
            {t("prototype.marketplace.installToAgentDescription")}
          </DialogDescription>
        </DialogHeader>

        <div className="grid max-h-[220px] grid-cols-1 gap-2 overflow-auto sm:grid-cols-2">
          {targetAgents.map((agent) => {
            const isSelected = selectedAgentId === agent.id;

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
                onClick={() => setSelectedAgentId(agent.id)}
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

        {targetAgents.length === 0 ? (
          <div className="text-muted-foreground text-sm">{t("prototype.emptyList")}</div>
        ) : null}

        <div className="space-y-2">
          <div className="text-sm font-medium">
            {t("prototype.marketplace.installMethodLabel")}
          </div>
          <RadioGroup
            value={installMethod}
            onValueChange={(value) => setInstallMethod(value as MarketplaceInstallMethod)}
            className="grid grid-cols-1 gap-2 sm:grid-cols-2"
          >
            <label
              className={cn(
                "flex cursor-pointer items-start gap-3 rounded-lg border px-3 py-3 transition-colors",
                installMethod === "skillsh"
                  ? "border-primary bg-accent text-accent-foreground"
                  : "bg-background border-border/70 hover:bg-accent hover:text-accent-foreground"
              )}
            >
              <RadioGroupItem value="skillsh" />
              <div className="min-w-0 flex-1">
                <div className="text-sm font-medium">
                  {t("prototype.marketplace.installMethods.skillsh")}
                </div>
                <div className="text-muted-foreground mt-1 text-xs">
                  {t("prototype.marketplace.installMethods.skillshDescription")}
                </div>
              </div>
            </label>
            <label
              className={cn(
                "flex cursor-pointer items-start gap-3 rounded-lg border px-3 py-3 transition-colors",
                installMethod === "github"
                  ? "border-primary bg-accent text-accent-foreground"
                  : "bg-background border-border/70 hover:bg-accent hover:text-accent-foreground"
              )}
            >
              <RadioGroupItem value="github" />
              <div className="min-w-0 flex-1">
                <div className="text-sm font-medium">
                  {t("prototype.marketplace.installMethods.github")}
                </div>
                <div className="text-muted-foreground mt-1 text-xs">
                  {t("prototype.marketplace.installMethods.githubDescription")}
                </div>
              </div>
            </label>
          </RadioGroup>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("prototype.actions.cancel")}
          </Button>
          <Button
            disabled={selectedAgent == null || isSubmitting}
            onClick={() => void handleConfirm()}
          >
            {t("prototype.actions.available")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
