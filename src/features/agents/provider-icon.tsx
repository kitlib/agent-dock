import { Antigravity, Claude, Codex, Cursor } from "@lobehub/icons";
import type { ComponentType } from "react";
import type { AgentProvider } from "./types";

type ColorIconComponent = ComponentType<{ size?: number; className?: string }>;

const agentProviderIcons: Partial<Record<AgentProvider, ColorIconComponent>> = {
  cursor: Cursor as ColorIconComponent,
  claude: Claude.Color as ColorIconComponent,
  codex: Codex.Color as ColorIconComponent,
  antigravity: Antigravity.Color as ColorIconComponent,
};

export function AgentProviderIcon({
  provider,
  className,
  size = 18,
}: {
  provider: AgentProvider;
  className?: string;
  size?: number;
}) {
  const Icon = agentProviderIcons[provider]!;

  return <Icon size={size} className={className} />;
}
