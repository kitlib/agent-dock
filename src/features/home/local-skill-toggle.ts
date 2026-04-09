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

function getSkillEntryFileName(path: string | undefined): string {
  if (!path) {
    return "";
  }

  return path.split(/[\\/]/).pop() ?? "";
}

export function getLocalSkillToggleTarget(
  resource: AgentDiscoveryItem | null | undefined
): LocalSkillToggleTarget | null {
  if (!resource || resource.kind !== "skill" || resource.origin !== "local") {
    return null;
  }

  const localSkill = resource as LocalDiscoveryItem & SkillResource;
  const entryFilePath = localSkill.entryFilePath?.trim() ?? "";
  const skillPath = localSkill.skillPath?.trim() ?? "";
  const entryFileName = getSkillEntryFileName(entryFilePath);

  if (
    !skillPath ||
    !entryFilePath ||
    skillPath === entryFilePath ||
    (entryFileName !== "SKILL.md" && entryFileName !== "SKILL.md.disabled")
  ) {
    return null;
  }

  return {
    enabled: localSkill.enabled,
    entryFilePath,
    id: localSkill.id,
    skillPath,
  };
}
