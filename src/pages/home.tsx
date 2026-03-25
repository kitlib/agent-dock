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
import { AgentRail } from "@/features/agents/agent-rail";
import { AgentImportPanel } from "@/features/agents/agent-import-panel";
import { AgentResourcePanel } from "@/features/agents/resource-panel";
import { AgentDetailPanel } from "@/features/agents/detail-panel";

const SHORTCUT_KEY = "global-shortcut-show-main";

export default function HomePage() {
  const { t } = useAppTranslation();
  const {
    conflicts,
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
    importAgent,
    managedAgentsForView,
    setAgentEnabled,
    workspaceMode,
    enterAddingMode,
    onImportAgentsSuccess,
    onCreateAgentSuccess,
    onDeleteAgentSuccess,
    onRemoveAgentSuccess,
  } = useAgentsPrototype();

  const [isRailCollapsed, setIsRailCollapsed] = useState(false);
  const leftPanelRef = useRef<PanelImperativeHandle | null>(null);
  const leftPanelCollapsedSize = 56;

  const toggleRailCollapsed = () => {
    const nextCollapsed = !isRailCollapsed;
    setIsRailCollapsed(nextCollapsed);

    if (nextCollapsed) {
      leftPanelRef.current?.collapse();
      return;
    }

    leftPanelRef.current?.expand();
  };

  const handleDragStart = (event: DragEvent<HTMLDivElement>, resourceId: string) => {
    event.dataTransfer.setData("text/plain", resourceId);
  };

  useEffect(() => {
    const unlistenShortcutChanged = listen<{ shortcut: string }>(
      "shortcut-changed",
      async (event) => {
        console.log("Shortcut changed event received:", event.payload.shortcut);
        const newShortcut = event.payload.shortcut;
        if (newShortcut) {
          await registerShortcut(newShortcut, async () => {
            await toggleWindow("main");
          });
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

    const initShortcut = async () => {
      const savedShortcut = localStorage.getItem(SHORTCUT_KEY);
      if (savedShortcut) {
        console.log("Registering saved shortcut:", savedShortcut);
        await registerShortcut(savedShortcut, async () => {
          await toggleWindow("main");
        });
      }
    };
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
              <ResizablePanel defaultSize="50%" minSize={420}>
                <AgentResourcePanel
                  activeKind={activeKind}
                  checkedIds={checkedIds}
                  filteredResources={filteredResources}
                  isRailCollapsed={isRailCollapsed}
                  onClearChecked={clearChecked}
                  onDragStart={handleDragStart}
                  onSearchChange={setSearch}
                  onSelectKind={selectKind}
                  onSelectResource={selectResource}
                  onToggleChecked={toggleChecked}
                  onUpdateMarketplaceInstallState={updateMarketplaceInstallState}
                  search={search}
                  selectedAgentName={selectedAgent?.alias ?? selectedAgent?.name}
                  selectedResourceId={selectedResourceId}
                  t={t}
                />
              </ResizablePanel>

              <ResizableHandle withHandle />

              <ResizablePanel defaultSize="32%" minSize={200}>
                <AgentDetailPanel
                  conflicts={conflicts}
                  onImportAgent={importAgent}
                  onSetAgentEnabled={setAgentEnabled}
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
