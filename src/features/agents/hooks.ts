import { useState } from "react";
import { useAgentSelection } from "./use-agent-selection";
import { useResourceDiscovery } from "./use-resource-discovery";

export function useAgentsPrototype() {
  const [search, setSearch] = useState("");

  const {
    agentGroups,
    filteredAgents,
    selectedAgent,
    selectedAgentId,
    selectedGroupId,
    setSelectedAgentId,
    setSelectedGroupId,
  } = useAgentSelection(search);

  const {
    activeKind,
    checkedIds,
    clearChecked,
    filteredResources,
    selectKind,
    selectResource,
    selectedResource,
    selectedResourceId,
    toggleChecked,
    updateMarketplaceInstallState,
  } = useResourceDiscovery(search);

  return {
    agentGroups,
    filteredAgents,
    selectedGroupId,
    setSelectedGroupId,
    selectedAgent,
    selectedAgentId,
    setSelectedAgentId,
    activeKind,
    selectKind,
    search,
    setSearch,
    filteredResources,
    selectedResource,
    selectedResourceId,
    selectResource,
    checkedIds,
    toggleChecked,
    clearChecked,
    updateMarketplaceInstallState,
  };
}
