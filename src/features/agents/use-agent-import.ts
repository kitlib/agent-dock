import { useCallback, useEffect, useMemo, useState } from "react";
import {
  createManualAgent,
  deleteAgent,
  importAgents,
  removeManagedAgent,
  scanAgents,
} from "./api";
import { agentMeta } from "./agent-meta";
import type {
  AgentManagementCard,
  CreateAgentResult,
  DeleteAgentResult,
  ImportAgentsResult,
  ManualAgentDraft,
  RemoveAgentResult,
  ResolvedAgentView,
  ScanTarget,
} from "./types";

function inferScanState(managed: boolean, status: ResolvedAgentView["status"]) {
  if (managed) {
    return "imported" as const;
  }

  if (status === "unreadable" || status === "invalid") {
    return "unreadable" as const;
  }

  return "ready" as const;
}

const defaultManualDraft: ManualAgentDraft = {
  provider: "claude",
  name: agentMeta.claude.name,
  rootPath: agentMeta.claude.directory,
};

const scanTargets: ScanTarget[] = Object.values(agentMeta).map((meta) => ({
  agent: meta.id,
  name: meta.name,
  rootPath: meta.directory.replace(/\/$/, ""),
}));

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
    resourceCounts: agent.resourceCounts,
    state: inferScanState(isManaged, agent.status),
    reason: agent.summary,
    managedAgentId: agent.managedAgentId,
    managed: isManaged,
    detectedAt: agent.lastScannedAt,
    origin: "manual",
    deletable: true,
  };
}

function isManualOnlyAgent(agent: ResolvedAgentView, scannedFingerprints: Set<string>) {
  return !scannedFingerprints.has(agent.fingerprint) && !!agent.managedAgentId;
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
  const [scanResults, setScanResults] = useState<AgentManagementCard[]>([]);
  const [manualDraft, setManualDraft] = useState<ManualAgentDraft>(defaultManualDraft);
  const [isImporting, setIsImporting] = useState(false);
  const [isCreatingManually, setIsCreatingManually] = useState(false);

  const managementCards = useMemo(() => {
    const fingerprints = new Set(scanResults.map((candidate) => candidate.fingerprint));
    const manualOnlyCards = managedAgentsForView
      .filter((agent) => isManualOnlyAgent(agent, fingerprints))
      .map(createManualManagementCard);

    return [...scanResults, ...manualOnlyCards];
  }, [managedAgentsForView, scanResults]);

  useEffect(() => {
    setScanResults((current) =>
      current.map((candidate) => {
        const managedAgent = managedAgentsForView.find(
          (agent) => agent.fingerprint === candidate.fingerprint
        );

        if (!managedAgent) {
          if (!candidate.managed && candidate.state !== "imported") {
            return candidate;
          }

          return {
            ...candidate,
            managed: false,
            managedAgentId: undefined,
            state: inferScanState(false, "discovered"),
          };
        }

        const nextManaged = managedAgent.managed;
        const nextState = inferScanState(nextManaged, managedAgent.status);

        if (
          candidate.managed === nextManaged &&
          candidate.managedAgentId === managedAgent.managedAgentId &&
          candidate.state === nextState
        ) {
          return candidate;
        }

        return {
          ...candidate,
          managed: nextManaged,
          managedAgentId: managedAgent.managedAgentId,
          state: nextState,
        };
      })
    );
  }, [managedAgentsForView]);

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
      const results = await scanAgents(scanTargets);
      setScanResults(
        results.map((candidate) => ({
          ...candidate,
          origin: "scanned",
          deletable: false,
        }))
      );
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

  const runImportAction = useCallback(async (action: () => Promise<void>) => {
    setIsImporting(true);

    try {
      await action();
    } finally {
      setIsImporting(false);
    }
  }, []);

  const toggleCandidate = useCallback(
    async (candidate: AgentManagementCard) => {
      const { managedAgentId, state } = candidate;

      if (state !== "ready" && state !== "imported") {
        return;
      }

      await runImportAction(async () => {
        if (state === "ready") {
          const result = await importAgents([candidate.id], scanTargets);
          onImportSuccess?.(result);
        } else if (managedAgentId) {
          const result = await removeManagedAgent(managedAgentId, scanTargets);
          onRemoveSuccess?.(result);
        }
      });
    },
    [onImportSuccess, onRemoveSuccess, runImportAction]
  );

  const deleteCandidate = useCallback(
    async (candidate: AgentManagementCard) => {
      const { managedAgentId } = candidate;

      if (!candidate.deletable || !managedAgentId) {
        return;
      }

      await runImportAction(async () => {
        const result = await deleteAgent(managedAgentId, scanTargets);
        onDeleteSuccess?.(result);
      });
    },
    [onDeleteSuccess, runImportAction]
  );

  const updateManualDraft = <K extends keyof ManualAgentDraft>(
    field: K,
    value: ManualAgentDraft[K]
  ) => {
    setManualDraft((current: ManualAgentDraft) => {
      if (field === "provider") {
        const provider = value as ManualAgentDraft["provider"];
        return {
          ...current,
          provider,
          name: agentMeta[provider].name,
          rootPath: agentMeta[provider].directory,
        };
      }

      return { ...current, [field]: value };
    });
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
      resetManualDraft();
    } finally {
      setIsCreatingManually(false);
    }
  };

  const canSubmitManual =
    manualDraft.name.trim().length > 0 &&
    manualDraft.rootPath.trim().length > 0;

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
