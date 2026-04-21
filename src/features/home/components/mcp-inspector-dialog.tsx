import { useEffect, useState, useRef } from "react";
import { Button } from "@/components/ui/button";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { ExternalLink, Loader2 } from "lucide-react";
import type { McpResource } from "@/features/agents/types";
import { getLocalMcpEditData, stopMcpInspector } from "@/features/agents/api";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";

type McpInspectorDialogProps = {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    resource: McpResource | null;
    t: (key: string, options?: Record<string, unknown>) => string;
};

export function McpInspectorDialog({
    open,
    onOpenChange,
    resource,
    t,
}: McpInspectorDialogProps) {
    const [isLoading, setIsLoading] = useState(false);
    const [isRunning, setIsRunning] = useState(false);
    const [errorMessage, setErrorMessage] = useState<string | null>(null);
    const [hasLaunched, setHasLaunched] = useState(false);
    const [inspectorUrl, setInspectorUrl] = useState<string | null>(null);
    const [logs, setLogs] = useState<Array<{type: 'stdout' | 'stderr', content: string}>>([]);
    const logsEndRef = useRef<HTMLDivElement>(null);
    const unlistenOutputRef = useRef<(() => void) | null>(null);
    const unlistenUrlRef = useRef<(() => void) | null>(null);
    const unlistenExitRef = useRef<(() => void) | null>(null);

    useEffect(() => {
        if (!open || !resource) {
            // Reset state when dialog closes
            setIsLoading(false);
            setIsRunning(false);
            setErrorMessage(null);
            setHasLaunched(false);
            setInspectorUrl(null);
            setLogs([]);
            return;
        }

        // Prevent infinite loop: only launch once per dialog open
        if (hasLaunched || isLoading || isRunning) return;

        // Auto start inspector when dialog opens
        const startInspector = async () => {
            setHasLaunched(true);
            setIsLoading(true);
            setErrorMessage(null);

            try {
                console.log("[MCP] Launch inspector request for server:", {
                    name: resource.name,
                    agentType: resource.agentType,
                    configPath: resource.configPath,
                    scope: resource.scope,
                    projectPath: resource.projectPath
                });

                // Get edit config first
                const editValue = await getLocalMcpEditData(
                    resource.agentType,
                    resource.configPath,
                    resource.name,
                    resource.scope ?? "user",
                    resource.projectPath
                );

                console.log("[MCP] Fetched edit config for inspection:", editValue);

                // Launch inspector - backend manages singleton, no pid returned
                await invoke<void>("launch_mcp_inspector", {
                    config: editValue,
                });

                console.log("[MCP] Inspector launched successfully");
                setLogs([]); // Clear history logs
                setInspectorUrl(null); // Reset URL

                // Listen to log output from backend (singleton, no pid check needed)
                unlistenOutputRef.current = await listen("mcp-inspector-output", (event: any) => {
                    setLogs(prev => [...prev, {
                        type: event.payload.type as 'stdout' | 'stderr',
                        content: event.payload.data
                    }]);
                    // Auto scroll to bottom
                    requestAnimationFrame(() => {
                        logsEndRef.current?.scrollIntoView({ behavior: "smooth" });
                    });
                });

                // Listen to inspector URL extracted from logs
                unlistenUrlRef.current = await listen("mcp-inspector-url", (event: any) => {
                    const url = event.payload.url as string;
                    console.log("[MCP] Got inspector access URL:", url);
                    setInspectorUrl(url);
                    // Service is ready, update status
                    setIsRunning(true);
                    setIsLoading(false);
                });

                // Listen to process exit event
                unlistenExitRef.current = await listen("mcp-inspector-exit", () => {
                    if (!isRunning) {
                        console.error("[MCP] Inspector process exited unexpectedly");
                        setErrorMessage(t("prototype.inspector.errors.launchFailed"));
                        setIsLoading(false);
                    }
                });
            } catch (error) {
                console.error("[MCP] Failed to launch inspector:", error);
                // Parse structured error code from backend
                let errorCode = "UNKNOWN";
                let rawMessage = "";
                try {
                    if (error instanceof Error && error.message) {
                        const errObj = JSON.parse(error.message);
                        if (errObj.code && typeof errObj.code === "string") {
                            errorCode = errObj.code;
                            rawMessage = errObj.message || "";
                        } else {
                            rawMessage = error.message;
                        }
                    } else if (typeof error === "string") {
                        rawMessage = error;
                    } else if (error && typeof error === "object" && "message" in error && typeof error.message === "string") {
                        rawMessage = error.message;
                    }
                } catch {
                    // Fallback to raw message if parsing fails
                    rawMessage = error instanceof Error ? error.message : String(error);
                }

                // Match localized message by error code
                switch(errorCode) {
                    case "NODE_NOT_INSTALLED":
                        setErrorMessage(`${t("prototype.inspector.errors.nodeRequired")}\n\n${t("prototype.inspector.errors.nodeRequiredHint")}`);
                        break;
                    case "LAUNCH_FAILED":
                        setErrorMessage(t("prototype.inspector.errors.launchFailed"));
                        break;
                    case "MISSING_COMMAND":
                        setErrorMessage(t("prototype.inspector.errors.missingCommand"));
                        break;
                    case "MISSING_URL":
                        setErrorMessage(t("prototype.inspector.errors.missingUrl"));
                        break;
                    default:
                        // Show raw message first for unknown errors
                        setErrorMessage(rawMessage || t("prototype.feedback.loadMcpInspectFailed"));
                }
            } finally {
                setIsLoading(false);
            }
        };

        startInspector();

        // Cleanup function to stop process when component unmounts or dialog closes
        return () => {
            // Remove event listeners
            if (unlistenOutputRef.current) {
                unlistenOutputRef.current();
                unlistenOutputRef.current = null;
            }
            if (unlistenUrlRef.current) {
                unlistenUrlRef.current();
                unlistenUrlRef.current = null;
            }
            if (unlistenExitRef.current) {
                unlistenExitRef.current();
                unlistenExitRef.current = null;
            }
            // 停止单例进程
            stopMcpInspector().catch(console.error);
        };
    }, [open, resource, t]); // 仅保留必要外部依赖，避免无限循环

    const handleOpenInspector = async () => {
        // Use official opener plugin to ensure correct permissions
        if (inspectorUrl) {
            await openUrl(inspectorUrl);
        }
    };

    const handleClose = async () => {
        try {
            setIsLoading(true);
            await stopMcpInspector(); // 无需参数，后端维护单例
            setIsRunning(false);
            setInspectorUrl(null);
            setLogs([]);
        } catch (error) {
            console.error("[MCP] Failed to stop inspector:", error);
        } finally {
            setIsLoading(false);
        }
        onOpenChange(false);
    };

    const getStatusText = () => {
        if (isLoading || (!isRunning && !errorMessage)) {
            // 正在加载中，等待服务就绪
            return t("prototype.inspector.status.starting");
        }
        if (isRunning) {
            return t("prototype.inspector.status.running");
        }
        if (errorMessage?.includes("not installed")) {
            return t("prototype.inspector.status.notInstalled");
        }
        return t("prototype.inspector.status.stopped");
    };

    return (
        <Dialog open={open} onOpenChange={handleClose}>
            <DialogContent className="max-w-lg">
                <DialogHeader>
                    <DialogTitle>{t("prototype.inspector.title")}</DialogTitle>
                    <DialogDescription>
                        {t("prototype.inspector.description")}
                    </DialogDescription>
                </DialogHeader>

                <div className="space-y-4 py-2">

                    {/* Status - only show if no installation error and no node not found error */}
                    {!errorMessage?.includes(t("prototype.inspector.status.notInstalled")) &&
                     !errorMessage?.includes(t("prototype.inspector.errors.nodeRequired")) &&
                     !errorMessage?.includes("program not found") &&
                     !errorMessage?.includes("Node.js") && (
                        <div className="rounded-lg border p-3">
                            <div className="text-sm font-medium mb-2">{t("prototype.inspector.status.title")}</div>
                            <div className="flex items-center gap-2 text-sm">
                                {isLoading && <Loader2 className="h-4 w-4 animate-spin" />}
                                {getStatusText()}
                            </div>
                        </div>
                    )}

                    {/* Error message */}
                    {errorMessage && (
                        <div className="border-destructive/30 bg-destructive/10 text-black dark:text-white rounded-md border px-3 py-2 text-sm whitespace-pre-wrap">
                            {errorMessage}
                        </div>
                    )}

                    {/* Terminal output log */}
                    {!errorMessage?.includes(t("prototype.inspector.status.notInstalled")) &&
                     !errorMessage?.includes(t("prototype.inspector.errors.nodeRequired")) && (
                        <div className="rounded-lg border p-3">
                            <div className="text-sm font-medium mb-2">{t("prototype.inspector.terminalOutput")}</div>
                            <div className="bg-black rounded-md p-3 h-48 overflow-y-auto overflow-x-hidden font-mono text-xs text-white">
                                {logs.length === 0 ? (
                                    <div className="text-gray-500 leading-relaxed break-all">{t("prototype.inspector.waitingForOutput")}</div>
                                ) : (
                                    logs.map((log, index) => (
                                        <div
                                            key={index}
                                            className={log.type === 'stderr' ? 'text-red-400 whitespace-pre-wrap wrap-break-words break-all leading-relaxed w-full' : 'text-gray-200 whitespace-pre-wrap break-words break-all leading-relaxed w-full'}
                                        >
                                            {log.content}
                                        </div>
                                    ))
                                )}
                                <div ref={logsEndRef} />
                            </div>
                        </div>
                    )}

                    {/* Access URL - only show if running and no installation error */}
                    {isRunning && !errorMessage?.includes(t("prototype.inspector.status.notInstalled")) && (
                        <div className="space-y-2">
                            <div className="text-sm font-medium">{t("prototype.inspector.accessUrl")}</div>
                            <div className="flex gap-2">
                                <Input
                                    value={inspectorUrl || ''}
                                    readOnly
                                    className="font-mono text-sm"
                                />
                                <Button
                                    variant="default"
                                    size="icon"
                                    onClick={handleOpenInspector}
                                    title={t("prototype.inspector.openButton")}
                                >
                                    <ExternalLink className="h-4 w-4" />
                                </Button>
                            </div>
                        </div>
                    )}
                </div>

                <DialogFooter className="gap-2">
                    <Button
                        variant="outline"
                        onClick={handleClose}
                        disabled={isLoading}
                        className="gap-2 flex-1 sm:flex-none"
                    >
                        {isLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : null}
                        {t("prototype.inspector.stopAndClose")}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
