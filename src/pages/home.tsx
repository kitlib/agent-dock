import { useEffect, useRef, useState, type DragEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  Bot,
  Brain,
  ChevronLeft,
  ChevronRight,
  Download,
  MoreHorizontal,
  Plug,
  Search,
  Sparkles,
} from "lucide-react";
import { WindowFrame } from "@/components/window-frame";
import { MainTitleBar } from "@/components/main-title-bar";
import { UpdaterDialog } from "@/components/updater-dialog";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "@/components/ui/resizable";
import type { PanelImperativeHandle } from "react-resizable-panels";
import { registerShortcut } from "@/lib/shortcut";
import { toggleWindow } from "@/lib/window";
import { useAppTranslation } from "@/hooks/use-app-translation";
import { cn } from "@/lib/utils";
import { useAgentsPrototype } from "@/features/agents/hooks";
import type { AgentDiscoveryItem, ResourceKind } from "@/features/agents/types";

const SHORTCUT_KEY = "global-shortcut-show-main";

const kindIcons = {
  skill: Sparkles,
  mcp: Plug,
  subagent: Brain,
};

const installStateKey = {
  enabled: "prototype.actions.enabled",
  installed: "prototype.actions.installed",
  update: "prototype.actions.update",
  available: "prototype.actions.available",
} as const;

const agentStatusClassName = {
  online: "bg-emerald-500",
  idle: "bg-amber-500",
  busy: "bg-sky-500",
};

export default function HomePage() {
  const { t } = useAppTranslation();
  const {
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

  const renderDiscoveryMeta = (resource: AgentDiscoveryItem) => {
    return (
      <div className="text-muted-foreground mt-1 flex flex-wrap items-center gap-2 text-xs">
        <span className="bg-muted rounded px-2 py-1">
          {resource.origin === "local"
            ? t("prototype.badges.local")
            : t("prototype.badges.marketplace")}
        </span>
        <span className="bg-muted rounded px-2 py-1">
          {t(installStateKey[resource.installState])}
        </span>
        {resource.origin === "local" && resource.usageLabel !== undefined ? (
          <span>{t("prototype.detail.usedBy", { count: resource.usageLabel })}</span>
        ) : null}
        {resource.origin === "marketplace" ? (
          <>
            <span>{resource.author}</span>
            <span>
              <Download className="mr-1 inline h-3 w-3" />
              {resource.downloads}
            </span>
          </>
        ) : null}
      </div>
    );
  };

  const renderDetail = (resource: AgentDiscoveryItem | null) => {
    if (!resource) {
      return (
        <div className="text-muted-foreground flex h-full items-center justify-center text-sm">
          {t("prototype.emptySelection")}
        </div>
      );
    }

    if (resource.origin === "marketplace") {
      return (
        <div className="space-y-4">
          <section className="grid grid-cols-2 gap-3 text-sm">
            <div className="bg-background rounded-lg border p-3">
              <div className="text-muted-foreground text-xs">{t("prototype.detail.source")}</div>
              <div className="mt-1 font-medium">{resource.sourceLabel}</div>
            </div>
            <div className="bg-background rounded-lg border p-3">
              <div className="text-muted-foreground text-xs">{t("prototype.detail.version")}</div>
              <div className="mt-1 font-medium">{resource.version}</div>
            </div>
          </section>

          <section className="grid grid-cols-2 gap-3 text-sm">
            <div className="bg-background rounded-lg border p-3">
              <div className="text-muted-foreground text-xs">{t("prototype.detail.author")}</div>
              <div className="mt-1 font-medium">{resource.author}</div>
            </div>
            <div className="bg-background rounded-lg border p-3">
              <div className="text-muted-foreground text-xs">{t("prototype.detail.downloads")}</div>
              <div className="mt-1 font-medium">{resource.downloads}</div>
            </div>
          </section>

          <section className="space-y-2">
            <h3 className="text-sm font-semibold">{t("prototype.detail.highlights")}</h3>
            <ul className="space-y-2 text-sm">
              {resource.highlights.map((highlight) => (
                <li key={highlight} className="bg-muted/40 rounded-lg border px-3 py-2">
                  {highlight}
                </li>
              ))}
            </ul>
          </section>

          <div className="bg-background space-y-3 rounded-lg border p-4">
            <div className="flex items-center justify-between gap-3">
              <div>
                <div className="text-sm font-medium">
                  {t(installStateKey[resource.installState])}
                </div>
                <div className="text-muted-foreground mt-1 text-xs">{resource.sourceLabel}</div>
              </div>
              <Button onClick={() => updateMarketplaceInstallState(resource.id)}>
                {t(installStateKey[resource.installState])}
              </Button>
            </div>
          </div>
        </div>
      );
    }

    if (resource.kind === "skill") {
      return (
        <div className="space-y-4">
          <section className="space-y-2">
            <h3 className="text-sm font-semibold">{t("prototype.detail.preview")}</h3>
            <div className="bg-muted/40 rounded-lg border p-3 text-sm whitespace-pre-wrap">
              {resource.markdown}
            </div>
          </section>
          <section className="space-y-2">
            <h3 className="text-sm font-semibold">{t("prototype.detail.tags")}</h3>
            <div className="flex flex-wrap gap-2">
              {resource.tags.map((tag) => (
                <span
                  key={tag}
                  className="bg-muted text-muted-foreground rounded-md px-2 py-1 text-xs"
                >
                  {tag}
                </span>
              ))}
            </div>
          </section>
        </div>
      );
    }

    if (resource.kind === "mcp") {
      return (
        <div className="space-y-4">
          <section className="space-y-2">
            <h3 className="text-sm font-semibold">{t("prototype.detail.document")}</h3>
            <div className="bg-muted/40 rounded-lg border p-3 text-sm whitespace-pre-wrap">
              {resource.document}
            </div>
          </section>
          <section className="space-y-2">
            <h3 className="text-sm font-semibold">{t("prototype.detail.config")}</h3>
            <pre className="bg-muted/40 overflow-x-auto rounded-lg border p-3 text-xs">
              {resource.config}
            </pre>
          </section>
        </div>
      );
    }

    return (
      <div className="space-y-4">
        <section className="space-y-2">
          <h3 className="text-sm font-semibold">{t("prototype.detail.prompt")}</h3>
          <div className="bg-muted/40 rounded-lg border p-3 text-sm whitespace-pre-wrap">
            {resource.prompt}
          </div>
        </section>
        <section className="space-y-2">
          <h3 className="text-sm font-semibold">{t("prototype.detail.capabilities")}</h3>
          <ul className="space-y-2 text-sm">
            {resource.capabilities.map((capability) => (
              <li key={capability} className="bg-muted/40 rounded-lg border px-3 py-2">
                {capability}
              </li>
            ))}
          </ul>
        </section>
      </div>
    );
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
            minSize={220}
            collapsedSize={leftPanelCollapsedSize}
            collapsible
            onResize={() => setIsRailCollapsed(leftPanelRef.current?.isCollapsed() ?? false)}
          >
              <aside className="border-border flex h-full min-w-0 flex-col overflow-x-hidden border-r">
                <div className="flex items-center justify-between gap-2 border-b p-3">
                  {!isRailCollapsed && (
                    <span className="text-sm font-semibold">{t("prototype.agents.agents")}</span>
                  )}
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    onClick={toggleRailCollapsed}
                    title={
                      isRailCollapsed
                        ? t("prototype.actions.expand")
                        : t("prototype.actions.collapse")
                    }
                  >
                    {isRailCollapsed ? (
                      <ChevronRight className="h-4 w-4" />
                    ) : (
                      <ChevronLeft className="h-4 w-4" />
                    )}
                  </Button>
                </div>

                <div className="flex-1 overflow-auto p-2">
                  <div className="space-y-1">
                    {agentGroups.map((group) => (
                      <button
                        key={group.id}
                        onClick={() => setSelectedGroupId(group.id)}
                        className={cn(
                          "flex w-full items-center rounded-lg px-2 py-2 text-left text-sm transition-colors",
                          selectedGroupId === group.id
                            ? "bg-accent text-accent-foreground"
                            : "text-muted-foreground hover:bg-accent/50 hover:text-foreground"
                        )}
                        title={group.name}
                      >
                        <Bot className="h-4 w-4 shrink-0" />
                        {!isRailCollapsed && (
                          <>
                            <span className="ml-2 flex-1 truncate">{group.name}</span>
                            <span className="text-xs">{group.count}</span>
                          </>
                        )}
                      </button>
                    ))}
                  </div>

                  <div className="mt-3 border-t pt-3">
                    <div className="space-y-1">
                      {filteredAgents.map((agent) => (
                        <button
                          key={agent.id}
                          onClick={() => setSelectedAgentId(agent.id)}
                          className={cn(
                            "flex w-full items-center rounded-lg px-2 py-2 text-left text-sm transition-colors",
                            selectedAgentId === agent.id
                              ? "bg-primary/10 text-foreground"
                              : "text-muted-foreground hover:bg-accent/50 hover:text-foreground"
                          )}
                          title={agent.name}
                        >
                          <span
                            className={cn(
                              "mr-2 h-2.5 w-2.5 rounded-full",
                              agentStatusClassName[agent.status]
                            )}
                          />
                          {!isRailCollapsed ? (
                            <div className="min-w-0 flex-1">
                              <div className="truncate font-medium">{agent.name}</div>
                              <div className="text-muted-foreground truncate text-xs">
                                {agent.role}
                              </div>
                            </div>
                          ) : null}
                        </button>
                      ))}
                    </div>
                  </div>
                </div>
              </aside>
            </ResizablePanel>

            <ResizableHandle withHandle />

            <ResizablePanel defaultSize="50%" minSize={420}>
              <section className="flex h-full min-w-0 flex-col overflow-hidden">
                <div className="border-b p-3">
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div className="flex flex-wrap items-center gap-2">
                      {(["skill", "mcp", "subagent"] as ResourceKind[]).map((kind) => {
                        const Icon = kindIcons[kind];
                        const active = activeKind === kind;
                        return (
                          <Button
                            key={kind}
                            variant={active ? "default" : "outline"}
                            size="sm"
                            onClick={() => selectKind(kind)}
                          >
                            <Icon className="h-4 w-4" />
                            {t(`prototype.tabs.${kind}`)}
                          </Button>
                        );
                      })}
                    </div>
                    {selectedAgent && !isRailCollapsed && (
                      <div className="bg-muted text-muted-foreground hidden rounded-md px-2 py-1 text-xs lg:flex">
                        {selectedAgent.name}
                      </div>
                    )}
                  </div>
                  <div className="mt-3 flex flex-wrap items-center gap-2">
                    <div className="relative min-w-[240px] flex-1">
                      <Search className="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2" />
                      <Input
                        value={search}
                        onChange={(event) => setSearch(event.target.value)}
                        className="pl-9"
                        placeholder={t("prototype.actions.searchPlaceholder")}
                      />
                    </div>
                    <Button variant="outline" size="sm">
                      {t("prototype.agents.discovery")}
                    </Button>
                  </div>
                </div>

                {checkedIds.length > 0 && (
                  <div className="bg-muted/50 flex items-center justify-between border-b px-3 py-2 text-sm">
                    <span>
                      {t("prototype.actions.batchSelected", { count: checkedIds.length })}
                    </span>
                    <div className="flex items-center gap-2">
                      <Button variant="outline" size="xs">
                        {t("prototype.actions.disable")}
                      </Button>
                      <Button variant="outline" size="xs" onClick={clearChecked}>
                        {t("prototype.actions.clear")}
                      </Button>
                    </div>
                  </div>
                )}

                <div className="flex-1 overflow-auto p-2">
                  {filteredResources.length === 0 ? (
                    <div className="text-muted-foreground flex h-full items-center justify-center text-sm">
                      {search ? t("prototype.noResults") : t("prototype.emptyList")}
                    </div>
                  ) : (
                    <div className="space-y-1">
                      {filteredResources.map((resource) => {
                        const Icon = kindIcons[resource.kind];
                        const active = resource.id === selectedResourceId;
                        return (
                          <div
                            key={resource.id}
                            draggable={resource.origin === "local"}
                            onDragStart={(event) => handleDragStart(event, resource.id)}
                            onClick={() => selectResource(resource)}
                            className={cn(
                              "group border-border/70 hover:bg-accent/40 rounded-lg border px-3 py-2 transition-colors",
                              active ? "bg-accent border-primary/40" : "bg-background"
                            )}
                          >
                            <div className="flex items-start gap-3">
                              {resource.origin === "local" ? (
                                <Checkbox
                                  checked={checkedIds.includes(resource.id)}
                                  onCheckedChange={() => toggleChecked(resource.id)}
                                  onClick={(event) => event.stopPropagation()}
                                  className="mt-1"
                                  aria-label={resource.name}
                                />
                              ) : null}
                              <Icon className="text-muted-foreground mt-0.5 h-4 w-4 shrink-0" />
                              <div className="min-w-0 flex-1">
                                <div className="flex items-center justify-between gap-3">
                                  <div className="truncate text-sm font-medium">
                                    {resource.name}
                                  </div>
                                  <div className="text-muted-foreground text-xs">
                                    {resource.updatedAt}
                                  </div>
                                </div>
                                <div className="text-muted-foreground mt-1 line-clamp-1 text-xs">
                                  {resource.summary}
                                </div>
                                {renderDiscoveryMeta(resource)}
                              </div>
                              <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                  <Button
                                    variant="ghost"
                                    size="icon-xs"
                                    onClick={(event) => event.stopPropagation()}
                                  >
                                    <MoreHorizontal className="h-4 w-4" />
                                  </Button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent align="end">
                                  {resource.origin === "local" ? (
                                    <>
                                      <DropdownMenuItem>
                                        {t("prototype.actions.enable")}
                                      </DropdownMenuItem>
                                      <DropdownMenuItem>
                                        {t("prototype.actions.disable")}
                                      </DropdownMenuItem>
                                    </>
                                  ) : (
                                    <DropdownMenuItem
                                      onClick={() => updateMarketplaceInstallState(resource.id)}
                                    >
                                      {t(installStateKey[resource.installState])}
                                    </DropdownMenuItem>
                                  )}
                                </DropdownMenuContent>
                              </DropdownMenu>
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              </section>
            </ResizablePanel>

            <ResizableHandle withHandle />

            <ResizablePanel defaultSize="32%" minSize={200}>
              <aside className="bg-muted/20 flex h-full min-w-0 flex-col overflow-hidden border-l">
                <div className="border-b p-4">
                  <div className="text-lg font-semibold break-words">
                    {selectedResource?.name ?? t("prototype.detail.title")}
                  </div>
                  <div className="text-muted-foreground mt-1 text-sm">
                    {selectedResource?.summary ??
                      selectedAgent?.summary ??
                      t("prototype.emptySelection")}
                  </div>
                  {selectedResource ? (
                    <div className="text-muted-foreground mt-3 flex flex-wrap items-center gap-2 text-xs">
                      <span className="bg-muted rounded px-2 py-1">
                        {selectedResource.origin === "local"
                          ? t("prototype.badges.local")
                          : t("prototype.badges.marketplace")}
                      </span>
                      <span className="bg-muted rounded px-2 py-1">
                        {t(installStateKey[selectedResource.installState])}
                      </span>
                    </div>
                  ) : null}
                  {selectedAgent && (
                    <div className="text-muted-foreground mt-3 text-xs">
                      {selectedAgent.name} · {selectedAgent.role}
                    </div>
                  )}
                </div>
                <div className="flex-1 overflow-auto p-4">{renderDetail(selectedResource)}</div>
              </aside>
            </ResizablePanel>
        </ResizablePanelGroup>
      </div>
    </WindowFrame>
  );
}
