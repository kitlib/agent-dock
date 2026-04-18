import { agentTypeMeta, supportsAgentMcp } from "@/features/agents/agent-meta";
import type {
  AgentTypeId,
  McpScanTarget,
  ResolvedAgentView,
  SkillResource,
  SkillScanTarget,
} from "@/features/agents/types";

function trimTrailingSlash(value: string) {
  return value.replace(/\/+$/, "");
}

function trimLeadingSlash(value: string) {
  return value.replace(/^\/+/, "");
}

function buildScanPath(agent: ResolvedAgentView, relativePath: string) {
  const rootPath = trimTrailingSlash(agent.rootPath);
  const normalizedRelativePath = trimLeadingSlash(trimTrailingSlash(relativePath));
  return `${rootPath}/${normalizedRelativePath}`;
}

export function toSkillScanTargets(agent: ResolvedAgentView): SkillScanTarget[] {
  if (!agent.managed || agent.hidden || !agent.rootPath) {
    return [];
  }

  const meta = agentTypeMeta[agent.agentType as AgentTypeId];
  const displayName = agent.alias ?? agent.name;
  const targets: SkillScanTarget[] = [];

  if (meta?.skills) {
    targets.push({
      agentId: agent.id,
      agentType: agent.agentType,
      rootPath: buildScanPath(agent, meta.skills),
      displayName,
      source: "skills",
    });
  }

  if (meta?.commands) {
    targets.push({
      agentId: agent.id,
      agentType: agent.agentType,
      rootPath: buildScanPath(agent, meta.commands),
      displayName,
      source: "commands",
    });
  }

  return targets;
}

export function toSkillScanTargetsForAgents(agents: ResolvedAgentView[]): SkillScanTarget[] {
  const seenKeys = new Set<string>();
  const targets: SkillScanTarget[] = [];

  agents.forEach((agent) => {
    toSkillScanTargets(agent).forEach((target) => {
      const key = `${target.agentId}:${target.source}:${target.rootPath}`;
      if (seenKeys.has(key)) {
        return;
      }

      seenKeys.add(key);
      targets.push(target);
    });
  });

  return targets;
}

export function toMcpScanTarget(agent: ResolvedAgentView): McpScanTarget[] {
  if (!agent.managed || agent.hidden || !agent.rootPath) {
    return [];
  }

  const meta = agentTypeMeta[agent.agentType as AgentTypeId];
  if (!meta || !supportsAgentMcp(agent.agentType as AgentTypeId)) {
    return [];
  }

  return [
    {
      agentId: agent.id,
      agentType: agent.agentType,
      rootPath: trimTrailingSlash(agent.rootPath),
      displayName: agent.alias ?? agent.name,
    },
  ];
}

export function toMcpScanTargetsForAgents(agents: ResolvedAgentView[]): McpScanTarget[] {
  const seenKeys = new Set<string>();
  const targets: McpScanTarget[] = [];

  agents.forEach((agent) => {
    toMcpScanTarget(agent).forEach((target) => {
      const key = `${target.agentId}:${target.rootPath}`;
      if (seenKeys.has(key)) {
        return;
      }

      seenKeys.add(key);
      targets.push(target);
    });
  });

  return targets;
}

export function filterSkillsForAgent(skills: SkillResource[], selectedAgentId: string | null) {
  if (!selectedAgentId) {
    return [];
  }

  return skills.filter((skill) => skill.ownerAgentId === selectedAgentId);
}
