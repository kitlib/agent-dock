import { useEffect, useMemo, useState } from "react";
import { marketplaceItems } from "@/features/marketplace/mock";
import {
  buildDiscoveryItems,
  createMarketplaceInstallStateMap,
  filterDiscoveryItems,
  sortDiscoveryItems,
} from "@/features/resources/core/discovery";
import { resourcesByKind } from "@/features/resources/core/resource-catalog";
import { useAgentDiscovery } from "@/features/agents/use-agent-discovery";
import { useAgentManagement } from "@/features/agents/use-agent-management";
import { getLocalSkillDetail, listLocalSkills, openSkillFolder } from "@/features/agents/api";
import { filterSkillsForAgent, toSkillScanTargets } from "@/features/home/skill-targets";
import type {
  AgentDiscoveryItem,
  CreateAgentResult,
  DeleteAgentResult,
  ImportAgentsResult,
  MarketplaceInstallStateLabel,
  RemoveAgentResult,
  ResourceKind,
  ResolvedAgentView,
  SkillResource,
} from "@/features/agents/types";

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

export function useHomeWorkspace() {
  const [workspaceMode, setWorkspaceMode] = useState<WorkspaceMode>("browse");
  const [search, setSearch] = useState("");
  const [selectedAgentId, setSelectedAgentId] = useState("");
  const [activeKind, setActiveKind] = useState<ResourceKind>("skill");
  const [selectedResourceId, setSelectedResourceId] = useState("");
  const [checkedIds, setCheckedIds] = useState<string[]>([]);
  const [localSkills, setLocalSkills] = useState<SkillResource[]>([]);
  const [skillDetails, setSkillDetails] = useState<Record<string, SkillResource>>({});
  const [marketplaceInstallStates, setMarketplaceInstallStates] = useState<
    Record<string, MarketplaceInstallStateLabel>
  >(() => createMarketplaceInstallStateMap(marketplaceItems));

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

  const clearResourceSelection = () => {
    setSelectedResourceId("");
    setCheckedIds([]);
  };

  const setMode = (mode: WorkspaceMode) => {
    setWorkspaceMode(mode);
    clearResourceSelection();
  };

  const resetWorkspaceSelection = () => {
    setMode("adding");
  };

  const syncManagedState = (resolvedAgents: ResolvedAgentView[]) => {
    setResolvedAgents(resolvedAgents);
    setManagedAgents(buildManagedAgents(resolvedAgents));
    resetWorkspaceSelection();
  };

  const syncImportedAgents = ({ resolvedAgents }: ImportAgentsResult) => {
    syncManagedState(resolvedAgents);
  };

  const syncCreatedAgent = ({ resolvedAgents }: CreateAgentResult) => {
    syncManagedState(resolvedAgents);
  };

  const syncRemovedAgent = ({ removedAgentId, resolvedAgents }: RemoveAgentResult) => {
    syncManagedState(resolvedAgents);
    if (removedAgentId && selectedAgentId === removedAgentId) {
      setSelectedAgentId(resolvedAgents.find((entry) => entry.managed && !entry.hidden)?.id ?? "");
    }
  };

  const syncDeletedAgent = ({ deletedAgentId, resolvedAgents }: DeleteAgentResult) => {
    syncManagedState(resolvedAgents);
    if (deletedAgentId && selectedAgentId === deletedAgentId) {
      setSelectedAgentId(resolvedAgents.find((entry) => entry.managed && !entry.hidden)?.id ?? "");
    }
  };

  const normalizedSearch = search.trim().toLowerCase();

  const managedVisibleAgents = useMemo(
    () => resolvedAgents.filter((agent) => agent.managed && !agent.hidden),
    [resolvedAgents]
  );

  const managedAgentsForRail = useMemo(() => {
    return managedVisibleAgents.filter((agent) => {
      return (
        normalizedSearch.length === 0 ||
        agent.name.toLowerCase().includes(normalizedSearch) ||
        agent.role.toLowerCase().includes(normalizedSearch) ||
        agent.summary.toLowerCase().includes(normalizedSearch) ||
        agent.rootPath.toLowerCase().includes(normalizedSearch)
      );
    });
  }, [managedVisibleAgents, normalizedSearch]);

  const selectedAgent =
    managedAgentsForRail.find((agent) => agent.id === selectedAgentId) ??
    managedVisibleAgents.find((agent) => agent.id === selectedAgentId) ??
    managedAgentsForRail[0] ??
    managedVisibleAgents[0] ??
    null;

  const skillScanTargets = useMemo(() => managedVisibleAgents.flatMap(toSkillScanTargets), [managedVisibleAgents]);

  const visibleLocalSkills = useMemo(
    () => filterSkillsForAgent(localSkills, selectedAgent?.id ?? null),
    [localSkills, selectedAgent?.id]
  );

  useEffect(() => {
    console.log("[skills] workspace selection snapshot", {
      selectedAgentId,
      resolvedAgentIds: resolvedAgents.map((agent) => agent.id),
      managedVisibleAgentIds: managedVisibleAgents.map((agent) => agent.id),
      railAgentIds: managedAgentsForRail.map((agent) => agent.id),
      effectiveSelectedAgentId: selectedAgent?.id ?? null,
    });
  }, [
    managedAgentsForRail,
    managedVisibleAgents,
    resolvedAgents,
    selectedAgent?.id,
    selectedAgentId,
  ]);

  useEffect(() => {
    console.log("[skills] visible skills snapshot", {
      selectedAgentId: selectedAgent?.id ?? null,
      localSkillOwners: localSkills.map((skill) => ({
        id: skill.id,
        ownerAgentId: skill.ownerAgentId ?? null,
      })),
      visibleSkillIds: visibleLocalSkills.map((skill) => skill.id),
    });
  }, [localSkills, selectedAgent?.id, visibleLocalSkills]);

  useEffect(() => {
    if (skillScanTargets.length === 0) {
      setLocalSkills([]);
      setSkillDetails({});
      return;
    }

    let cancelled = false;

    void listLocalSkills(skillScanTargets)
      .then((skills) => {
        if (cancelled) {
          return;
        }
        setLocalSkills(skills.map((skill) => ({ ...skill, markdown: skill.markdown ?? "" })));
      })
      .catch(() => {
        if (!cancelled) {
          setLocalSkills([]);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [skillScanTargets]);

  const localResources = useMemo(
    () => ({
      ...resourcesByKind,
      skill: visibleLocalSkills,
    }),
    [visibleLocalSkills]
  );

  const discoveryItems = useMemo(() => {
    return buildDiscoveryItems(
      activeKind,
      localResources,
      marketplaceItems,
      marketplaceInstallStates,
      selectedAgent?.id ?? null,
      selectedAgent?.managed ?? false
    );
  }, [activeKind, localResources, marketplaceInstallStates, selectedAgent]);

  const filteredResources = useMemo(() => {
    const includeMarketplaceWhenEmpty = activeKind !== "skill" || normalizedSearch.length > 0;

    return sortDiscoveryItems(
      filterDiscoveryItems(discoveryItems, normalizedSearch, { includeMarketplaceWhenEmpty }),
      normalizedSearch
    );
  }, [activeKind, discoveryItems, normalizedSearch]);

  useEffect(() => {
    if (activeKind !== "skill") {
      return;
    }

    const detailTarget = filteredResources.find(
      (resource) => resource.id === selectedResourceId
    ) ?? filteredResources[0];

    if (
      !detailTarget ||
      detailTarget.kind !== "skill" ||
      detailTarget.origin !== "local" ||
      skillDetails[detailTarget.id]
    ) {
      return;
    }

    let cancelled = false;
    void getLocalSkillDetail(skillScanTargets, detailTarget.id)
      .then((detail) => {
        if (!cancelled) {
          setSkillDetails((current) => ({ ...current, [detail.id]: detail }));
        }
      })
      .catch(() => undefined);

    return () => {
      cancelled = true;
    };
  }, [activeKind, filteredResources, selectedResourceId, skillDetails, skillScanTargets]);

  const selectedResourceBase =
    filteredResources.find((resource) => resource.id === selectedResourceId) ??
    filteredResources[0] ??
    null;

  const selectedResource =
    selectedResourceBase?.kind === "skill" && selectedResourceBase.origin === "local"
      ? ({
          ...selectedResourceBase,
          ...skillDetails[selectedResourceBase.id],
        } as AgentDiscoveryItem)
      : selectedResourceBase;

  const toggleChecked = (id: string) => {
    const item = filteredResources.find((resource) => resource.id === id);
    if (!item || item.origin !== "local") return;

    setCheckedIds((current) =>
      current.includes(id) ? current.filter((entry) => entry !== id) : [...current, id]
    );
  };

  const clearChecked = () => setCheckedIds([]);

  const selectKind = (kind: ResourceKind) => {
    setActiveKind(kind);
    clearResourceSelection();
  };

  const selectResource = (resource: AgentDiscoveryItem | null) => {
    setSelectedResourceId(resource?.id ?? "");
  };

  const updateMarketplaceInstallState = (id: string) => {
    setMarketplaceInstallStates((current) => {
      const nextState =
        current[id] === "update" || current[id] === "available" ? "installed" : current[id];
      return { ...current, [id]: nextState };
    });
  };

  const openSelectedSkillFolder = (skillPath: string) => {
    void openSkillFolder(skillPath).catch(() => undefined);
  };

  const refreshSkills = () => {
    setSkillDetails({});
    if (skillScanTargets.length === 0) {
      setLocalSkills([]);
      return;
    }

    void listLocalSkills(skillScanTargets)
      .then((skills) => {
        setLocalSkills(skills.map((skill) => ({ ...skill, markdown: skill.markdown ?? "" })));
      })
      .catch(() => {
        setLocalSkills([]);
      });
  };

  const selectAgent = (id: string) => {
    console.log("[skills] user selected agent", { nextSelectedAgentId: id });
    setSelectedAgentId(id);
    setMode("browse");
  };

  const enterAddingMode = () => {
    setMode("adding");
  };

  const exitAddingMode = () => {
    setMode("browse");
  };

  return {
    activeKind,
    checkedIds,
    clearChecked,
    discoveredAgents,
    discoveryState,
    filteredAgents: managedAgentsForRail,
    filteredResources,
    managedAgents,
    managedAgentsForView: resolvedAgents,
    onCreateAgentSuccess: syncCreatedAgent,
    onDeleteAgentSuccess: syncDeletedAgent,
    onImportAgentsSuccess: syncImportedAgents,
    onOpenSkillFolder: openSelectedSkillFolder,
    onRemoveAgentSuccess: syncRemovedAgent,
    search,
    refreshAgents,
    refreshSkills,
    selectKind,
    selectResource,
    selectedAgent,
    selectedAgentId: selectedAgent?.id ?? selectedAgentId,
    selectedResource,
    selectedResourceId: selectedResource?.id ?? selectedResourceId,
    setSearch,
    setSelectedAgentId: selectAgent,
    toggleChecked,
    updateMarketplaceInstallState,
    workspaceMode,
    enterAddingMode,
    exitAddingMode,
  };
}
