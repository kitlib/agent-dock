export type ResourceKind = "skill" | "mcp" | "subagent";

export type ResourceItem = {
  id: string;
  kind: ResourceKind;
  name: string;
  summary: string;
  version: string;
  source: string;
  usedByCount: number;
  updatedAt: string;
};

export type SkillDetail = ResourceItem & {
  kind: "skill";
  tags: string[];
  markdown: string;
};

export type McpDetail = ResourceItem & {
  kind: "mcp";
  endpoint: string;
  transport: string;
  document: string;
  config: string;
};

export type SubagentDetail = ResourceItem & {
  kind: "subagent";
  model: string;
  prompt: string;
  capabilities: string[];
};

export type ResourceDetail = SkillDetail | McpDetail | SubagentDetail;
