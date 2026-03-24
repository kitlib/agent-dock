import { Bot, Brain, Plug, Sparkles } from "lucide-react";
import type { ComponentType } from "react";
import type { InstallStateLabel, ResourceKind } from "@/features/agents/types";

export const kindIcons: Record<ResourceKind, ComponentType<{ className?: string }>> = {
  skill: Sparkles,
  mcp: Plug,
  subagent: Brain,
};

export const installStateKey: Record<InstallStateLabel, string> = {
  enabled: "prototype.actions.enabled",
  installed: "prototype.actions.installed",
  update: "prototype.actions.update",
  available: "prototype.actions.available",
};

export const agentStatusClassName = {
  online: "bg-emerald-500",
  idle: "bg-amber-500",
  busy: "bg-sky-500",
};

export const agentRailGroupIcon = Bot;
