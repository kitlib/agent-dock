export type ResourceKind = "skill" | "mcp" | "subagent";
export type DiscoveryOrigin = "local" | "marketplace";
export type InstallStateLabel = "enabled" | "installed" | "update" | "available";
export type MarketplaceInstallStateLabel = Exclude<InstallStateLabel, "enabled">;

export type AgentGroup = {
  id: string;
  name: string;
  count: number;
};

export type AgentSummary = {
  id: string;
  name: string;
  role: string;
  status: "online" | "idle" | "busy";
  groupId: string;
  skillsCount: number;
  mcpCount: number;
  subagentsCount: number;
  summary: string;
};

export type SkillResource = {
  id: string;
  kind: "skill";
  name: string;
  summary: string;
  enabled: boolean;
  tags: string[];
  usageCount: number;
  updatedAt: string;
  markdown: string;
};

export type McpResource = {
  id: string;
  kind: "mcp";
  name: string;
  summary: string;
  enabled: boolean;
  endpoint: string;
  transport: string;
  usageCount: number;
  updatedAt: string;
  document: string;
  config: string;
};

export type SubagentResource = {
  id: string;
  kind: "subagent";
  name: string;
  summary: string;
  enabled: boolean;
  model: string;
  usageCount: number;
  updatedAt: string;
  prompt: string;
  capabilities: string[];
};

export type AgentResource = SkillResource | McpResource | SubagentResource;

export type MarketplaceDiscoveryFields = {
  origin: "marketplace";
  installState: MarketplaceInstallStateLabel;
  sourceLabel: string;
  version: string;
  author: string;
  downloads: number;
  description: string;
  highlights: string[];
  usageLabel?: never;
};

export type LocalDiscoveryFields = {
  origin: "local";
  installState: "enabled" | "installed";
  sourceLabel: string;
  version?: undefined;
  author?: undefined;
  downloads?: undefined;
  description: string;
  highlights: string[];
  usageLabel: number;
};

export type MarketplaceDiscoveryItem = {
  id: string;
  kind: ResourceKind;
  name: string;
  summary: string;
  updatedAt: string;
} & MarketplaceDiscoveryFields;

export type LocalDiscoveryItem = AgentResource & LocalDiscoveryFields;

export type AgentDiscoveryItem = LocalDiscoveryItem | MarketplaceDiscoveryItem;
