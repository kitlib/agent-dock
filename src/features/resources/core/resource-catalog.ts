import type { AgentResource, ResourceKind } from "@/features/agents/types";

const localResources = {
  skill: [],
  mcp: [],
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
