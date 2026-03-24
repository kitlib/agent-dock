export type MarketplaceKind = "skill" | "mcp" | "subagent";
export type InstallState = "install" | "installed" | "update";

export type MarketplaceItem = {
  id: string;
  kind: MarketplaceKind;
  name: string;
  summary: string;
  author: string;
  source: string;
  version: string;
  downloads: number;
  updatedAt: string;
  installState: InstallState;
  description: string;
  highlights: string[];
};
