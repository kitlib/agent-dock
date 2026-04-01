import {
  Amp,
  Antigravity,
  Claude,
  ClaudeCode,
  Cline,
  Codex,
  Cursor,
  GithubCopilot,
  Goose,
  Junie,
  Kimi,
  Mistral,
  OpenClaw,
  OpenCode,
  OpenHands,
  Qwen,
  Replit,
  Trae,
  Windsurf,
  Zencoder,
} from "@lobehub/icons";
import { Bot } from "lucide-react";
import type { ComponentType } from "react";
import type { AgentTypeId } from "../types";

type IconComponent = ComponentType<{ size?: number; className?: string }>;

export const agentIcons: Record<AgentTypeId, IconComponent> = {
  adal: Bot,
  amp: Amp,
  antigravity: Antigravity.Color,
  augment: Bot,
  claude: Claude.Color,
  "claude-plugin": ClaudeCode.Color,
  cline: Cline,
  codebuddy: Bot,
  codex: Codex.Color,
  "command-code": Bot,
  continue: Bot,
  crush: Bot,
  cursor: Cursor,
  factory: Bot,
  "github-copilot": GithubCopilot,
  goose: Goose,
  iflow: Bot,
  junie: Junie.Color,
  kilo: Bot,
  kimi: Kimi.Color,
  kiro: Bot,
  kode: Bot,
  mcpjam: Bot,
  mistral: Mistral.Color,
  mux: Bot,
  neovate: Bot,
  openclaw: OpenClaw.Color,
  opencode: OpenCode,
  openhands: OpenHands.Color,
  "pi-mono": Bot,
  pochi: Bot,
  qoder: Bot,
  qwen: Qwen.Color,
  replit: Replit.Color,
  roo: Bot,
  trae: Trae.Color,
  "trae-cn": Trae.Color,
  warp: Bot,
  windsurf: Windsurf,
  zencoder: Zencoder.Color,
};

export function AgentIcon({
  agentType,
  className,
  size = 18,
}: {
  agentType: AgentTypeId;
  className?: string;
  size?: number;
}) {
  const Icon = agentIcons[agentType];

  return <Icon size={size} className={className} />;
}
