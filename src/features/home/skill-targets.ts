import { agentMeta } from "@/features/agents/agent-meta";
import type { AgentId, ResolvedAgentView, SkillResource, SkillScanTarget } from "@/features/agents/types";

function trimTrailingSlash(value: string) {
  return value.replace(/\/+$/, "");
}

function trimLeadingSlash(value: string) {
  return value.replace(/^\/+/, "");
}

export function buildSkillScanPath(agent: ResolvedAgentView) {
  const skillRelativePath = agentMeta[agent.provider as AgentId]?.skills;
  if (!skillRelativePath) {
    return null;
  }

  const rootPath = trimTrailingSlash(agent.rootPath);
  const relativePath = trimLeadingSlash(trimTrailingSlash(skillRelativePath));
  return `${rootPath}/${relativePath}`;
}

export function toSkillScanTarget(agent: ResolvedAgentView): SkillScanTarget | null {
  if (!agent.managed || agent.hidden || !agent.rootPath) {
    return null;
  }

  const rootPath = buildSkillScanPath(agent);
  if (!rootPath) {
    return null;
  }

  return {
    agentId: agent.id,
    provider: agent.provider,
    rootPath,
    displayName: agent.alias ?? agent.name,
  };
}

export function filterSkillsForAgent(skills: SkillResource[], selectedAgentId: string | null) {
  if (!selectedAgentId) {
    return [];
  }

  return skills.filter((skill) => skill.ownerAgentId === selectedAgentId);
}
