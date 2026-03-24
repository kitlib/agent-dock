import { useMemo, useState } from "react";
import { agents, agentGroups } from "./mock";

export function useAgentSelection(search: string) {
  const [selectedGroupId, setSelectedGroupId] = useState(agentGroups[0]?.id ?? "all");
  const [selectedAgentId, setSelectedAgentId] = useState(agents[0]?.id ?? "");

  const normalizedSearch = search.trim().toLowerCase();

  const filteredAgents = useMemo(() => {
    return agents.filter((agent) => {
      const matchGroup = selectedGroupId === "all" || agent.groupId === selectedGroupId;
      const matchSearch =
        normalizedSearch.length === 0 ||
        agent.name.toLowerCase().includes(normalizedSearch) ||
        agent.role.toLowerCase().includes(normalizedSearch) ||
        agent.summary.toLowerCase().includes(normalizedSearch);

      return matchGroup && matchSearch;
    });
  }, [normalizedSearch, selectedGroupId]);

  const selectedAgent =
    filteredAgents.find((agent) => agent.id === selectedAgentId) ?? filteredAgents[0] ?? null;

  return {
    agentGroups,
    filteredAgents,
    selectedAgent,
    selectedAgentId,
    selectedGroupId,
    setSelectedAgentId,
    setSelectedGroupId,
  };
}
