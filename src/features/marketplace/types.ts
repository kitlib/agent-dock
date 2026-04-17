export type MarketplaceKind = "skill" | "mcp" | "subagent";
export type InstallState = "install" | "installed" | "update";
export type MarketplaceInstallMethod = "skillsh" | "github";

export type MarketplaceItem = {
  id: string;
  kind: MarketplaceKind;
  name: string;
  skillId?: string;
  author: string;
  source: string;
  version: string;
  installs: number;
  updatedAt: string;
  installState: InstallState;
  description: string;
  highlights: string[];
  url?: string;
  markdown?: string;
};

export type MarketplaceQueryResult = {
  items: MarketplaceItem[];
  totalSkills?: number;
  hasMore: boolean;
  page: number;
};

export type MarketplaceSkillDetail = {
  description: string;
  markdown: string;
  rawMarkdown: string;
};

export type MarketplaceInstallResult = {
  skillPath: string;
  entryFilePath: string;
};

export type MarketplaceInstallPreview = {
  skillPath: string;
  entryFilePath: string;
  hasConflict: boolean;
  existingPath?: string;
};

export type MarketplaceSkillUpdateCheck = {
  managed: boolean;
  hasUpdate: boolean;
  source?: string;
  skillId?: string;
};
