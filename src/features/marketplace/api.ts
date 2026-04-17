import { invoke } from "@tauri-apps/api/core";
import type { LocalSkillCopyTargetAgent } from "@/features/agents/types";
import type {
  MarketplaceInstallPreview,
  MarketplaceInstallResult,
  MarketplaceItem,
  MarketplaceSkillDetail,
  MarketplaceSkillUpdateCheck,
} from "./types";

export async function fetchSkillsshLeaderboard(board = "hot") {
  return invoke<MarketplaceItem[]>("fetch_skillssh_leaderboard", { board });
}

export async function searchSkillsshMarketplace(query: string, limit = 60) {
  return invoke<MarketplaceItem[]>("search_skillssh_marketplace", { query, limit });
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
  overwrite = false
) {
  return invoke<MarketplaceInstallResult>("install_skillssh_marketplace_item", {
    request: {
      source,
      skillId,
      name,
      description,
      targetAgent,
      overwrite,
    },
  });
}

export async function previewSkillsshMarketplaceInstall(
  source: string,
  skillId: string,
  name: string,
  description: string,
  targetAgent: LocalSkillCopyTargetAgent
) {
  return invoke<MarketplaceInstallPreview>("preview_skillssh_marketplace_install", {
    request: {
      source,
      skillId,
      name,
      description,
      targetAgent,
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
