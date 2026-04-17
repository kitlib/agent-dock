import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  checkMarketplaceSkillUpdate,
  getLocalSkillDetail,
  listLocalSkills,
} from "@/features/agents/api";
import type { SkillResource, SkillScanTarget } from "@/features/agents/types";

const AGENT_SKILLS_QUERY_KEY = "agent-skills";
const AGENT_SKILL_DETAIL_QUERY_KEY = "agent-skill-detail";
const MARKETPLACE_SKILL_UPDATE_QUERY_KEY = "marketplace-skill-update";

function normalizeSkills(skills: SkillResource[]): SkillResource[] {
  return skills.map((skill) => ({ ...skill, markdown: skill.markdown ?? "" }));
}

function buildSkillTargetKey(targets: SkillScanTarget[]): string {
  return targets.map((target) => `${target.agentId}:${target.source}:${target.rootPath}`).join("|");
}

export function useAgentSkillsQuery(scopeKey: string, targets: SkillScanTarget[]) {
  const targetKey = buildSkillTargetKey(targets);

  const query = useQuery({
    queryKey: [AGENT_SKILLS_QUERY_KEY, scopeKey, targetKey],
    enabled: scopeKey.length > 0,
    queryFn: async () => {
      if (targets.length === 0) {
        return [] as SkillResource[];
      }

      return normalizeSkills(await listLocalSkills(targets));
    },
    staleTime: 0,
    gcTime: 30 * 60 * 1000,
    refetchOnWindowFocus: false,
    retry: 0,
  });

  return {
    skills: query.data ?? [],
    isFetchingSkills: query.isFetching,
    skillTargetKey: targetKey,
  };
}

export function useAgentSkillDetailQuery(
  scopeKey: string,
  skillId: string,
  targets: SkillScanTarget[],
  enabled: boolean
) {
  const targetKey = buildSkillTargetKey(targets);

  return useQuery({
    queryKey: [AGENT_SKILL_DETAIL_QUERY_KEY, scopeKey, skillId, targetKey],
    enabled: enabled && scopeKey.length > 0 && skillId.length > 0,
    queryFn: async () => {
      return getLocalSkillDetail(targets, skillId);
    },
    staleTime: 0,
    gcTime: 30 * 60 * 1000,
    refetchOnWindowFocus: false,
    retry: 0,
  });
}

export function useRefreshAgentSkills() {
  const queryClient = useQueryClient();

  return (scopeKey: string, skillId?: string) => {
    if (!scopeKey) {
      return;
    }

    void queryClient.invalidateQueries({
      queryKey: [AGENT_SKILLS_QUERY_KEY, scopeKey],
    });

    if (skillId) {
      void queryClient.invalidateQueries({
        queryKey: [AGENT_SKILL_DETAIL_QUERY_KEY, scopeKey, skillId],
      });
    }
  };
}

export function useMarketplaceSkillUpdateQuery(
  skillPath: string,
  entryFilePath: string,
  enabled: boolean
) {
  return useQuery({
    queryKey: [MARKETPLACE_SKILL_UPDATE_QUERY_KEY, skillPath, entryFilePath],
    enabled: enabled && skillPath.length > 0 && entryFilePath.length > 0,
    queryFn: async () => checkMarketplaceSkillUpdate(skillPath, entryFilePath),
    staleTime: 0,
    gcTime: 30 * 60 * 1000,
    refetchOnWindowFocus: false,
    retry: 0,
  });
}
