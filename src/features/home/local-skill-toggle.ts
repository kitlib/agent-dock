import type {
  AgentDiscoveryItem,
  LocalDiscoveryItem,
  SkillResource,
} from "@/features/agents/types";

export type LocalSkillToggleTarget = {
  enabled: boolean;
  entryFilePath: string;
  id: string;
  skillPath: string;
};

export function getLocalSkillToggleTarget(
  resource: AgentDiscoveryItem | null | undefined
): LocalSkillToggleTarget | null {
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
