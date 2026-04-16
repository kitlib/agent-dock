import { Boxes, ChevronLeft, ChevronRight, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { AgentIcon } from "./agent-icon";
import type { AgentSelectionScope, AgentSummary } from "../types";

type AgentRailProps = {
  filteredAgents: AgentSummary[];
  isCollapsed: boolean;
  onAddAgent?: () => void;
  onSelectAll: () => void;
  onSelectAgent: (id: string) => void;
  onToggleCollapsed: () => void;
  selectedScope: AgentSelectionScope;
  selectedAgentId: string;
  t: (key: string) => string;
};

export function AgentRail({
  filteredAgents,
  isCollapsed,
  onAddAgent,
  onSelectAll,
  onSelectAgent,
  onToggleCollapsed,
  selectedScope,
  selectedAgentId,
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
          <button
            onClick={onSelectAll}
            className={cn(
              "flex w-full items-center rounded-lg px-2 py-2 text-left text-sm transition-colors",
              isCollapsed && "mx-auto h-9 w-9 justify-center p-0",
              selectedScope === "all"
                ? "border-border bg-accent text-foreground border shadow-xs"
                : "text-muted-foreground hover:bg-accent/50 hover:text-foreground border border-transparent"
            )}
            title={t("prototype.agents.all")}
          >
            <div className={cn("mr-2 flex items-center gap-2", isCollapsed && "mr-0")}>
              <Boxes className="h-[18px] w-[18px] shrink-0" />
            </div>
            {!isCollapsed ? (
              <div className="min-w-0 flex-1">
                <div className="truncate font-medium">{t("prototype.agents.all")}</div>
              </div>
            ) : null}
          </button>
          <div className="border-border/70 my-2 border-t" />
          {filteredAgents.map((agent) => (
            <button
              key={agent.id}
              onClick={() => onSelectAgent(agent.id)}
              className={cn(
                "flex w-full items-center rounded-lg px-2 py-2 text-left text-sm transition-colors",
                isCollapsed && "mx-auto h-9 w-9 justify-center p-0",
                selectedScope === "agent" && selectedAgentId === agent.id
                  ? "border-border bg-accent text-foreground border shadow-xs"
                  : "text-muted-foreground hover:bg-accent/50 hover:text-foreground border border-transparent"
              )}
              title={agent.name}
            >
              <div className={cn("mr-2 flex items-center gap-2", isCollapsed && "mr-0")}>
                <AgentIcon agentType={agent.agentType} className="shrink-0" size={18} />
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
