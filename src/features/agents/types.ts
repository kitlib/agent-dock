export type ResourceKind = "skill" | "mcp" | "subagent";
export type DiscoveryOrigin = "local" | "marketplace";
export type InstallStateLabel = "enabled" | "installed" | "update" | "available";
export type MarketplaceInstallStateLabel = Exclude<InstallStateLabel, "enabled">;

export type AgentTypeId =
  | "adal"
  | "amp"
  | "antigravity"
  | "augment"
  | "claude"
  | "cline"
  | "codebuddy"
  | "codex"
  | "command-code"
  | "continue"
  | "crush"
  | "cursor"
  | "factory"
  | "github-copilot"
  | "goose"
  | "iflow"
  | "junie"
  | "kilo"
  | "kimi"
  | "kiro"
  | "kode"
  | "mcpjam"
  | "mistral"
  | "mux"
  | "neovate"
  | "openclaw"
  | "opencode"
  | "openhands"
  | "pi-mono"
  | "pochi"
  | "qoder"
  | "qwen"
  | "replit"
  | "roo"
  | "trae"
  | "trae-cn"
  | "warp"
  | "windsurf"
  | "zencoder";

export type AgentTypeMeta = {
  agentType: AgentTypeId;
  name: string;
  directory: string;
  rootFile: string | null;
  rules: string | null;
  commands: string | null;
  agents: string | null;
  skills: string | null;
  mcp: string | null;
};

export type AgentTypeMetaMap = Record<AgentTypeId, AgentTypeMeta>;

export type AgentDiscoveryStatus = "discovered" | "invalid" | "unreadable";
export type AgentHealth = "ok" | "error";
export type AgentImportCandidateState = "ready" | "imported" | "unreadable";

export type AgentResourceCounts = {
  skill: number;
  command: number;
  mcp: number;
  subagent: number;
};

export type DiscoveredAgent = {
  discoveryId: string;
  fingerprint: string;
  agentType: AgentTypeId;
  displayName: string;
  rootPath: string;
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

export type ResolvedAgentView = {
  id: string;
  discoveryId: string;
  fingerprint: string;
  agentType: AgentTypeId;
  name: string;
  alias?: string;
  role: string;
  rootPath: string;
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
  agentType: AgentTypeId;
  displayName: string;
  rootPath: string;
  resourceCounts: AgentResourceCounts;
  state: AgentImportCandidateState;
  reason?: string;
  managedAgentId?: string;
  managed: boolean;
  detectedAt: string;
};

export type AgentManagementCard =
  | (ScannedAgentCandidate & {
      origin: "scanned";
      deletable: false;
    })
  | (ScannedAgentCandidate & {
      origin: "manual";
      deletable: true;
    });

export type ScanTarget = {
  agentType: AgentTypeId;
  name: string;
  rootPath: string;
};

export type ManualAgentDraft = {
  agentType: AgentTypeId;
  name: string;
  rootPath: string;
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

export type SkillScanTarget = {
  agentId: string;
  agentType: AgentTypeId;
  rootPath: string;
  displayName: string;
  source: "skills" | "commands";
};

export type SkillSupportingFile = {
  path: string;
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
  ownerAgentId?: string | null;
  sourceLabel?: string;
  description?: string;
  status?: string;
  skillPath?: string;
  entryFilePath?: string;
  agentType?: string;
  agentName?: string;
  warnings?: string[];
  errors?: string[];
  frontmatter?: Record<string, unknown> | null;
  frontmatterRaw?: string | null;
  supportingFiles?: SkillSupportingFile[];
  allowedTools?: string[];
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
};
