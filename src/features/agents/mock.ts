import type {
  AgentGroup,
  AgentResource,
  AgentSummary,
  McpResource,
  ResourceKind,
  SkillResource,
  SubagentResource,
} from "./types";

export const agentGroups: AgentGroup[] = [
  { id: "all", name: "All Agents", count: 4 },
  { id: "assistant", name: "Assistant", count: 4 },
];

export const agents: AgentSummary[] = [
  {
    id: "agent-cursor",
    name: "Cursor",
    role: "AI coding assistant",
    status: "online",
    groupId: "assistant",
    skillsCount: 6,
    mcpCount: 1,
    subagentsCount: 1,
    summary: "Uses .cursor rules, commands, agents, skills, and mcp.json based workspace setup.",
  },
  {
    id: "agent-claude-code",
    name: "Claude Code",
    role: "CLI coding assistant",
    status: "busy",
    groupId: "assistant",
    skillsCount: 6,
    mcpCount: 1,
    subagentsCount: 1,
    summary:
      "Uses CLAUDE.md with rules, commands, agents, skills, and root .mcp.json configuration.",
  },
  {
    id: "agent-codex-cli",
    name: "Codex CLI",
    role: "Terminal coding assistant",
    status: "idle",
    groupId: "assistant",
    skillsCount: 3,
    mcpCount: 1,
    subagentsCount: 0,
    summary: "Uses .codex prompts, skills, and config.toml for terminal-based coding workflows.",
  },
  {
    id: "agent-antigravity",
    name: "Antigravity",
    role: "Workflow automation assistant",
    status: "online",
    groupId: "assistant",
    skillsCount: 4,
    mcpCount: 0,
    subagentsCount: 1,
    summary: "Uses .agent rules, workflows, and skills to drive structured agent workflows.",
  },
];

export const skillResources: SkillResource[] = [
  {
    id: "skill-release-checklist",
    kind: "skill",
    name: "Release Checklist",
    summary: "Runs local release preparation steps with manual gates.",
    enabled: true,
    tags: ["workflow", "release"],
    usageCount: 12,
    updatedAt: "2026-03-22",
    markdown:
      "# Release Checklist\n\n- Verify version files\n- Run smoke checks\n- Prepare notes\n- Pause before push",
  },
  {
    id: "skill-prototype-review",
    kind: "skill",
    name: "Prototype Review",
    summary: "Audits layout consistency, spacing, and interaction states.",
    enabled: true,
    tags: ["ui", "review"],
    usageCount: 9,
    updatedAt: "2026-03-21",
    markdown:
      "# Prototype Review\n\nUse this skill to inspect hierarchy, spacing, and i18n coverage before handoff.",
  },
  {
    id: "skill-doc-sync",
    kind: "skill",
    name: "Doc Sync",
    summary: "Keeps feature docs aligned with implementation milestones.",
    enabled: false,
    tags: ["docs"],
    usageCount: 4,
    updatedAt: "2026-03-18",
    markdown: "# Doc Sync\n\nCompare feature docs with current code paths and list mismatches.",
  },
];

export const mcpResources: McpResource[] = [
  {
    id: "mcp-filesystem-local",
    kind: "mcp",
    name: "Filesystem Local",
    summary: "Accesses workspace files and metadata with local permissions.",
    enabled: true,
    endpoint: "stdio://filesystem-local",
    transport: "stdio",
    usageCount: 14,
    updatedAt: "2026-03-20",
    document:
      "# Filesystem Local\n\nProvides controlled file reads, writes, and glob access inside the workspace.",
    config: '{\n  "command": "pnpm",\n  "args": ["mcp:filesystem"],\n  "cwd": "~/workspace"\n}',
  },
  {
    id: "mcp-browser-inspect",
    kind: "mcp",
    name: "Browser Inspect",
    summary: "Captures DOM snapshots and interaction traces for debugging.",
    enabled: true,
    endpoint: "http://127.0.0.1:3030/mcp",
    transport: "http",
    usageCount: 7,
    updatedAt: "2026-03-19",
    document:
      "# Browser Inspect\n\nExposes page snapshots, console logs, and input automation for local test sessions.",
    config:
      '{\n  "url": "http://127.0.0.1:3030/mcp",\n  "headers": {\n    "x-project": "agentdock"\n  }\n}',
  },
  {
    id: "mcp-notes-index",
    kind: "mcp",
    name: "Notes Index",
    summary: "Searches indexed markdown notes and internal summaries.",
    enabled: false,
    endpoint: "stdio://notes-index",
    transport: "stdio",
    usageCount: 3,
    updatedAt: "2026-03-17",
    document: "# Notes Index\n\nReturns note matches ranked by recency and semantic relevance.",
    config: '{\n  "command": "node",\n  "args": ["./tools/notes-index.js"]\n}',
  },
];

export const subagentResources: SubagentResource[] = [
  {
    id: "subagent-ui-critic",
    kind: "subagent",
    name: "UI Critic",
    summary: "Reviews layouts against compact desktop UI patterns.",
    enabled: true,
    model: "claude-sonnet-4-6",
    usageCount: 11,
    updatedAt: "2026-03-22",
    prompt: "Inspect layout density, navigation clarity, and right-panel readability.",
    capabilities: ["layout review", "dark mode review", "state coverage"],
  },
  {
    id: "subagent-schema-audit",
    kind: "subagent",
    name: "Schema Audit",
    summary: "Checks typed mock data and API replacement readiness.",
    enabled: true,
    model: "claude-haiku-4-5-20251001",
    usageCount: 8,
    updatedAt: "2026-03-20",
    prompt: "Verify that mock structures can be swapped with server payloads later.",
    capabilities: ["type audit", "mock review"],
  },
  {
    id: "subagent-ops-shadow",
    kind: "subagent",
    name: "Ops Shadow",
    summary: "Simulates release and rollout handoff notes for human review.",
    enabled: false,
    model: "claude-opus-4-6",
    usageCount: 2,
    updatedAt: "2026-03-16",
    prompt: "Summarize rollout blockers, open checks, and likely owner actions.",
    capabilities: ["release notes", "handoff summary"],
  },
];

export const resourcesByKind: Record<ResourceKind, AgentResource[]> = {
  skill: skillResources,
  mcp: mcpResources,
  subagent: subagentResources,
};
