import { useMemo, useState } from "react";
import { marketplaceItems, marketplaceSources } from "./mock";
import type { MarketplaceItem, MarketplaceKind } from "./types";

export function useMarketplacePrototype() {
  const [activeKind, setActiveKind] = useState<MarketplaceKind>("skill");
  const [search, setSearch] = useState("");
  const [selectedSource, setSelectedSource] = useState(marketplaceSources[0]);
  const [selectedId, setSelectedId] = useState(
    marketplaceItems.find((item) => item.kind === "skill")?.id ?? ""
  );
  const [bindAfterInstall, setBindAfterInstall] = useState(false);
  const [installStates, setInstallStates] = useState<
    Record<string, MarketplaceItem["installState"]>
  >(Object.fromEntries(marketplaceItems.map((item) => [item.id, item.installState])));

  const filteredItems = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    return marketplaceItems.filter((item) => {
      if (item.kind !== activeKind) return false;
      if (selectedSource !== "All Sources" && item.source !== selectedSource) return false;
      if (!keyword) return true;
      return (
        item.name.toLowerCase().includes(keyword) || item.summary.toLowerCase().includes(keyword)
      );
    });
  }, [activeKind, search, selectedSource]);

  const selectedItem =
    filteredItems.find((item) => item.id === selectedId) ?? filteredItems[0] ?? null;

  const selectKind = (kind: MarketplaceKind) => {
    setActiveKind(kind);
    setSelectedId(marketplaceItems.find((item) => item.kind === kind)?.id ?? "");
  };

  const updateInstallState = (id: string) => {
    setInstallStates((current) => {
      const state = current[id];
      const next =
        state === "install" ? "installed" : state === "installed" ? "installed" : "installed";
      return { ...current, [id]: next };
    });
  };

  return {
    marketplaceSources,
    activeKind,
    selectKind,
    search,
    setSearch,
    selectedSource,
    setSelectedSource,
    filteredItems: filteredItems.map((item) => ({
      ...item,
      installState: installStates[item.id],
    })),
    selectedItem: selectedItem
      ? {
          ...selectedItem,
          installState: installStates[selectedItem.id],
        }
      : null,
    selectedId,
    setSelectedId,
    bindAfterInstall,
    setBindAfterInstall,
    updateInstallState,
  };
}
