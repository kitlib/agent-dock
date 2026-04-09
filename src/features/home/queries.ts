import { useQuery, useQueryClient } from "@tanstack/react-query";
import { getLocalSkillDetail, listLocalSkills } from "@/features/agents/api";
import type { SkillResource, SkillScanTarget } from "@/features/agents/types";

const AGENT_SKILLS_QUERY_KEY = "agent-skills";
const AGENT_SKILL_DETAIL_QUERY_KEY = "agent-skill-detail";

function normalizeSkills(skills: SkillResource[]): SkillResource[] {
  return skills.map((skill) => ({ ...skill, markdown: skill.markdown ?? "" }));
}

function buildSkillTargetKey(targets: SkillScanTarget[]): string {
  return targets.map((target) => `${target.source}:${target.rootPath}`).join("|");
}

export function useAgentSkillsQuery(agentId: string, targets: SkillScanTarget[]) {
  const targetKey = buildSkillTargetKey(targets);

  const query = useQuery({
    queryKey: [AGENT_SKILLS_QUERY_KEY, agentId, targetKey],
    enabled: agentId.length > 0,
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
  agentId: string,
  skillId: string,
  targets: SkillScanTarget[],
  enabled: boolean
) {
  const targetKey = buildSkillTargetKey(targets);

  return useQuery({
    queryKey: [AGENT_SKILL_DETAIL_QUERY_KEY, agentId, skillId, targetKey],
    enabled: enabled && agentId.length > 0 && skillId.length > 0,
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

  return (agentId: string, skillId?: string) => {
    if (!agentId) {
      return;
    }

    void queryClient.invalidateQueries({
      queryKey: [AGENT_SKILLS_QUERY_KEY, agentId],
    });

    if (skillId) {
      void queryClient.invalidateQueries({
        queryKey: [AGENT_SKILL_DETAIL_QUERY_KEY, agentId, skillId],
      });
    }
  };
}
