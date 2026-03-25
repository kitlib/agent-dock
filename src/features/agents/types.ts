export type ResourceKind = "skill" | "mcp" | "subagent";
export type DiscoveryOrigin = "local" | "marketplace";
export type InstallStateLabel = "enabled" | "installed" | "update" | "available";
export type MarketplaceInstallStateLabel = Exclude<InstallStateLabel, "enabled">;

export type AgentProvider = "cursor" | "claude" | "codex" | "antigravity";
export type AgentSourceScope = "global" | "user" | "workspace" | "manual";
export type AgentDiscoveryStatus = "discovered" | "invalid" | "unreadable" | "conflict";
export type AgentHealth = "ok" | "warning" | "error";
export type AgentConflictType =
  | "duplicate-fingerprint"
  | "same-provider-multi-source"
  | "same-root-path"
  | "manual-vs-discovered";
export type AgentConflictSeverity = "info" | "warning" | "error";
export type AgentImportCandidateState = "ready" | "imported" | "conflict" | "unreadable";

export type AgentGroup = {
  id: string;
  name: string;
  count: number;
};

export type AgentResourceCounts = {
  skill: number;
  mcp: number;
  subagent: number;
};

export type DiscoveredAgent = {
  discoveryId: string;
  fingerprint: string;
  provider: AgentProvider;
  displayName: string;
  rootPath: string;
  configPath?: string;
  sourceScope: AgentSourceScope;
  workspaceName?: string;
  status: AgentDiscoveryStatus;
  reason?: string;
  resourceCounts: AgentResourceCounts;
  detectedAt: string;
};

export type ManagedAgent = {
  managedAgentId: string;
  fingerprint: string;
  alias?: string;
  enabled: boolean;
  hidden: boolean;
  importedAt: string;
  source: "auto-imported" | "manual-imported";
};

export type AgentConflict = {
  id: string;
  type: AgentConflictType;
  severity: AgentConflictSeverity;
  summary: string;
  agentFingerprints: string[];
  suggestedResolution?: "keep-latest" | "keep-managed" | "ask-user";
};

export type ResolvedAgentView = {
  id: string;
  discoveryId: string;
  fingerprint: string;
  provider: AgentProvider;
  name: string;
  alias?: string;
  role: string;
  rootPath: string;
  configPath?: string;
  sourceScope: AgentSourceScope;
  managed: boolean;
  managedAgentId?: string;
  enabled: boolean;
  hidden: boolean;
  health: AgentHealth;
  status: AgentDiscoveryStatus;
  statusLabel: string;
  summary: string;
  groupId: string;
  resourceCounts: AgentResourceCounts;
  conflictIds: string[];
  lastScannedAt: string;
};

export type AgentDiscoveryState = {
  initialized: boolean;
  scanning: boolean;
  refreshing: boolean;
  error: string | null;
  lastScannedAt: string | null;
};

export type ScannedAgentCandidate = {
  id: string;
  fingerprint: string;
  provider: AgentProvider;
  displayName: string;
  rootPath: string;
  configPath?: string;
  sourceScope: AgentSourceScope;
  workspaceName?: string;
  resourceCounts: AgentResourceCounts;
  state: AgentImportCandidateState;
  reason?: string;
  managedAgentId?: string;
  managed: boolean;
  detectedAt: string;
};

export type AgentManagementCard = ScannedAgentCandidate & {
  origin: "scanned" | "manual";
  deletable: boolean;
};

export type ManualAgentDraft = {
  provider: AgentProvider;
  name: string;
  rootPath: string;
  configPath: string;
};

export type ImportAgentsResult = {
  importedAgents: ResolvedAgentView[];
  resolvedAgents: ResolvedAgentView[];
};

export type RemoveAgentResult = {
  removedAgentId: string;
  resolvedAgents: ResolvedAgentView[];
};

export type DeleteAgentResult = {
  deletedAgentId: string;
  resolvedAgents: ResolvedAgentView[];
};

export type CreateAgentResult = {
  agent: ResolvedAgentView;
  resolvedAgents: ResolvedAgentView[];
};

export type AgentSummary = ResolvedAgentView;

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

export type AgentResourceView = AgentDiscoveryItem & {
  ownerAgentId: string | null;
  managed: boolean;
  configPath?: string;
  conflictState?: AgentConflictSeverity;
};
