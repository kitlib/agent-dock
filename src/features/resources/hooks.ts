import { useMemo, useState } from "react";
import { resourceItems } from "./mock";
import type { ResourceDetail, ResourceKind } from "./types";

export function useResourcesPrototype() {
  const [activeKind, setActiveKind] = useState<ResourceKind>("skill");
  const [search, setSearch] = useState("");
  const [selectedId, setSelectedId] = useState(
    resourceItems.find((item) => item.kind === "skill")?.id ?? ""
  );

  const filteredItems = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    return resourceItems.filter((item) => {
      if (item.kind !== activeKind) return false;
      if (!keyword) return true;
      return (
        item.name.toLowerCase().includes(keyword) || item.summary.toLowerCase().includes(keyword)
      );
    });
  }, [activeKind, search]);

  const selectedItem =
    filteredItems.find((item) => item.id === selectedId) ?? filteredItems[0] ?? null;

  const selectKind = (kind: ResourceKind) => {
    setActiveKind(kind);
    setSelectedId(resourceItems.find((item) => item.kind === kind)?.id ?? "");
  };

  const selectItem = (item: ResourceDetail | null) => {
    setSelectedId(item?.id ?? "");
  };

  return {
    activeKind,
    selectKind,
    search,
    setSearch,
    filteredItems,
    selectedItem,
    selectedId,
    selectItem,
  };
}
