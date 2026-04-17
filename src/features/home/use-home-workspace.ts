import { useDeferredValue, useEffect, useMemo, useState } from "react";
import { useAgentDiscovery } from "@/features/agents/use-agent-discovery";
import { useAgentManagement } from "@/features/agents/use-agent-management";
import {
  deleteLocalSkill,
  openSkillEntryFile,
  openSkillFolder,
  previewLocalSkillCopy,
  copyLocalSkills,
} from "@/features/agents/api";
import type {
  AgentSelectionScope,
  CreateAgentResult,
  DeleteAgentResult,
  ImportAgentsResult,
  LocalSkillCopySource,
  LocalSkillCopyTargetAgent,
  LocalSkillConflictResolution,
  PreviewLocalSkillCopyResult,
  RemoveAgentResult,
  ResolvedAgentView,
} from "@/features/agents/types";
import { toSkillScanTargets, toSkillScanTargetsForAgents } from "@/features/home/skill-targets";
import {
  useAgentSkillDetailQuery,
  useAgentSkillsQuery,
  useMarketplaceSkillUpdateQuery,
  useRefreshAgentSkills,
} from "@/features/home/queries";
import { useResourceBrowser } from "@/features/home/use-resource-browser";
import { useSkillsshMarketplaceDetailQuery } from "@/features/marketplace/queries";

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
  const deferredSearch = useDeferredValue(search);
  const [selectedScope, setSelectedScope] = useState<AgentSelectionScope>("all");
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

  const syncRemovedAgent = ({
    removedAgentId,
    resolvedAgents: nextResolvedAgents,
  }: RemoveAgentResult) => {
    syncManagedState(nextResolvedAgents);
    if (removedAgentId && selectedAgentId === removedAgentId) {
      setSelectedAgentId(findFirstVisibleManagedAgentId(nextResolvedAgents));
    }
  };

  const syncDeletedAgent = ({
    deletedAgentId,
    resolvedAgents: nextResolvedAgents,
  }: DeleteAgentResult) => {
    syncManagedState(nextResolvedAgents);
    if (deletedAgentId && selectedAgentId === deletedAgentId) {
      setSelectedAgentId(findFirstVisibleManagedAgentId(nextResolvedAgents));
    }
  };

  const managedVisibleAgents = useMemo(
    () => resolvedAgents.filter(isManagedVisibleAgent),
    [resolvedAgents]
  );

  useEffect(() => {
    if (managedVisibleAgents.length === 0) {
      setSelectedAgentId("");
      return;
    }

    if (selectedScope === "all") {
      return;
    }

    if (!managedVisibleAgents.some((agent) => agent.id === selectedAgentId)) {
      setSelectedAgentId(managedVisibleAgents[0]?.id ?? "");
    }
  }, [managedVisibleAgents, selectedAgentId, selectedScope]);

  const selectedAgent =
    selectedScope === "all"
      ? null
      : getSelectedAgent(managedVisibleAgents, managedVisibleAgents, selectedAgentId);

  const currentSelectedAgentId = selectedAgent?.id ?? "";
  const scopeKey = selectedScope === "all" ? "all" : `agent:${currentSelectedAgentId}`;
  const allVisibleManagedAgentSkillScanTargets = useMemo(
    () => toSkillScanTargetsForAgents(managedVisibleAgents),
    [managedVisibleAgents]
  );
  const selectedAgentSkillScanTargets = useMemo(
    () => (selectedAgent ? toSkillScanTargets(selectedAgent) : []),
    [selectedAgent]
  );
  const effectiveSkillScanTargets =
    selectedScope === "all"
      ? allVisibleManagedAgentSkillScanTargets
      : selectedAgentSkillScanTargets;

  const { skills } = useAgentSkillsQuery(scopeKey, effectiveSkillScanTargets);

  useEffect(() => {
    console.log("[skills] workspace selection snapshot", {
      selectedScope,
      selectedAgentId,
      resolvedAgentIds: resolvedAgents.map((agent) => agent.id),
      managedVisibleAgentIds: managedVisibleAgents.map((agent) => agent.id),
      railAgentIds: managedVisibleAgents.map((agent) => agent.id),
      effectiveSelectedAgentId: currentSelectedAgentId || null,
      skillTargetCount: effectiveSkillScanTargets.length,
    });
  }, [
    currentSelectedAgentId,
    effectiveSkillScanTargets.length,
    managedVisibleAgents,
    resolvedAgents,
    selectedAgentId,
    selectedScope,
  ]);

  useEffect(() => {
    console.log("[skills] visible skills snapshot", {
      selectedScope,
      selectedAgentId: currentSelectedAgentId || null,
      localSkillOwners: skills.map((skill) => ({
        id: skill.id,
        ownerAgentId: skill.ownerAgentId ?? null,
      })),
      visibleSkillIds: skills.map((skill) => skill.id),
    });
  }, [currentSelectedAgentId, selectedScope, skills]);

  const resourceBrowser = useResourceBrowser(deferredSearch, selectedAgent, skills);

  const selectedSkillId =
    resourceBrowser.selectedResourceBase?.kind === "skill" &&
    resourceBrowser.selectedResourceBase.origin === "local"
      ? resourceBrowser.selectedResourceBase.id
      : "";
  const selectedLocalSkill =
    resourceBrowser.selectedResourceBase?.kind === "skill" &&
    resourceBrowser.selectedResourceBase.origin === "local"
      ? resourceBrowser.selectedResourceBase
      : null;

  const selectedSkillDetailQuery = useAgentSkillDetailQuery(
    scopeKey,
    selectedSkillId,
    effectiveSkillScanTargets,
    resourceBrowser.activeKind === "skill"
  );
  const selectedMarketplaceSkillDetailQuery = useSkillsshMarketplaceDetailQuery(
    resourceBrowser.selectedResourceBase?.origin === "marketplace" &&
      resourceBrowser.selectedResourceBase.kind === "skill"
      ? resourceBrowser.selectedResourceBase.sourceLabel
      : undefined,
    resourceBrowser.selectedResourceBase?.origin === "marketplace" &&
      resourceBrowser.selectedResourceBase.kind === "skill"
      ? resourceBrowser.selectedResourceBase.skillId
      : undefined,
    resourceBrowser.activeKind === "skill"
  );
  const selectedLocalMarketplaceUpdateQuery = useMarketplaceSkillUpdateQuery(
    selectedLocalSkill?.skillPath ?? "",
    selectedLocalSkill?.entryFilePath ?? "",
    resourceBrowser.activeKind === "skill"
  );
  const selectedLocalMarketplaceUpdate = selectedLocalMarketplaceUpdateQuery.data;

  const selectedResource =
    resourceBrowser.selectedResourceBase?.kind === "skill" &&
    resourceBrowser.selectedResourceBase.origin === "local"
      ? {
          ...resourceBrowser.selectedResourceBase,
          ...selectedSkillDetailQuery.data,
          marketplaceSource: selectedLocalMarketplaceUpdate?.source,
          marketplaceRemoteId: selectedLocalMarketplaceUpdate?.skillId,
          marketplaceHasUpdate: selectedLocalMarketplaceUpdate?.hasUpdate ?? false,
        }
      : resourceBrowser.selectedResourceBase?.kind === "skill" &&
          resourceBrowser.selectedResourceBase.origin === "marketplace"
        ? {
            ...resourceBrowser.selectedResourceBase,
            ...selectedMarketplaceSkillDetailQuery.data,
          }
        : resourceBrowser.selectedResource;

  const refreshSkills = useRefreshAgentSkills();

  const openSelectedSkillFolder = (skillPath: string) => {
    void openSkillFolder(skillPath).catch(() => undefined);
  };

  const openSelectedSkillEntryFile = async (skillPath: string, entryFilePath: string) => {
    await openSkillEntryFile(skillPath, entryFilePath).catch(() => undefined);
  };

  const deleteSelectedLocalSkill = async (skillPath: string, entryFilePath: string) => {
    await deleteLocalSkill(skillPath, entryFilePath);
  };

  const selectAllAgents = () => {
    setSelectedScope("all");
    setWorkspaceMode("browse");
  };

  const selectAgent = (id: string) => {
    console.log("[skills] user selected agent", { nextSelectedAgentId: id });
    setSelectedScope("agent");
    setSelectedAgentId(id);
    setWorkspaceMode("browse");
  };

  const enterAddingMode = () => {
    setWorkspaceMode("adding");
  };

  const exitAddingMode = () => {
    setWorkspaceMode("browse");
  };

  const previewCopy = async (
    sources: LocalSkillCopySource[],
    targetAgent: LocalSkillCopyTargetAgent
  ): Promise<PreviewLocalSkillCopyResult> => {
    return previewLocalSkillCopy(sources, targetAgent);
  };

  const executeCopy = async (
    sources: LocalSkillCopySource[],
    targetAgent: LocalSkillCopyTargetAgent,
    resolutions: LocalSkillConflictResolution[]
  ): Promise<void> => {
    await copyLocalSkills(sources, targetAgent, resolutions);
    refreshSkills(scopeKey);
    refreshAgents();
  };

  return {
    activeKind: resourceBrowser.activeKind,
    checkedIds: resourceBrowser.checkedIds,
    clearChecked: resourceBrowser.clearChecked,
    discoveredAgents,
    discoveryState,
    filteredAgents: managedVisibleAgents,
    filteredResources: resourceBrowser.filteredResources,
    managedAgents,
    managedAgentsForView: resolvedAgents,
    onCreateAgentSuccess: syncCreatedAgent,
    onDeleteAgentSuccess: syncDeletedAgent,
    onDeleteLocalSkill: deleteSelectedLocalSkill,
    onImportAgentsSuccess: syncImportedAgents,
    onOpenSkillEntryFile: openSelectedSkillEntryFile,
    onOpenSkillFolder: openSelectedSkillFolder,
    onRemoveAgentSuccess: syncRemovedAgent,
    onPreviewCopy: previewCopy,
    onExecuteCopy: executeCopy,
    search,
    isMarketplaceLoading: resourceBrowser.isMarketplaceLoading,
    isMarketplaceLoadingMore: resourceBrowser.isMarketplaceLoadingMore,
    hasMoreMarketplaceItems: resourceBrowser.hasMoreMarketplaceItems,
    loadMoreMarketplaceItems: resourceBrowser.loadMoreMarketplaceItems,
    marketplaceTotalSkills: resourceBrowser.marketplaceTotalSkills,
    isMarketplaceDetailLoading: selectedMarketplaceSkillDetailQuery.isFetching,
    isLocalMarketplaceDetailLoading: selectedLocalMarketplaceUpdateQuery.isFetching,
    marketplaceError: resourceBrowser.marketplaceError,
    refreshAgents,
    refreshSkills: (skillId?: string) => refreshSkills(scopeKey, skillId ?? selectedSkillId),
    selectAllAgents,
    selectKind: resourceBrowser.selectKind,
    selectResource: resourceBrowser.selectResource,
    selectedAgent,
    selectedAgentId: currentSelectedAgentId || selectedAgentId,
    selectedScope,
    selectedResource,
    selectedResourceId: resourceBrowser.selectedResourceId,
    setSearch,
    setSelectedAgentId: selectAgent,
    toggleChecked: resourceBrowser.toggleChecked,
    toggleAllChecked: resourceBrowser.toggleAllChecked,
    updateMarketplaceInstallState: resourceBrowser.updateMarketplaceInstallState,
    workspaceMode,
    enterAddingMode,
    exitAddingMode,
  };
}
