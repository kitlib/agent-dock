import { useEffect, useRef, useState } from "react";
import { useUpdater } from "@/hooks/use-updater";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Progress } from "@/components/ui/progress";
import { useTranslation } from "react-i18next";
import type { UpdateCheckResult } from "@/lib/updater";

interface UpdaterDialogProps {
  manualCheckToken?: number;
  onCheckResult?: (result: UpdateCheckResult) => void;
}

export function UpdaterDialog({ manualCheckToken, onCheckResult }: UpdaterDialogProps) {
  const { update, downloading, progress, checkUpdate, installUpdate } = useUpdater();
  const [open, setOpen] = useState(false);
  const hasAutoCheckedRef = useRef(false);
  const lastManualCheckTokenRef = useRef<number | null>(null);
  const { t } = useTranslation();

  useEffect(() => {
    if (manualCheckToken != null || hasAutoCheckedRef.current) {
      return;
    }

    hasAutoCheckedRef.current = true;
    void checkUpdate();
  }, [manualCheckToken, checkUpdate]);

  useEffect(() => {
    if (manualCheckToken == null || manualCheckToken === lastManualCheckTokenRef.current) {
      return;
    }

    lastManualCheckTokenRef.current = manualCheckToken;
    void checkUpdate().then((result) => {
      if (result.status === "available") {
        setOpen(true);
      }
      onCheckResult?.(result);
    });
  }, [manualCheckToken, checkUpdate, onCheckResult]);

  useEffect(() => {
    if (update) {
      setOpen(true);
    }
  }, [update]);

  const handleInstall = () => {
    void installUpdate();
  };

  const handleCancel = () => {
    setOpen(false);
  };

  const getProgressPercentage = () => {
    if (!progress || progress.event === "Started") return 0;
    const { downloaded, contentLength } = progress.data || {};
    if (!contentLength) return 0;
    if (progress.event === "Finished") return 100;
    return Math.round(((downloaded ?? 0) / contentLength) * 100);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {downloading ? t("updater.downloading") : t("updater.updateAvailable")}
          </DialogTitle>
          <DialogDescription>
            {downloading ? (
              <div className="space-y-2">
                <p>{t("updater.installingVersion", { version: update?.version })}</p>
                <Progress value={getProgressPercentage()} />
              </div>
            ) : (
              <div className="space-y-2">
                <p>{t("updater.versionAvailable", { version: update?.version })}</p>
                {update?.body && (
                  <div className="bg-muted mt-2 rounded-md p-3 text-sm">
                    <p className="font-semibold">{t("updater.releaseNotes")}</p>
                    <p className="mt-1 whitespace-pre-wrap">{update.body}</p>
                  </div>
                )}
              </div>
            )}
          </DialogDescription>
        </DialogHeader>
        {!downloading && (
          <DialogFooter>
            <Button variant="outline" onClick={handleCancel}>
              {t("updater.later")}
            </Button>
            <Button onClick={handleInstall}>{t("updater.installNow")}</Button>
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  );
}
