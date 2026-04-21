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
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { EditableLocalMcp, McpResource, UpdateLocalMcpInput } from "@/features/agents/types";

type EditMcpDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  targetResource: McpResource | null;
  initialValue: EditableLocalMcp | null;
  onSubmit: (nextServer: UpdateLocalMcpInput) => Promise<void>;
  t: (key: string, options?: Record<string, unknown>) => string;
};

type TransportOption = EditableLocalMcp["transport"];

function toPrettyJson(value: Record<string, string>) {
  return JSON.stringify(value, null, 2);
}

function parseStringMap(value: string, fieldLabel: string): Record<string, string> {
  const trimmed = value.trim();
  if (!trimmed) {
    return {};
  }

  const parsed = JSON.parse(trimmed) as unknown;
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error(`${fieldLabel} must be a JSON object.`);
  }

  return Object.entries(parsed as Record<string, unknown>).reduce<Record<string, string>>(
    (result, [key, entry]) => {
      if (typeof entry !== "string") {
        throw new Error(`${fieldLabel} values must be strings.`);
      }
      result[key] = entry;
      return result;
    },
    {}
  );
}

function getTransportOptions(agentType?: string): TransportOption[] {
  if (agentType === "opencode") {
    return ["stdio", "sse"];
  }

  return ["stdio", "http", "sse"];
}

export function EditMcpDialog({
  open,
  onOpenChange,
  targetResource,
  initialValue,
  onSubmit,
  t,
}: EditMcpDialogProps) {
  const [serverName, setServerName] = useState("");
  const [transport, setTransport] = useState<TransportOption>("stdio");
  const [command, setCommand] = useState("");
  const [argsText, setArgsText] = useState("");
  const [envText, setEnvText] = useState("{}");
  const [url, setUrl] = useState("");
  const [headersText, setHeadersText] = useState("{}");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const transportOptions = useMemo(
    () => getTransportOptions(targetResource?.agentType),
    [targetResource?.agentType]
  );

  useEffect(() => {
    if (!open || !initialValue) {
      if (!open) {
        setErrorMessage(null);
        setIsSubmitting(false);
      }
      return;
    }

    setServerName(initialValue.serverName);
    setTransport(initialValue.transport);
    setCommand(initialValue.command ?? "");
    setArgsText(initialValue.args.join("\n"));
    setEnvText(toPrettyJson(initialValue.env));
    setUrl(initialValue.url ?? "");
    setHeadersText(toPrettyJson(initialValue.headers));
    setErrorMessage(null);
    setIsSubmitting(false);
  }, [initialValue, open]);

  useEffect(() => {
    if (!transportOptions.includes(transport)) {
      setTransport(transportOptions[0] ?? "stdio");
    }
  }, [transport, transportOptions]);

  const handleSubmit = async () => {
    try {
      const nextServerName = serverName.trim();
      if (!nextServerName) {
        throw new Error(t("prototype.editMcp.validation.serverNameRequired"));
      }

      const nextCommand = command.trim();
      const nextUrl = url.trim();
      if (transport === "stdio" && !nextCommand) {
        throw new Error(t("prototype.editMcp.validation.commandRequired"));
      }
      if ((transport === "http" || transport === "sse") && !nextUrl) {
        throw new Error(t("prototype.editMcp.validation.urlRequired"));
      }

      const nextServer: UpdateLocalMcpInput = {
        serverName: nextServerName,
        transport,
        command: transport === "stdio" ? nextCommand : null,
        args:
          transport === "stdio"
            ? argsText
                .split(/\r?\n/)
                .map((value) => value.trim())
                .filter((value) => value.length > 0)
            : [],
        env:
          transport === "stdio" ? parseStringMap(envText, t("prototype.editMcp.fields.env")) : {},
        url: transport === "stdio" ? null : nextUrl,
        headers:
          transport === "stdio"
            ? {}
            : parseStringMap(headersText, t("prototype.editMcp.fields.headers")),
      };

      setErrorMessage(null);
      setIsSubmitting(true);
      await onSubmit(nextServer);
      onOpenChange(false);
    } catch (error) {
      setErrorMessage(
        error instanceof Error ? error.message : t("prototype.feedback.updateMcpFailed")
      );
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[90vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>{t("prototype.editMcp.title")}</DialogTitle>
          <DialogDescription>
            {t("prototype.editMcp.description", {
              name: targetResource?.name ?? "",
              agentName: targetResource?.agentName ?? "",
            })}
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-4 overflow-y-auto flex-1">
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <div className="text-sm font-medium">{t("prototype.editMcp.fields.serverName")}</div>
              <Input value={serverName} onChange={(event) => setServerName(event.target.value)} />
            </div>
            <div className="space-y-2">
              <div className="text-sm font-medium">{t("prototype.editMcp.fields.transport")}</div>
              <Select
                value={transport}
                onValueChange={(value) => setTransport(value as TransportOption)}
              >
                <SelectTrigger className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {transportOptions.map((option) => (
                    <SelectItem key={option} value={option}>
                      {option}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>

          {transport === "stdio" ? (
            <>
              <div className="space-y-2">
                <div className="text-sm font-medium">{t("prototype.editMcp.fields.command")}</div>
                <Input value={command} onChange={(event) => setCommand(event.target.value)} />
              </div>
              <div className="space-y-2">
                <div className="text-sm font-medium">{t("prototype.editMcp.fields.args")}</div>
                <textarea
                  value={argsText}
                  onChange={(event) => setArgsText(event.target.value)}
                  className="border-input bg-background focus-visible:ring-ring min-h-32 w-full rounded-md border px-3 py-2 font-mono text-xs outline-none focus-visible:ring-2"
                  placeholder={t("prototype.editMcp.placeholders.args")}
                  spellCheck={false}
                />
                <div className="text-muted-foreground text-xs">
                  {t("prototype.editMcp.hints.args")}
                </div>
              </div>
              <div className="space-y-2">
                <div className="text-sm font-medium">{t("prototype.editMcp.fields.env")}</div>
                <textarea
                  value={envText}
                  onChange={(event) => setEnvText(event.target.value)}
                  className="border-input bg-background focus-visible:ring-ring min-h-40 w-full rounded-md border px-3 py-2 font-mono text-xs outline-none focus-visible:ring-2"
                  placeholder="{}"
                  spellCheck={false}
                />
              </div>
            </>
          ) : (
            <>
              <div className="space-y-2">
                <div className="text-sm font-medium">{t("prototype.editMcp.fields.url")}</div>
                <Input value={url} onChange={(event) => setUrl(event.target.value)} />
              </div>
              <div className="space-y-2">
                <div className="text-sm font-medium">{t("prototype.editMcp.fields.headers")}</div>
                <textarea
                  value={headersText}
                  onChange={(event) => setHeadersText(event.target.value)}
                  className="border-input bg-background focus-visible:ring-ring min-h-40 w-full rounded-md border px-3 py-2 font-mono text-xs outline-none focus-visible:ring-2"
                  placeholder="{}"
                  spellCheck={false}
                />
              </div>
            </>
          )}

          {errorMessage ? (
            <div className="border-destructive/30 bg-destructive/10 text-destructive rounded-md border px-3 py-2 text-xs">
              {errorMessage}
            </div>
          ) : null}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("prototype.actions.cancel")}
          </Button>
          <Button onClick={() => void handleSubmit()} disabled={isSubmitting || !initialValue}>
            {t("prototype.editMcp.submit")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
