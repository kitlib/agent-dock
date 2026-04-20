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
import { openUrl } from "@tauri-apps/plugin-opener";

type McpInspectorDialogProps = {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    resource: McpResource | null;
    t: (key: string, options?: Record<string, unknown>) => string;
};

const INSPECTOR_URL = "http://localhost:54397";

export function McpInspectorDialog({
    open,
    onOpenChange,
    resource,
    t,
}: McpInspectorDialogProps) {
    const [isLoading, setIsLoading] = useState(false);
    const [isRunning, setIsRunning] = useState(false);
    const [pid, setPid] = useState<number | null>(null);
    const [errorMessage, setErrorMessage] = useState<string | null>(null);
    const [hasLaunched, setHasLaunched] = useState(false);
    const checkTimerRef = useRef<NodeJS.Timeout | null>(null);

    useEffect(() => {
        if (!open || !resource) {
            // Reset state when dialog closes
            setIsLoading(false);
            setIsRunning(false);
            setPid(null);
            setErrorMessage(null);
            setHasLaunched(false);
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

                // Launch inspector
                const newPid = await invoke<number>("launch_mcp_inspector", {
                    config: editValue,
                    serverName: resource.name,
                });

                console.log("[MCP] Inspector launched successfully, PID:", newPid);
                setPid(newPid);

                // 轮询检测服务是否真正就绪，就绪后才显示访问地址
                let retries = 0;
                const maxRetries = 10; // 最多等待5秒
                const checkServiceReady = async () => {
                    try {
                        // 检测端口是否可访问，忽略响应内容
                        await fetch(INSPECTOR_URL, { method: "HEAD", mode: "no-cors" });
                        // 服务就绪，显示访问地址
                        setIsRunning(true);
                    } catch (error) {
                        retries++;
                        if (retries < maxRetries) {
                            // 500ms后重试
                            checkTimerRef.current = setTimeout(checkServiceReady, 500);
                        } else {
                            // 超时，启动失败
                            console.error("[MCP] Inspector service failed to start within 5 seconds");
                            setErrorMessage(t("prototype.inspector.errors.launchFailed"));
                            // 清理残留进程
                            stopMcpInspector(newPid).catch(console.error);
                        }
                    }
                };
                // 开始检测服务状态
                checkServiceReady();
            } catch (error) {
                console.error("[MCP] Failed to launch inspector:", error);
                // Show user-friendly error messages instead of raw technical errors
                let errorMsg = "";

                if (error instanceof Error && error.message) {
                    errorMsg = error.message;
                } else if (typeof error === "string" && error) {
                    errorMsg = error;
                } else if (error && typeof error === "object" && "message" in error && typeof error.message === "string") {
                    errorMsg = error.message;
                }

                // Map known error types to localized messages
                if (
                    errorMsg.includes("npx") ||
                    errorMsg.includes("Node.js") ||
                    errorMsg.includes("node") ||
                    errorMsg.includes("program not found") ||
                    errorMsg.includes("未安装") ||
                    errorMsg.includes("安装")
                ) {
                    setErrorMessage(
                        t("prototype.inspector.errors.nodeRequired") +
                        "\n\n" +
                        "请从 https://nodejs.org/ 安装Node.js后重试"
                    );
                } else if (
                    errorMsg.includes("Failed to launch") ||
                    errorMsg.includes("启动失败") ||
                    errorMsg.includes("launch")
                ) {
                    setErrorMessage(t("prototype.inspector.errors.launchFailed"));
                } else if (
                    errorMsg.includes("command") && errorMsg.includes("required") ||
                    errorMsg.includes("命令") && errorMsg.includes("缺失") ||
                    errorMsg.includes("command")
                ) {
                    setErrorMessage(t("prototype.inspector.errors.missingCommand"));
                } else if (
                    errorMsg.includes("url") && errorMsg.includes("required") ||
                    errorMsg.includes("地址") && errorMsg.includes("缺失") ||
                    errorMsg.includes("url") || errorMsg.includes("URL")
                ) {
                    setErrorMessage(t("prototype.inspector.errors.missingUrl"));
                } else if (errorMsg) {
                    // If we have an error message but didn't match any known type, show it directly
                    setErrorMessage(errorMsg);
                } else {
                    // Fallback to generic error
                    setErrorMessage(t("prototype.feedback.loadMcpInspectFailed"));
                }
            } finally {
                setIsLoading(false);
            }
        };

        startInspector();

        // Cleanup function to stop process when component unmounts or dialog closes
        return () => {
            // 清除可能存在的检测定时器
            if (checkTimerRef.current) {
                clearTimeout(checkTimerRef.current);
            }
            if (pid && isRunning) {
                stopMcpInspector(pid).catch(console.error);
            }
        };
    }, [open, resource, t, hasLaunched, isLoading, isRunning, pid]);

    const handleOpenInspector = async () => {
        // 和关于页面保持一致，用官方opener插件打开，保证权限正常
        await openUrl(INSPECTOR_URL);
    };

    const handleClose = async () => {
        if (pid && isRunning) {
            try {
                setIsLoading(true);
                await stopMcpInspector(pid);
                setIsRunning(false);
                setPid(null);
            } catch (error) {
                console.error("[MCP] Failed to stop inspector:", error);
            } finally {
                setIsLoading(false);
            }
        }
        onOpenChange(false);
    };

    const getStatusText = () => {
        if (isLoading) {
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

                    {/* Access URL - only show if running and no installation error */}
                    {isRunning && !errorMessage?.includes(t("prototype.inspector.status.notInstalled")) && (
                        <div className="space-y-2">
                            <div className="text-sm font-medium">{t("prototype.inspector.accessUrl")}</div>
                            <div className="flex gap-2">
                                <Input
                                    value={INSPECTOR_URL}
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
