import { useEffect, useMemo, useState } from "react";
import { useAgentDiscovery } from "@/features/agents/use-agent-discovery";
import { useAgentManagement } from "@/features/agents/use-agent-management";
import { openSkillFolder } from "@/features/agents/api";
import type {
  CreateAgentResult,
  DeleteAgentResult,
  ImportAgentsResult,
  RemoveAgentResult,
  ResolvedAgentView,
} from "@/features/agents/types";
import { toSkillScanTargets } from "@/features/home/skill-targets";
import { useAgentSkillDetailQuery, useAgentSkillsQuery, useRefreshAgentSkills } from "@/features/home/queries";
import { useResourceBrowser } from "@/features/home/use-resource-browser";

type WorkspaceMode = "browse" | "adding";

function buildManagedAgents(resolvedAgents: ResolvedAgentView[]) {
  return resolvedAgents
    .filter((agent) => agent.managed)
    .map((agent) => ({
      managedAgentId: agent.managedAgentId ?? `managed-${agent.id}`,
      fingerprint: agent.fingerprint,
      alias: agent.alias,
      enabled: agent.enabled,
      hidden: agent.hidden,
      importedAt: agent.lastScannedAt,
      source: "manual-imported" as const,
    }));
}

function findFirstVisibleManagedAgentId(resolvedAgents: ResolvedAgentView[]) {
  return resolvedAgents.find((agent) => agent.managed && !agent.hidden)?.id ?? "";
}

function isManagedVisibleAgent(agent: ResolvedAgentView) {
  return agent.managed && !agent.hidden;
}

function matchesAgentSearch(agent: ResolvedAgentView, normalizedSearch: string) {
  if (normalizedSearch.length === 0) {
    return true;
  }

  return (
    agent.name.toLowerCase().includes(normalizedSearch) ||
    agent.role.toLowerCase().includes(normalizedSearch) ||
    agent.summary.toLowerCase().includes(normalizedSearch) ||
    agent.rootPath.toLowerCase().includes(normalizedSearch)
  );
}

function getSelectedAgent(
  filteredAgents: ResolvedAgentView[],
  managedVisibleAgents: ResolvedAgentView[],
  selectedAgentId: string
) {
  return (
    filteredAgents.find((agent) => agent.id === selectedAgentId) ??
    managedVisibleAgents.find((agent) => agent.id === selectedAgentId) ??
    filteredAgents[0] ??
    managedVisibleAgents[0] ??
    null
  );
}

export function useHomeWorkspace() {
  const [workspaceMode, setWorkspaceMode] = useState<WorkspaceMode>("browse");
  const [search, setSearch] = useState("");
  const [selectedAgentId, setSelectedAgentId] = useState("");

  const {
    discoveredAgents,
    discoveryState,
    managedAgents,
    resolvedAgents,
    setDiscoveryState,
    setManagedAgents,
    setResolvedAgents,
  } = useAgentDiscovery();

  const { refreshAgents } = useAgentManagement({
    setDiscoveryState,
    setManagedAgents,
    setResolvedAgents,
  });

  const resetWorkspaceSelection = () => {
    setWorkspaceMode("adding");
  };

  const syncManagedState = (nextResolvedAgents: ResolvedAgentView[]) => {
    setResolvedAgents(nextResolvedAgents);
    setManagedAgents(buildManagedAgents(nextResolvedAgents));
    resetWorkspaceSelection();
  };

  const syncImportedAgents = ({ resolvedAgents: nextResolvedAgents }: ImportAgentsResult) => {
    syncManagedState(nextResolvedAgents);
  };

  const syncCreatedAgent = ({ resolvedAgents: nextResolvedAgents }: CreateAgentResult) => {
    syncManagedState(nextResolvedAgents);
  };

  const syncRemovedAgent = ({ removedAgentId, resolvedAgents: nextResolvedAgents }: RemoveAgentResult) => {
    syncManagedState(nextResolvedAgents);
    if (removedAgentId && selectedAgentId === removedAgentId) {
      setSelectedAgentId(findFirstVisibleManagedAgentId(nextResolvedAgents));
    }
  };

  const syncDeletedAgent = ({ deletedAgentId, resolvedAgents: nextResolvedAgents }: DeleteAgentResult) => {
    syncManagedState(nextResolvedAgents);
    if (deletedAgentId && selectedAgentId === deletedAgentId) {
      setSelectedAgentId(findFirstVisibleManagedAgentId(nextResolvedAgents));
    }
  };

  const normalizedSearch = search.trim().toLowerCase();

  const managedVisibleAgents = useMemo(
    () => resolvedAgents.filter(isManagedVisibleAgent),
    [resolvedAgents]
  );

  const filteredAgents = useMemo(() => {
    return managedVisibleAgents.filter((agent) => matchesAgentSearch(agent, normalizedSearch));
  }, [managedVisibleAgents, normalizedSearch]);

  const selectedAgent = getSelectedAgent(filteredAgents, managedVisibleAgents, selectedAgentId);

  const currentSelectedAgentId = selectedAgent?.id ?? "";
  const selectedAgentSkillScanTargets = useMemo(
    () => (selectedAgent ? toSkillScanTargets(selectedAgent) : []),
    [selectedAgent]
  );

  const { skills } = useAgentSkillsQuery(currentSelectedAgentId, selectedAgentSkillScanTargets);

  useEffect(() => {
    console.log("[skills] workspace selection snapshot", {
      selectedAgentId,
      resolvedAgentIds: resolvedAgents.map((agent) => agent.id),
      managedVisibleAgentIds: managedVisibleAgents.map((agent) => agent.id),
      railAgentIds: filteredAgents.map((agent) => agent.id),
      effectiveSelectedAgentId: currentSelectedAgentId || null,
    });
  }, [currentSelectedAgentId, filteredAgents, managedVisibleAgents, resolvedAgents, selectedAgentId]);

  useEffect(() => {
    console.log("[skills] visible skills snapshot", {
      selectedAgentId: currentSelectedAgentId || null,
      localSkillOwners: skills.map((skill) => ({
        id: skill.id,
        ownerAgentId: skill.ownerAgentId ?? null,
      })),
      visibleSkillIds: skills.map((skill) => skill.id),
    });
  }, [currentSelectedAgentId, skills]);

  const resourceBrowser = useResourceBrowser(search, selectedAgent, skills);

  const selectedSkillId =
    resourceBrowser.selectedResourceBase?.kind === "skill" &&
    resourceBrowser.selectedResourceBase.origin === "local"
      ? resourceBrowser.selectedResourceBase.id
      : "";

  const selectedSkillDetailQuery = useAgentSkillDetailQuery(
    currentSelectedAgentId,
    selectedSkillId,
    selectedAgentSkillScanTargets,
    resourceBrowser.activeKind === "skill"
  );

  const selectedResource =
    resourceBrowser.selectedResourceBase?.kind === "skill" &&
    resourceBrowser.selectedResourceBase.origin === "local"
      ? ({
          ...resourceBrowser.selectedResourceBase,
          ...selectedSkillDetailQuery.data,
        })
      : resourceBrowser.selectedResource;

  const refreshSkills = useRefreshAgentSkills();

  const openSelectedSkillFolder = (skillPath: string) => {
    void openSkillFolder(skillPath).catch(() => undefined);
  };

  const selectAgent = (id: string) => {
    console.log("[skills] user selected agent", { nextSelectedAgentId: id });
    setSelectedAgentId(id);
    setWorkspaceMode("browse");
  };

  const enterAddingMode = () => {
    setWorkspaceMode("adding");
  };

  const exitAddingMode = () => {
    setWorkspaceMode("browse");
  };

  return {
    activeKind: resourceBrowser.activeKind,
    checkedIds: resourceBrowser.checkedIds,
    clearChecked: resourceBrowser.clearChecked,
    discoveredAgents,
    discoveryState,
    filteredAgents,
    filteredResources: resourceBrowser.filteredResources,
    managedAgents,
    managedAgentsForView: resolvedAgents,
    onCreateAgentSuccess: syncCreatedAgent,
    onDeleteAgentSuccess: syncDeletedAgent,
    onImportAgentsSuccess: syncImportedAgents,
    onOpenSkillFolder: openSelectedSkillFolder,
    onRemoveAgentSuccess: syncRemovedAgent,
    search,
    refreshAgents,
    refreshSkills: (skillId?: string) =>
      refreshSkills(currentSelectedAgentId, skillId ?? selectedSkillId),
    selectKind: resourceBrowser.selectKind,
    selectResource: resourceBrowser.selectResource,
    selectedAgent,
    selectedAgentId: currentSelectedAgentId || selectedAgentId,
    selectedResource,
    selectedResourceId: resourceBrowser.selectedResourceId,
    setSearch,
    setSelectedAgentId: selectAgent,
    toggleChecked: resourceBrowser.toggleChecked,
    updateMarketplaceInstallState: resourceBrowser.updateMarketplaceInstallState,
    workspaceMode,
    enterAddingMode,
    exitAddingMode,
  };
}
