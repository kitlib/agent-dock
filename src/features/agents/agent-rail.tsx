import { ChevronLeft, ChevronRight, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { AgentProviderIcon } from "./provider-icon";
import type { AgentSummary } from "./types";

type AgentRailProps = {
  filteredAgents: AgentSummary[];
  isCollapsed: boolean;
  onAddAgent?: () => void;
  onToggleCollapsed: () => void;
  selectedAgentId: string;
  setSelectedAgentId: (id: string) => void;
  t: (key: string) => string;
};

export function AgentRail({
  filteredAgents,
  isCollapsed,
  onAddAgent,
  onToggleCollapsed,
  selectedAgentId,
  setSelectedAgentId,
  t,
}: AgentRailProps) {
  return (
    <div className="flex h-full min-w-0 flex-col overflow-x-hidden">
      <div className="flex items-center justify-between gap-2 p-3">
        {!isCollapsed && (
          <span className="text-sm font-semibold">{t("prototype.agents.agents")}</span>
        )}
        <Button
          variant="ghost"
          size="icon-sm"
          onClick={onToggleCollapsed}
          title={isCollapsed ? t("prototype.actions.expand") : t("prototype.actions.collapse")}
        >
          {isCollapsed ? <ChevronRight className="h-4 w-4" /> : <ChevronLeft className="h-4 w-4" />}
        </Button>
      </div>

      <div className="flex-1 overflow-auto p-2">
        <div className="space-y-1">
          {filteredAgents.map((agent) => (
            <button
              key={agent.id}
              onClick={() => setSelectedAgentId(agent.id)}
              className={cn(
                "flex w-full items-center rounded-lg px-2 py-2 text-left text-sm transition-colors",
                isCollapsed && "mx-auto h-9 w-9 justify-center p-0",
                selectedAgentId === agent.id
                  ? "bg-primary/10 text-foreground"
                  : "text-muted-foreground hover:bg-accent/50 hover:text-foreground"
              )}
              title={agent.name}
            >
              <div className={cn("mr-2 flex items-center gap-2", isCollapsed && "mr-0")}>
                <AgentProviderIcon provider={agent.provider} className="shrink-0" size={18} />
              </div>
              {!isCollapsed ? (
                <div className="min-w-0 flex-1">
                  <div className="truncate font-medium">{agent.alias ?? agent.name}</div>
                  <div className="text-muted-foreground truncate text-xs">{agent.rootPath}</div>
                </div>
              ) : null}
            </button>
          ))}
        </div>
      </div>

      <div className="border-t p-2">
        <Button
          variant="outline"
          size={isCollapsed ? "icon-sm" : "sm"}
          className={cn("w-full", isCollapsed && "mx-auto h-9 w-9 p-0")}
          onClick={onAddAgent}
          title={t("prototype.actions.add")}
        >
          <Plus className="h-4 w-4" />
          {!isCollapsed ? <span>{t("prototype.actions.add")}</span> : null}
        </Button>
      </div>
    </div>
  );
}
