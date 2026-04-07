import { useEffect, useRef, useState, type DragEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { WindowFrame } from "@/components/window-frame";
import { MainTitleBar } from "@/components/main-title-bar";
import { UpdaterDialog } from "@/components/updater-dialog";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "@/components/ui/resizable";
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
    clearChecked,
    updateMarketplaceInstallState,
    onOpenSkillFolder,
    managedAgentsForView,
    workspaceMode,
    enterAddingMode,
    onImportAgentsSuccess,
    onCreateAgentSuccess,
    onDeleteAgentSuccess,
    onRemoveAgentSuccess,
    refreshAgents,
  } = useAgentsPrototype();

  const [isRailCollapsed, setIsRailCollapsed] = useState(false);
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

  return (
    <WindowFrame titleBar={<MainTitleBar />} contentClassName="flex flex-1 overflow-hidden">
      <UpdaterDialog />
      <div className="h-full w-full overflow-hidden">
        <ResizablePanelGroup orientation="horizontal" className="h-full w-full">
          <ResizablePanel
            panelRef={leftPanelRef}
            defaultSize="18%"
            minSize={180}
            collapsedSize={leftPanelCollapsedSize}
            collapsible
            onResize={() => setIsRailCollapsed(leftPanelRef.current?.isCollapsed() ?? false)}
          >
            <AgentRail
              filteredAgents={filteredAgents}
              isCollapsed={isRailCollapsed}
              onAddAgent={enterAddingMode}
              onToggleCollapsed={toggleRailCollapsed}
              selectedAgentId={selectedAgentId}
              setSelectedAgentId={setSelectedAgentId}
              t={t}
            />
          </ResizablePanel>

          <ResizableHandle withHandle />

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
              <ResizablePanel defaultSize="30%" minSize={420}>
                <AgentResourcePanel
                  activeKind={activeKind}
                  checkedIds={checkedIds}
                  filteredResources={filteredResources}
                  onClearChecked={clearChecked}
                  onDragStart={handleDragStart}
                  onSearchChange={setSearch}
                  onSelectKind={selectKind}
                  onSelectResource={selectResource}
                  onToggleChecked={toggleChecked}
                  onUpdateMarketplaceInstallState={updateMarketplaceInstallState}
                  onOpenSkillFolder={onOpenSkillFolder}
                  search={search}
                  totalCount={filteredResources.length}
                  selectedResourceId={selectedResourceId}
                  t={t}
                />
              </ResizablePanel>

              <ResizableHandle withHandle />

              <ResizablePanel defaultSize="52%" minSize={200}>
                <AgentDetailPanel
                  onOpenSkillFolder={onOpenSkillFolder}
                  onRefreshAgents={refreshAgents}
                  selectedAgent={selectedAgent}
                  selectedResource={selectedResource}
                  onUpdateMarketplaceInstallState={updateMarketplaceInstallState}
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
