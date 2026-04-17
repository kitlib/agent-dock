import { useQuery } from "@tanstack/react-query";
import {
  fetchSkillsshLeaderboard,
  getSkillsshMarketplaceDetail,
  searchSkillsshMarketplace,
} from "./api";

const SKILLSH_MARKETPLACE_QUERY_KEY = "skillssh-marketplace";

export function useSkillsshMarketplaceQuery(query: string, enabled: boolean) {
  const normalizedQuery = query.trim();

  return useQuery({
    queryKey: [SKILLSH_MARKETPLACE_QUERY_KEY, normalizedQuery],
    enabled,
    queryFn: async () => {
      if (normalizedQuery.length > 0) {
        return searchSkillsshMarketplace(normalizedQuery, 60);
      }

      return fetchSkillsshLeaderboard();
    },
    staleTime: 2 * 60 * 1000,
    gcTime: 30 * 60 * 1000,
    refetchOnWindowFocus: false,
    retry: 0,
  });
}

export function useSkillsshMarketplaceDetailQuery(
  source: string | undefined,
  skillId: string | undefined,
  enabled: boolean
) {
  return useQuery({
    queryKey: [SKILLSH_MARKETPLACE_QUERY_KEY, "detail", source, skillId],
    enabled: enabled && Boolean(source) && Boolean(skillId),
    queryFn: async () => getSkillsshMarketplaceDetail(source!, skillId!),
    staleTime: 5 * 60 * 1000,
    gcTime: 30 * 60 * 1000,
    refetchOnWindowFocus: false,
    retry: 0,
  });
}
