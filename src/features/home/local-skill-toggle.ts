import type {
  AgentDiscoveryItem,
  LocalDiscoveryItem,
  SkillResource,
} from "@/features/agents/types";

type LocalSkillActionTarget = {
  enabled: boolean;
  entryFilePath: string;
  id: string;
  skillPath: string;
};

export type LocalSkillToggleTarget = LocalSkillActionTarget;
export type LocalSkillDeleteTarget = LocalSkillActionTarget;

function getLocalSkillActionTarget(
  resource: AgentDiscoveryItem | null | undefined
): LocalSkillActionTarget | null {
  if (!resource || resource.kind !== "skill" || resource.origin !== "local") {
    return null;
  }

  const localSkill = resource as LocalDiscoveryItem & SkillResource;
  const entryFilePath = localSkill.entryFilePath?.trim() ?? "";
  const skillPath = localSkill.skillPath?.trim() ?? "";

  if (!skillPath || !entryFilePath) {
    return null;
  }

  return {
    enabled: localSkill.enabled,
    entryFilePath,
    id: localSkill.id,
    skillPath,
  };
}

export function getLocalSkillToggleTarget(
  resource: AgentDiscoveryItem | null | undefined
): LocalSkillToggleTarget | null {
  const target = getLocalSkillActionTarget(resource);

  if (!target || target.skillPath === target.entryFilePath) {
    return null;
  }

  return target;
}

export function getLocalSkillDeleteTarget(
  resource: AgentDiscoveryItem | null | undefined
): LocalSkillDeleteTarget | null {
  return getLocalSkillActionTarget(resource);
}
