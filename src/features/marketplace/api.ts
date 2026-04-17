import { invoke } from "@tauri-apps/api/core";
import type { LocalSkillCopyTargetAgent } from "@/features/agents/types";
import type {
  MarketplaceInstallMethod,
  MarketplaceInstallPreview,
  MarketplaceInstallResult,
  MarketplaceQueryResult,
  MarketplaceSkillDetail,
  MarketplaceSkillUpdateCheck,
} from "./types";

export async function fetchSkillsshLeaderboard(board = "all-time", page = 0) {
  return invoke<MarketplaceQueryResult>("fetch_skillssh_leaderboard", { board, page });
}

export async function searchSkillsshMarketplace(query: string, limit = 100, page = 0) {
  return invoke<MarketplaceQueryResult>("search_skillssh_marketplace", { query, limit, page });
}

export async function getSkillsshMarketplaceDetail(source: string, skillId: string) {
  return invoke<MarketplaceSkillDetail>("get_skillssh_marketplace_detail", { source, skillId });
}

export async function installSkillsshMarketplaceItem(
  source: string,
  skillId: string,
  name: string,
  description: string,
  targetAgent: LocalSkillCopyTargetAgent,
  installMethod: MarketplaceInstallMethod,
  overwrite = false
) {
  return invoke<MarketplaceInstallResult>("install_skillssh_marketplace_item", {
    request: {
      source,
      skillId,
      name,
      description,
      targetAgent,
      installMethod,
      overwrite,
    },
  });
}

export async function previewSkillsshMarketplaceInstall(
  source: string,
  skillId: string,
  name: string,
  description: string,
  targetAgent: LocalSkillCopyTargetAgent,
  installMethod: MarketplaceInstallMethod
) {
  return invoke<MarketplaceInstallPreview>("preview_skillssh_marketplace_install", {
    request: {
      source,
      skillId,
      name,
      description,
      targetAgent,
      installMethod,
      overwrite: false,
    },
  });
}

export async function checkLocalMarketplaceSkillUpdate(skillPath: string, entryFilePath: string) {
  return invoke<MarketplaceSkillUpdateCheck>("check_local_marketplace_skill_update", {
    skillPath,
    entryFilePath,
  });
}
