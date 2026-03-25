import { discoveredAgents, resolvedAgents } from "./mock";
import type {
  CreateAgentResult,
  DeleteAgentResult,
  ImportAgentsResult,
  ManualAgentDraft,
  RemoveAgentResult,
  ResolvedAgentView,
  ScannedAgentCandidate,
} from "./types";

function createCandidateId(discoveryId: string) {
  return discoveryId.replace(/^discovery-/, "candidate-");
}

function inferCandidateState(
  discoveryId: string,
  managed: boolean,
  status: ResolvedAgentView["status"]
) {
  if (managed) {
    return "imported" as const;
  }

  if (status === "conflict") {
    return "conflict" as const;
  }

  if (status === "unreadable" || status === "invalid") {
    return "unreadable" as const;
  }

  return "ready" as const;
}

export function buildScanCandidates(
  agents: ResolvedAgentView[] = resolvedAgents
): ScannedAgentCandidate[] {
  return discoveredAgents.map((discoveredAgent) => {
    const resolvedAgent = agents.find((agent) => agent.discoveryId === discoveredAgent.discoveryId);

    return {
      id: createCandidateId(discoveredAgent.discoveryId),
      fingerprint: discoveredAgent.fingerprint,
      provider: discoveredAgent.provider,
      displayName: discoveredAgent.displayName,
      rootPath: discoveredAgent.rootPath,
      configPath: discoveredAgent.configPath,
      sourceScope: discoveredAgent.sourceScope,
      workspaceName: discoveredAgent.workspaceName,
      resourceCounts: discoveredAgent.resourceCounts,
      state: inferCandidateState(
        discoveredAgent.discoveryId,
        resolvedAgent?.managed ?? false,
        resolvedAgent?.status ?? discoveredAgent.status
      ),
      reason: discoveredAgent.reason,
      managedAgentId: resolvedAgent?.managedAgentId,
      managed: resolvedAgent?.managed ?? false,
      detectedAt: discoveredAgent.detectedAt,
    };
  });
}

export function applyImportToResolvedAgents(
  currentAgents: ResolvedAgentView[],
  candidateIds: string[]
): ImportAgentsResult {
  const discoveryIds = new Set(candidateIds.map((id) => id.replace(/^candidate-/, "discovery-")));

  const nextAgents = currentAgents.map((agent) => {
    if (!discoveryIds.has(agent.discoveryId) || agent.managed) {
      return agent;
    }

    return {
      ...agent,
      managed: true,
      managedAgentId: agent.managedAgentId ?? `managed-${agent.id}`,
      enabled: true,
      statusLabel: "Managed",
      summary: `Imported ${agent.name} into AgentDock management.`,
    };
  });

  return {
    importedAgents: nextAgents.filter(
      (agent) => discoveryIds.has(agent.discoveryId) && agent.managed && agent.enabled
    ),
    resolvedAgents: nextAgents,
  };
}

export function removeManagedAgentFromResolvedAgents(
  currentAgents: ResolvedAgentView[],
  managedAgentId: string
): RemoveAgentResult {
  const removedAgent = currentAgents.find((agent) => agent.managedAgentId === managedAgentId);

  const nextAgents = currentAgents.map((agent) => {
    if (agent.managedAgentId !== managedAgentId) {
      return agent;
    }

    const statusLabel = agent.sourceScope === "manual" ? "Saved" : "Discovered";
    const summary =
      agent.sourceScope === "manual"
        ? "Saved manually and ready to import back into AgentDock."
        : `Detected ${agent.name} and ready to import into AgentDock.`;

    return {
      ...agent,
      managed: false,
      enabled: false,
      hidden: agent.sourceScope === "manual",
      statusLabel,
      summary,
    };
  });

  return {
    removedAgentId: removedAgent?.id ?? "",
    resolvedAgents: nextAgents,
  };
}

export function deleteAgentFromResolvedAgents(
  currentAgents: ResolvedAgentView[],
  managedAgentId: string
): DeleteAgentResult {
  const deletedAgent = currentAgents.find((agent) => agent.managedAgentId === managedAgentId);

  return {
    deletedAgentId: deletedAgent?.id ?? "",
    resolvedAgents: currentAgents.filter((agent) => agent.managedAgentId !== managedAgentId),
  };
}

function createManualAgentId(draft: ManualAgentDraft) {
  return (
    draft.name
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-") || "manual-agent"
  );
}

export function createManualResolvedAgent(
  currentAgents: ResolvedAgentView[],
  draft: ManualAgentDraft
): CreateAgentResult {
  const idSuffix = createManualAgentId(draft);
  const now = new Date().toISOString();
  const agent: ResolvedAgentView = {
    id: `agent-${idSuffix}`,
    discoveryId: `discovery-${idSuffix}`,
    fingerprint: `${draft.provider}-${idSuffix}`,
    provider: draft.provider,
    name: draft.name.trim(),
    role: "Manually managed agent",
    rootPath: draft.rootPath.trim(),
    configPath: draft.configPath.trim() || undefined,
    sourceScope: "manual",
    managed: true,
    managedAgentId: `managed-${idSuffix}`,
    enabled: true,
    hidden: false,
    health: "ok",
    status: "discovered",
    statusLabel: "Managed",
    summary: "Created manually and ready for local resource management.",
    groupId: "assistant",
    resourceCounts: { skill: 0, mcp: 0, subagent: 0 },
    conflictIds: [],
    lastScannedAt: now,
  };

  return {
    agent,
    resolvedAgents: [agent, ...currentAgents],
  };
}
