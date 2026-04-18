import { useEffect, useRef, useState, type DragEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { WindowFrame } from "@/components/window-frame";
import { MainTitleBar } from "@/components/main-title-bar";
import { UpdaterDialog } from "@/components/updater-dialog";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Toaster } from "@/components/ui/sonner";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "@/components/ui/resizable";
import { setLocalSkillEnabled } from "@/features/agents/api";
import type { AgentDiscoveryItem, LocalDiscoveryItem } from "@/features/agents/types";
import { supportsAgentMcp } from "@/features/agents/agent-meta";
import { getLocalSkillToggleTarget } from "@/features/home/local-skill-toggle";
import {
  installSkillsshMarketplaceItem,
  previewSkillsshMarketplaceInstall,
} from "@/features/marketplace/api";
import { CopySkillDialog } from "@/features/home/components/copy-skill-dialog";
import { ImportMcpDialog } from "@/features/home/components/import-mcp-dialog";
import { MarketplaceInstallAgentDialog } from "@/features/home/components/marketplace-install-agent-dialog";
import type {
  MarketplaceDiscoveryItem,
  ResolvedAgentView,
  LocalSkillCopySource,
  LocalSkillCopyTargetAgent,
  LocalSkillConflictResolution,
} from "@/features/agents/types";
import type {
  MarketplaceInstallMethod,
  MarketplaceInstallPreview,
} from "@/features/marketplace/types";
import type { PanelImperativeHandle } from "react-resizable-panels";
import { registerShortcut } from "@/lib/shortcut";
import { toggleWindow } from "@/lib/window";
import { useAppTranslation } from "@/hooks/use-app-translation";
import { useAgentsPrototype } from "@/features/agents/hooks";
import { AgentRail } from "@/features/agents/components/agent-rail";
import { AgentImportPanel } from "@/features/home/components/agent-import-panel";
import { AgentResourcePanel } from "@/features/home/components/resource-panel";
import { AgentDetailPanel } from "@/features/home/components/detail-panel";

const SHORTCUT_KEY = "global-shortcut-show-main";

type PendingMarketplaceInstallRequest = {
  resourceId: string;
  source: string;
  skillId: string;
  name: string;
  description: string;
  targetAgent: LocalSkillCopyTargetAgent;
  installMethod: MarketplaceInstallMethod;
};

async function registerMainWindowShortcut(shortcut: string): Promise<void> {
  await registerShortcut(shortcut, async () => {
    await toggleWindow("main");
  });
}

export default function HomePage() {
  const { t } = useAppTranslation();
  const {
    filteredAgents,
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
    toggleAllChecked,
    clearChecked,
    updateMarketplaceInstallState,
    onDeleteLocalSkill,
    onDeleteLocalMcp,
    onImportLocalMcpJson,
    onOpenSkillEntryFile,
    onOpenSkillFolder,
    onOpenMcpConfigFile,
    onOpenMcpConfigFolder,
    onPreviewCopy,
    onExecuteCopy,
    refreshSkills,
    managedAgentsForView,
    isMarketplaceLoading,
    isMarketplaceLoadingMore,
    hasMoreMarketplaceItems,
    loadMoreMarketplaceItems,
    marketplaceTotalSkills,
    isMarketplaceDetailLoading,
    isLocalMarketplaceDetailLoading,
    marketplaceError,
    refreshMcps,
    selectAllAgents,
    selectedScope,
    workspaceMode,
    enterAddingMode,
    onImportAgentsSuccess,
    onCreateAgentSuccess,
    onDeleteAgentSuccess,
    onRemoveAgentSuccess,
    refreshAgents,
  } = useAgentsPrototype();

  const [isRailCollapsed, setIsRailCollapsed] = useState(false);
  const [isCopyDialogOpen, setIsCopyDialogOpen] = useState(false);
  const [copySources, setCopySources] = useState<LocalSkillCopySource[]>([]);
  const [pendingMarketplaceInstallSelection, setPendingMarketplaceInstallSelection] =
    useState<MarketplaceDiscoveryItem | null>(null);
  const [isImportMcpDialogOpen, setIsImportMcpDialogOpen] = useState(false);
  const [pendingInstallRequest, setPendingInstallRequest] =
    useState<PendingMarketplaceInstallRequest | null>(null);
  const [pendingInstallPreview, setPendingInstallPreview] =
    useState<MarketplaceInstallPreview | null>(null);
  const [isInstallingMarketplaceItem, setIsInstallingMarketplaceItem] = useState(false);
  const leftPanelRef = useRef<PanelImperativeHandle | null>(null);
  const leftPanelCollapsedSize = 56;

  function toggleRailCollapsed(): void {
    const nextCollapsed = !isRailCollapsed;
    setIsRailCollapsed(nextCollapsed);

    if (nextCollapsed) {
      leftPanelRef.current?.collapse();
      return;
    }

    leftPanelRef.current?.expand();
  }

  function handleDragStart(event: DragEvent<HTMLDivElement>, resourceId: string): void {
    event.dataTransfer.setData("text/plain", resourceId);
  }

  async function handleSetLocalSkillEnabled(
    skillPath: string,
    entryFilePath: string,
    enabled: boolean,
    skillId?: string
  ): Promise<void> {
    await setLocalSkillEnabled(skillPath, entryFilePath, enabled);
    refreshSkills(skillId);
  }

  async function handleToggleCheckedSkills(): Promise<void> {
    const selectedSkillResources = filteredResources
      .filter((resource) => checkedIds.includes(resource.id))
      .map((resource) => getLocalSkillToggleTarget(resource))
      .filter((target): target is NonNullable<typeof target> => target != null);

    const hasEnabled = selectedSkillResources.some((target) => target.enabled);
    const newEnabled = !hasEnabled;

    await Promise.all(
      selectedSkillResources.map((resource) =>
        handleSetLocalSkillEnabled(
          resource.skillPath,
          resource.entryFilePath,
          newEnabled,
          resource.id
        )
      )
    );
  }

  async function handleDeleteLocalSkill(
    skillPath: string,
    entryFilePath: string,
    skillId?: string
  ): Promise<void> {
    try {
      await onDeleteLocalSkill(skillPath, entryFilePath);
      refreshSkills(skillId);
      toast.success(t("prototype.feedback.deleteSuccess"));
    } catch (error) {
      const message = error instanceof Error ? error.message : t("prototype.feedback.deleteFailed");
      toast.error(message);
      throw error;
    }
  }

  async function handleDeleteLocalMcp(
    agentType: string,
    configPath: string,
    serverName: string
  ): Promise<void> {
    try {
      await onDeleteLocalMcp(agentType, configPath, serverName);
      refreshMcps();
      toast.success(t("prototype.feedback.deleteMcpSuccess"));
    } catch (error) {
      const message =
        error instanceof Error ? error.message : t("prototype.feedback.deleteMcpFailed");
      toast.error(message);
      throw error;
    }
  }

  async function handleImportLocalMcp(
    jsonPayload: string,
    conflictStrategy: "overwrite" | "skip"
  ) {
    if (!selectedAgent) {
      throw new Error(t("prototype.feedback.importMcpSelectAgent"));
    }

    try {
      const result = await onImportLocalMcpJson(
        selectedAgent.agentType,
        selectedAgent.rootPath,
        jsonPayload,
        conflictStrategy
      );
      refreshMcps();
      toast.success(
        t("prototype.feedback.importMcpSuccess", {
          count: result.importedCount,
        })
      );
      if (result.skippedCount > 0) {
        toast.message(
          t("prototype.feedback.importMcpSkipped", {
            count: result.skippedCount,
          })
        );
      }
      return result;
    } catch (error) {
      const message =
        error instanceof Error ? error.message : t("prototype.feedback.importMcpFailed");
      toast.error(message);
      throw error;
    }
  }

  function openCopyDialog(sources: LocalSkillCopySource[]): void {
    setCopySources(sources);
    setIsCopyDialogOpen(true);
  }

  function openSingleCopyDialog(source: LocalSkillCopySource): void {
    openCopyDialog([source]);
  }

  async function handleCopySkills(
    sources: LocalSkillCopySource[],
    targetAgent: LocalSkillCopyTargetAgent,
    resolutions: LocalSkillConflictResolution[]
  ): Promise<void> {
    try {
      await onExecuteCopy(sources, targetAgent, resolutions);
      toast.success(t("prototype.feedback.copySuccess"));
    } catch (error) {
      const message = error instanceof Error ? error.message : t("prototype.feedback.copyFailed");
      toast.error(message);
      throw error;
    }
  }

  function toTargetAgent(agent: ResolvedAgentView): LocalSkillCopyTargetAgent {
    return {
      agentId: agent.id,
      agentType: agent.agentType,
      agentName: agent.name,
      rootPath: agent.rootPath,
    };
  }

  async function installMarketplaceItemToAgent(
    resource: MarketplaceDiscoveryItem,
    agent: ResolvedAgentView,
    installMethod: MarketplaceInstallMethod
  ): Promise<void> {
    const installRequest: PendingMarketplaceInstallRequest = {
      resourceId: resource.id,
      source: resource.sourceLabel,
      skillId: resource.skillId ?? "",
      name: resource.name,
      description: resource.description,
      targetAgent: toTargetAgent(agent),
      installMethod,
    };

    try {
      const preview = await previewSkillsshMarketplaceInstall(
        installRequest.source,
        installRequest.skillId,
        installRequest.name,
        installRequest.description,
        installRequest.targetAgent,
        installRequest.installMethod
      );
      if (preview.hasConflict) {
        setPendingInstallRequest(installRequest);
        setPendingInstallPreview(preview);
        return;
      }

      setIsInstallingMarketplaceItem(true);
      await installSkillsshMarketplaceItem(
        installRequest.source,
        installRequest.skillId,
        installRequest.name,
        installRequest.description,
        installRequest.targetAgent,
        installRequest.installMethod
      );
      updateMarketplaceInstallState(installRequest.resourceId);
      refreshSkills();
      toast.success(t("prototype.feedback.marketplaceInstallSuccess"));
    } catch (error) {
      const message =
        error instanceof Error ? error.message : t("prototype.feedback.marketplaceInstallFailed");
      toast.error(message);
      throw error;
    } finally {
      setIsInstallingMarketplaceItem(false);
    }
  }

  async function handleInstallMarketplaceItem(resource: AgentDiscoveryItem): Promise<void> {
    if (resource.origin !== "marketplace") {
      return;
    }

    if (resource.kind !== "skill" || !resource.skillId) {
      toast.error(t("prototype.feedback.marketplaceInstallUnsupported"));
      return;
    }

    if (resource.installState === "installed") {
      return;
    }

    setPendingMarketplaceInstallSelection(resource);
  }

  async function handleUpdateLocalMarketplaceSkill(
    resource: LocalDiscoveryItem & { kind: "skill"; origin: "local" }
  ): Promise<void> {
    const source = resource.marketplaceSource ?? "";
    const remoteId = resource.marketplaceRemoteId ?? "";
    if (!source || !remoteId) {
      toast.error(t("prototype.feedback.marketplaceInstallUnsupported"));
      return;
    }

    const targetOwner = managedAgentsForView.find((agent) => agent.id === resource.ownerAgentId);
    if (!targetOwner) {
      toast.error(t("prototype.feedback.marketplaceInstallSelectAgent"));
      return;
    }

    try {
      const installRequest: PendingMarketplaceInstallRequest = {
        resourceId: resource.id,
        source,
        skillId: remoteId,
        name: resource.name,
        description: resource.description,
        targetAgent: {
          agentId: targetOwner.id,
          agentType: targetOwner.agentType,
          agentName: targetOwner.name,
          rootPath: targetOwner.rootPath,
        },
        installMethod: "skillsh",
      };
      const preview = await previewSkillsshMarketplaceInstall(
        installRequest.source,
        installRequest.skillId,
        installRequest.name,
        installRequest.description,
        installRequest.targetAgent,
        installRequest.installMethod
      );
      if (preview.hasConflict) {
        setPendingInstallRequest(installRequest);
        setPendingInstallPreview(preview);
        return;
      }

      setIsInstallingMarketplaceItem(true);
      await installSkillsshMarketplaceItem(
        installRequest.source,
        installRequest.skillId,
        installRequest.name,
        installRequest.description,
        installRequest.targetAgent,
        installRequest.installMethod
      );
      refreshSkills(resource.id);
      toast.success(t("prototype.feedback.marketplaceInstallSuccess"));
    } catch (error) {
      const message =
        error instanceof Error ? error.message : t("prototype.feedback.marketplaceInstallFailed");
      toast.error(message);
      throw error;
    } finally {
      setIsInstallingMarketplaceItem(false);
    }
  }

  async function handleConfirmMarketplaceOverwrite(): Promise<void> {
    if (!pendingInstallRequest) {
      return;
    }

    try {
      setIsInstallingMarketplaceItem(true);
      await installSkillsshMarketplaceItem(
        pendingInstallRequest.source,
        pendingInstallRequest.skillId,
        pendingInstallRequest.name,
        pendingInstallRequest.description,
        pendingInstallRequest.targetAgent,
        pendingInstallRequest.installMethod,
        true
      );
      updateMarketplaceInstallState(pendingInstallRequest.resourceId);
      refreshSkills();
      setPendingInstallPreview(null);
      setPendingInstallRequest(null);
      toast.success(t("prototype.feedback.marketplaceInstallSuccess"));
    } catch (error) {
      const message =
        error instanceof Error ? error.message : t("prototype.feedback.marketplaceInstallFailed");
      toast.error(message);
    } finally {
      setIsInstallingMarketplaceItem(false);
    }
  }

  useEffect(() => {
    const unlistenShortcutChanged = listen<{ shortcut: string }>(
      "shortcut-changed",
      async (event) => {
        console.log("Shortcut changed event received:", event.payload.shortcut);
        const newShortcut = event.payload.shortcut;
        if (newShortcut) {
          await registerMainWindowShortcut(newShortcut);
        }
      }
    );

    const initTrayMenu = async () => {
      try {
        await invoke("update_tray_menu", {
          showText: t("tray.show"),
          quitText: t("tray.quit"),
        });
      } catch (error) {
        console.error("Failed to initialize tray menu:", error);
      }
    };
    initTrayMenu();

    async function initShortcut() {
      const savedShortcut = localStorage.getItem(SHORTCUT_KEY);
      if (savedShortcut) {
        console.log("Registering saved shortcut:", savedShortcut);
        await registerMainWindowShortcut(savedShortcut);
      }
    }
    initShortcut();

    return () => {
      unlistenShortcutChanged.then((fn) => fn());
    };
  }, [t]);

  const canImportMcp =
    activeKind === "mcp" &&
    selectedScope === "agent" &&
    selectedAgent != null &&
    supportsAgentMcp(selectedAgent.agentType);
  const selectedAgentMcpNames = filteredResources
    .filter((resource) => resource.origin === "local" && resource.kind === "mcp")
    .map((resource) => resource.name);

  return (
    <WindowFrame titleBar={<MainTitleBar />} contentClassName="flex flex-1 overflow-hidden">
      <Toaster />
      <UpdaterDialog />
      <ImportMcpDialog
        open={isImportMcpDialogOpen}
        onOpenChange={setIsImportMcpDialogOpen}
        targetAgent={canImportMcp ? selectedAgent : null}
        existingServerNames={selectedAgentMcpNames}
        onImport={handleImportLocalMcp}
        t={t}
      />
      <Dialog
        open={pendingInstallRequest != null && pendingInstallPreview?.hasConflict === true}
        onOpenChange={(open) => {
          if (!open) {
            setPendingInstallPreview(null);
            setPendingInstallRequest(null);
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("prototype.marketplace.installConflict.title")}</DialogTitle>
            <DialogDescription>
              {t("prototype.marketplace.installConflict.description", {
                name: pendingInstallRequest?.name ?? "",
                agentName: pendingInstallRequest?.targetAgent.agentName ?? "",
              })}
            </DialogDescription>
          </DialogHeader>
          {pendingInstallPreview?.existingPath ? (
            <div className="bg-muted rounded-md border px-3 py-2 text-xs break-all">
              {pendingInstallPreview.existingPath}
            </div>
          ) : null}
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setPendingInstallPreview(null);
                setPendingInstallRequest(null);
              }}
            >
              {t("prototype.actions.cancel")}
            </Button>
            <Button
              onClick={() => void handleConfirmMarketplaceOverwrite()}
              disabled={isInstallingMarketplaceItem}
            >
              {t("prototype.actions.overwrite")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <CopySkillDialog
        open={isCopyDialogOpen}
        onOpenChange={setIsCopyDialogOpen}
        sources={copySources}
        targetAgents={managedAgentsForView.filter((agent) => agent.managed)}
        onPreview={onPreviewCopy}
        onCopy={handleCopySkills}
        t={t}
      />
      <MarketplaceInstallAgentDialog
        open={pendingMarketplaceInstallSelection != null}
        onOpenChange={(open) => {
          if (!open) {
            setPendingMarketplaceInstallSelection(null);
          }
        }}
        initialSelectedAgentId={selectedAgent?.id ?? null}
        targetAgents={managedAgentsForView.filter((agent) => agent.managed)}
        onConfirm={async (agent, installMethod) => {
          if (!pendingMarketplaceInstallSelection) {
            return;
          }

          const resource = pendingMarketplaceInstallSelection;
          setPendingMarketplaceInstallSelection(null);
          await installMarketplaceItemToAgent(resource, agent, installMethod);
        }}
        t={t}
      />
      <div className="h-full w-full overflow-hidden">
        <ResizablePanelGroup orientation="horizontal" className="h-full w-full">
          <ResizablePanel
            panelRef={leftPanelRef}
            defaultSize="18%"
            minSize={180}
            maxSize={320}
            collapsedSize={leftPanelCollapsedSize}
            collapsible
            onResize={() => setIsRailCollapsed(leftPanelRef.current?.isCollapsed() ?? false)}
          >
            <AgentRail
              filteredAgents={filteredAgents}
              isCollapsed={isRailCollapsed}
              onAddAgent={enterAddingMode}
              onSelectAll={selectAllAgents}
              onSelectAgent={setSelectedAgentId}
              onToggleCollapsed={toggleRailCollapsed}
              selectedScope={selectedScope}
              selectedAgentId={selectedAgentId}
              t={t}
            />
          </ResizablePanel>

          <ResizableHandle />

          {workspaceMode === "adding" ? (
            <ResizablePanel defaultSize="82%" minSize={420}>
              <AgentImportPanel
                managedAgentsForView={managedAgentsForView}
                onCreateSuccess={onCreateAgentSuccess}
                onDeleteSuccess={onDeleteAgentSuccess}
                onImportSuccess={onImportAgentsSuccess}
                onRemoveSuccess={onRemoveAgentSuccess}
                t={t}
              />
            </ResizablePanel>
          ) : (
            <>
              <ResizablePanel defaultSize="30%" minSize={420} maxSize={640}>
                <AgentResourcePanel
                  activeKind={activeKind}
                  canImportMcp={canImportMcp}
                  checkedIds={checkedIds}
                  filteredResources={filteredResources}
                  isAllAgentsView={selectedScope === "all"}
                  onClearChecked={clearChecked}
                  onCopySkill={openSingleCopyDialog}
                  onCopySkills={openCopyDialog}
                  onDeleteLocalSkill={handleDeleteLocalSkill}
                  onDeleteLocalMcp={handleDeleteLocalMcp}
                  onImportMcp={() => setIsImportMcpDialogOpen(true)}
                  onToggleCheckedSkills={handleToggleCheckedSkills}
                  onDragStart={handleDragStart}
                  onOpenSkillEntryFile={onOpenSkillEntryFile}
                  onOpenSkillFolder={onOpenSkillFolder}
                  onOpenMcpConfigFile={onOpenMcpConfigFile}
                  onOpenMcpConfigFolder={onOpenMcpConfigFolder}
                  onSearchChange={setSearch}
                  onSelectKind={selectKind}
                  onSelectResource={selectResource}
                  onSetLocalSkillEnabled={handleSetLocalSkillEnabled}
                  onToggleChecked={toggleChecked}
                  onToggleAllChecked={toggleAllChecked}
                  onInstallMarketplaceItem={handleInstallMarketplaceItem}
                  isMarketplaceLoading={isMarketplaceLoading}
                  isMarketplaceLoadingMore={isMarketplaceLoadingMore}
                  hasMoreMarketplaceItems={hasMoreMarketplaceItems}
                  onLoadMoreMarketplaceItems={loadMoreMarketplaceItems}
                  marketplaceError={marketplaceError}
                  marketplaceTotalSkills={marketplaceTotalSkills}
                  search={search}
                  selectedResourceId={selectedResourceId}
                  t={t}
                />
              </ResizablePanel>

              <ResizableHandle />

              <ResizablePanel defaultSize="52%" minSize={320}>
                <AgentDetailPanel
                  allAgentsDescription={t("prototype.detail.allAgentsDescription")}
                  allAgentsSkillCount={
                    filteredResources.filter((resource) => resource.kind === "skill").length
                  }
                  allAgentsTitle={t("prototype.agents.all")}
                  isAllAgentsView={selectedScope === "all"}
                  onDeleteLocalSkill={handleDeleteLocalSkill}
                  onDeleteLocalMcp={handleDeleteLocalMcp}
                  onOpenSkillEntryFile={onOpenSkillEntryFile}
                  onOpenSkillFolder={onOpenSkillFolder}
                  onOpenMcpConfigFile={onOpenMcpConfigFile}
                  onOpenMcpConfigFolder={onOpenMcpConfigFolder}
                  onRefreshAgents={refreshAgents}
                  onSetLocalSkillEnabled={handleSetLocalSkillEnabled}
                  selectedAgent={selectedAgent}
                  selectedResource={selectedResource}
                  isMarketplaceDetailLoading={isMarketplaceDetailLoading}
                  isLocalMarketplaceDetailLoading={isLocalMarketplaceDetailLoading}
                  onInstallMarketplaceItem={handleInstallMarketplaceItem}
                  onUpdateLocalMarketplaceSkill={handleUpdateLocalMarketplaceSkill}
                  t={t}
                />
              </ResizablePanel>
            </>
          )}
        </ResizablePanelGroup>
      </div>
    </WindowFrame>
  );
}
