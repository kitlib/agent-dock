import { useCallback, useMemo, useState } from "react";
import {
  createManualAgent,
  deleteAgent,
  importAgents,
  removeManagedAgent,
  scanAgents,
} from "./api";
import type {
  AgentManagementCard,
  CreateAgentResult,
  DeleteAgentResult,
  ImportAgentsResult,
  ManualAgentDraft,
  RemoveAgentResult,
  ResolvedAgentView,
  ScannedAgentCandidate,
} from "./types";

const defaultManualDraft: ManualAgentDraft = {
  provider: "claude",
  name: "",
  rootPath: "",
  configPath: "",
};

type UseAgentImportParams = {
  managedAgentsForView?: ResolvedAgentView[];
  onCreateSuccess?: (result: CreateAgentResult) => void;
  onDeleteSuccess?: (result: DeleteAgentResult) => void;
  onImportSuccess?: (result: ImportAgentsResult) => void;
  onRemoveSuccess?: (result: RemoveAgentResult) => void;
};

function createManualManagementCard(agent: ResolvedAgentView): AgentManagementCard {
  const isManaged = agent.managed;

  return {
    id: agent.discoveryId || `candidate-${agent.id}`,
    fingerprint: agent.fingerprint,
    provider: agent.provider,
    displayName: agent.alias ?? agent.name,
    rootPath: agent.rootPath,
    configPath: agent.configPath,
    sourceScope: agent.sourceScope,
    workspaceName: undefined,
    resourceCounts: agent.resourceCounts,
    state: isManaged ? "imported" : "ready",
    reason: agent.summary,
    managedAgentId: agent.managedAgentId,
    managed: isManaged,
    detectedAt: agent.lastScannedAt,
    origin: "manual",
    deletable: true,
  };
}

export function useAgentImport({
  managedAgentsForView = [],
  onCreateSuccess,
  onDeleteSuccess,
  onImportSuccess,
  onRemoveSuccess,
}: UseAgentImportParams = {}) {
  const [isScanning, setIsScanning] = useState(false);
  const [scanError, setScanError] = useState<string | null>(null);
  const [scanResults, setScanResults] = useState<ScannedAgentCandidate[]>([]);
  const [manualDraft, setManualDraft] = useState<ManualAgentDraft>(defaultManualDraft);
  const [isImporting, setIsImporting] = useState(false);
  const [isCreatingManually, setIsCreatingManually] = useState(false);

  const managementCards = useMemo(() => {
    const scanCards: AgentManagementCard[] = scanResults.map((candidate) => ({
      ...candidate,
      origin: "scanned",
      deletable: false,
    }));
    const fingerprints = new Set(scanCards.map((candidate) => candidate.fingerprint));
    const manualOnlyCards = managedAgentsForView
      .filter((agent) => agent.sourceScope === "manual" && !fingerprints.has(agent.fingerprint))
      .map(createManualManagementCard);

    return [...scanCards, ...manualOnlyCards];
  }, [managedAgentsForView, scanResults]);

  const resetImportState = useCallback(() => {
    setScanError(null);
    setScanResults([]);
    setManualDraft(defaultManualDraft);
    setIsScanning(false);
    setIsImporting(false);
    setIsCreatingManually(false);
  }, []);

  const runScan = useCallback(async () => {
    setIsScanning(true);
    setScanError(null);

    try {
      const results = await scanAgents();
      setScanResults(results);
    } catch (error) {
      setScanError(error instanceof Error ? error.message : "Failed to scan local agents.");
      setScanResults([]);
    } finally {
      setIsScanning(false);
    }
  }, []);

  const enterImporting = useCallback(async () => {
    resetImportState();
    await runScan();
  }, [resetImportState, runScan]);

  const toggleCandidate = async (candidate: AgentManagementCard) => {
    if (candidate.state !== "ready" && candidate.state !== "imported") {
      return;
    }

    setIsImporting(true);

    try {
      if (candidate.state === "ready") {
        const result = await importAgents([candidate.id]);
        onImportSuccess?.(result);
        return;
      }

      if (candidate.managedAgentId) {
        const result = await removeManagedAgent(candidate.managedAgentId);
        onRemoveSuccess?.(result);
      }
    } finally {
      setIsImporting(false);
    }
  };

  const deleteCandidate = async (candidate: AgentManagementCard) => {
    if (!candidate.deletable || !candidate.managedAgentId) {
      return;
    }

    setIsImporting(true);

    try {
      const result = await deleteAgent(candidate.managedAgentId);
      onDeleteSuccess?.(result);
    } finally {
      setIsImporting(false);
    }
  };

  const updateManualDraft = <K extends keyof ManualAgentDraft>(
    field: K,
    value: ManualAgentDraft[K]
  ) => {
    setManualDraft((current) => ({ ...current, [field]: value }));
  };

  const resetManualDraft = useCallback(() => {
    setManualDraft(defaultManualDraft);
    setIsCreatingManually(false);
  }, []);

  const submitManualAdd = async () => {
    setIsCreatingManually(true);

    try {
      const result = await createManualAgent(manualDraft);
      onCreateSuccess?.(result);
      resetImportState();
    } finally {
      setIsCreatingManually(false);
    }
  };

  const canSubmitManual =
    manualDraft.name.trim().length > 0 &&
    manualDraft.rootPath.trim().length > 0 &&
    manualDraft.configPath.trim().length > 0;

  return {
    canSubmitManual,
    enterImporting,
    isCreatingManually,
    isImporting,
    isScanning,
    managementCards,
    manualDraft,
    resetImportState,
    resetManualDraft,
    runScan,
    scanError,
    scanResults,
    submitManualAdd,
    toggleCandidate,
    deleteCandidate,
    updateManualDraft,
  };
}
