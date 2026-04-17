import { useInfiniteQuery, useQuery } from "@tanstack/react-query";
import {
  fetchSkillsshLeaderboard,
  getSkillsshMarketplaceDetail,
  searchSkillsshMarketplace,
} from "./api";
import type { MarketplaceQueryResult } from "./types";

const SKILLSH_MARKETPLACE_QUERY_KEY = "skillssh-marketplace";
const MARKETPLACE_PAGE_SIZE = 100;

export function useSkillsshMarketplaceQuery(query: string, enabled: boolean) {
  const normalizedQuery = query.trim();

  return useInfiniteQuery({
    queryKey: [SKILLSH_MARKETPLACE_QUERY_KEY, normalizedQuery],
    enabled,
    initialPageParam: 0,
    queryFn: async ({ pageParam }) => {
      if (normalizedQuery.length > 0) {
        return searchSkillsshMarketplace(normalizedQuery, MARKETPLACE_PAGE_SIZE, pageParam);
      }

      return fetchSkillsshLeaderboard("all-time", pageParam);
    },
    getNextPageParam: (lastPage, allPages) => {
      if (!lastPage.hasMore) {
        return undefined;
      }

      return allPages.length;
    },
    select: (data) => {
      const items = data.pages.flatMap((page) => page.items);
      const lastPage = data.pages[data.pages.length - 1];

      return {
        items,
        totalSkills: data.pages.find((page) => page.totalSkills != null)?.totalSkills,
        hasMore: lastPage?.hasMore ?? false,
        page: lastPage?.page ?? 0,
      } satisfies MarketplaceQueryResult;
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
