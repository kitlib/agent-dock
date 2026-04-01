import type { AgentResource, ResourceKind } from "@/features/agents/types";

const localResources = {
  skill: [],
  mcp: [
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
  ],
  subagent: [
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
  ],
} satisfies Record<ResourceKind, AgentResource[]>;

export const resourcesByKind = localResources;
