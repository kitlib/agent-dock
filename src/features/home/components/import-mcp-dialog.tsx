import { useEffect, useMemo, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import type {
  ImportLocalMcpResult,
  LocalMcpImportConflictStrategy,
  ResolvedAgentView,
} from "@/features/agents/types";

type ImportPreviewItem = {
  name: string;
  transport: string;
  hasConflict: boolean;
};

type ImportPreviewResult = {
  items: ImportPreviewItem[];
  error: string | null;
};

type ImportMcpDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  targetAgent: ResolvedAgentView | null;
  existingServerNames: string[];
  onImport: (
    jsonPayload: string,
    conflictStrategy: LocalMcpImportConflictStrategy
  ) => Promise<ImportLocalMcpResult>;
  t: (key: string, options?: Record<string, unknown>) => string;
};

const RESERVED_SERVER_FIELDS = new Set([
  "type",
  "command",
  "args",
  "env",
  "url",
  "httpUrl",
  "headers",
]);

function previewImportPayload(
  jsonPayload: string,
  existingServerNames: Set<string>
): ImportPreviewResult {
  const trimmed = jsonPayload.trim();
  if (!trimmed) {
    return { items: [], error: null };
  }

  try {
    const root = JSON.parse(trimmed) as unknown;
    if (!root || typeof root !== "object" || Array.isArray(root)) {
      return { items: [], error: "MCP import JSON must be an object." };
    }

    const rootObject = root as Record<string, unknown>;
    const serverMap =
      "mcpServers" in rootObject
        ? rootObject.mcpServers
        : Object.keys(rootObject).some((key) => RESERVED_SERVER_FIELDS.has(key))
          ? null
          : rootObject;

    if (!serverMap || typeof serverMap !== "object" || Array.isArray(serverMap)) {
      return {
        items: [],
        error:
          "MCP import JSON must be either a 'mcpServers' object or a map keyed by server name.",
      };
    }

    const items = Object.entries(serverMap as Record<string, unknown>).map(([name, value]) => {
      if (!value || typeof value !== "object" || Array.isArray(value)) {
        throw new Error(`MCP server '${name}' must be an object.`);
      }

      const server = value as Record<string, unknown>;
      const explicitType = typeof server.type === "string" ? server.type : "";
      const hasCommand = typeof server.command === "string" && server.command.trim().length > 0;
      const hasUrl =
        (typeof server.url === "string" && server.url.trim().length > 0) ||
        (typeof server.httpUrl === "string" && server.httpUrl.trim().length > 0);
      if (!hasCommand && !hasUrl) {
        throw new Error(`MCP server '${name}' must include either 'command' or 'url'.`);
      }

      const transport =
        explicitType === "stdio" || explicitType === "local"
          ? "stdio"
          : explicitType === "sse" || explicitType === "remote"
            ? "sse"
            : explicitType === "http"
              ? "http"
              : hasCommand
                ? "stdio"
                : "http";

      return {
        name,
        transport,
        hasConflict: existingServerNames.has(name),
      };
    });

    return { items, error: null };
  } catch (error) {
    return {
      items: [],
      error: error instanceof Error ? error.message : "Invalid MCP import JSON.",
    };
  }
}

export function ImportMcpDialog({
  open,
  onOpenChange,
  targetAgent,
  existingServerNames,
  onImport,
  t,
}: ImportMcpDialogProps) {
  const [jsonPayload, setJsonPayload] = useState("");
  const [conflictStrategy, setConflictStrategy] =
    useState<LocalMcpImportConflictStrategy>("overwrite");
  const [isImporting, setIsImporting] = useState(false);

  useEffect(() => {
    if (!open) {
      setJsonPayload("");
      setConflictStrategy("overwrite");
      setIsImporting(false);
    }
  }, [open]);

  const existingServerNameSet = useMemo(() => new Set(existingServerNames), [existingServerNames]);
  const preview = useMemo(
    () => previewImportPayload(jsonPayload, existingServerNameSet),
    [existingServerNameSet, jsonPayload]
  );
  const conflictCount = preview.items.filter((item) => item.hasConflict).length;

  const handleImport = async () => {
    if (!targetAgent || preview.error || preview.items.length === 0) {
      return;
    }

    setIsImporting(true);
    try {
      await onImport(jsonPayload, conflictStrategy);
      onOpenChange(false);
    } finally {
      setIsImporting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[90vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>{t("prototype.importMcp.title")}</DialogTitle>
          <DialogDescription>
            {t("prototype.importMcp.description", {
              agentName: targetAgent?.name ?? "",
            })}
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-4 overflow-y-auto flex-1">
          <div className="space-y-2">
            <div className="text-sm font-medium">{t("prototype.importMcp.inputLabel")}</div>
            <textarea
              value={jsonPayload}
              onChange={(event) => setJsonPayload(event.target.value)}
              className="border-input bg-background focus-visible:ring-ring min-h-56 w-full rounded-md border px-3 py-2 font-mono text-xs outline-none focus-visible:ring-2"
              placeholder={t("prototype.importMcp.placeholder")}
              spellCheck={false}
            />
            <div className="text-muted-foreground text-xs">{t("prototype.importMcp.hint")}</div>
          </div>

          {preview.error ? (
            <div className="border-destructive/30 bg-destructive/10 text-destructive rounded-md border px-3 py-2 text-xs">
              {preview.error}
            </div>
          ) : null}

          {preview.items.length > 0 ? (
            <div className="space-y-3">
              <div className="flex items-center justify-between text-sm">
                <div className="font-medium">
                  {t("prototype.importMcp.previewTitle", { count: preview.items.length })}
                </div>
                <div className="text-muted-foreground text-xs">
                  {t("prototype.importMcp.conflictCount", { count: conflictCount })}
                </div>
              </div>
              <div className="max-h-52 space-y-2 overflow-auto rounded-md border p-2">
                {preview.items.map((item) => (
                  <div
                    key={item.name}
                    className="flex items-center justify-between gap-3 rounded border px-3 py-2 text-xs"
                  >
                    <div className="min-w-0">
                      <div className="truncate font-medium">{item.name}</div>
                      <div className="text-muted-foreground">{item.transport}</div>
                    </div>
                    {item.hasConflict ? (
                      <span className="rounded border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-amber-700 dark:text-amber-300">
                        {t("prototype.importMcp.conflictBadge")}
                      </span>
                    ) : (
                      <span className="rounded border border-emerald-500/30 bg-emerald-500/10 px-2 py-1 text-emerald-700 dark:text-emerald-300">
                        {t("prototype.importMcp.newBadge")}
                      </span>
                    )}
                  </div>
                ))}
              </div>
            </div>
          ) : null}

          {conflictCount > 0 ? (
            <div className="space-y-3 rounded-md border p-3">
              <div className="text-sm font-medium">{t("prototype.importMcp.conflictStrategy")}</div>
              <RadioGroup
                value={conflictStrategy}
                onValueChange={(value) =>
                  setConflictStrategy(value as LocalMcpImportConflictStrategy)
                }
              >
                <label className="flex cursor-pointer items-start gap-3 rounded border px-3 py-2">
                  <RadioGroupItem value="overwrite" className="mt-0.5" />
                  <div>
                    <div className="text-sm font-medium">
                      {t("prototype.importMcp.overwriteTitle")}
                    </div>
                    <div className="text-muted-foreground text-xs">
                      {t("prototype.importMcp.overwriteDescription")}
                    </div>
                  </div>
                </label>
                <label className="flex cursor-pointer items-start gap-3 rounded border px-3 py-2">
                  <RadioGroupItem value="skip" className="mt-0.5" />
                  <div>
                    <div className="text-sm font-medium">{t("prototype.importMcp.skipTitle")}</div>
                    <div className="text-muted-foreground text-xs">
                      {t("prototype.importMcp.skipDescription")}
                    </div>
                  </div>
                </label>
              </RadioGroup>
            </div>
          ) : null}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("prototype.actions.cancel")}
          </Button>
          <Button
            onClick={() => void handleImport()}
            disabled={
              !targetAgent || isImporting || preview.error != null || preview.items.length === 0
            }
          >
            {t("prototype.importMcp.submit")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
