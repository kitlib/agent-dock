import { useCallback, useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { emit } from "@tauri-apps/api/event";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useTheme } from "@/components/theme-provider";
import { UpdaterDialog } from "@/components/updater-dialog";
import { TitleBar } from "@/components/title-bar";
import { WindowFrame } from "@/components/window-frame";
import { LanguageToggle } from "@/components/language-toggle";
import { ShortcutInput } from "@/components/shortcut-input";
import { Moon, Sun, Monitor, Palette, Keyboard, RefreshCw } from "lucide-react";
import { registerShortcut, unregisterShortcut } from "@/lib/shortcut";
import { toggleWindow } from "@/lib/window";
import { toast } from "sonner";
import { Toaster } from "@/components/ui/sonner";
import { useAppTranslation } from "@/hooks/use-app-translation";
import type { UpdateCheckResult } from "@/lib/updater";

const SHORTCUT_KEY = "global-shortcut-show-main";

type SettingSection = "appearance" | "shortcut" | "updates";

export default function SettingsPage() {
  const [shortcut, setShortcut] = useState<string>("");
  const [appVersion, setAppVersion] = useState("");
  const [activeSection, setActiveSection] = useState<SettingSection>("appearance");
  const [manualCheckToken, setManualCheckToken] = useState(0);
  const [showNoUpdate, setShowNoUpdate] = useState(false);
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);
  const { t } = useAppTranslation();
  const { theme, setTheme } = useTheme();

  const handleShowMainWindow = useCallback(async () => {
    await toggleWindow("main");
  }, []);

  useEffect(() => {
    // Load saved shortcut
    const savedShortcut = localStorage.getItem(SHORTCUT_KEY);
    if (savedShortcut) {
      setShortcut(savedShortcut);
      registerShortcut(savedShortcut, handleShowMainWindow);
    }
  }, [handleShowMainWindow]);

  useEffect(() => {
    void getVersion().then(setAppVersion);
  }, []);

  const handleShortcutChange = async (newShortcut: string) => {
    const oldShortcut = shortcut;
    setShortcut(newShortcut);

    if (newShortcut) {
      localStorage.setItem(SHORTCUT_KEY, newShortcut);
      await registerShortcut(newShortcut, handleShowMainWindow, oldShortcut);
      // Notify main window to update shortcut
      await emit("shortcut-changed", { shortcut: newShortcut });
      toast.success(t("settings.shortcut.setSuccess", { shortcut: newShortcut }));
    } else {
      localStorage.removeItem(SHORTCUT_KEY);
      if (oldShortcut) {
        await unregisterShortcut(oldShortcut);
      }
      // Notify main window to clear shortcut
      await emit("shortcut-changed", { shortcut: "" });
      toast.info(t("settings.shortcut.cleared"));
    }
  };

  const menuItems = [
    {
      id: "appearance" as SettingSection,
      label: t("settings.appearance.title"),
      icon: Palette,
    },
    {
      id: "shortcut" as SettingSection,
      label: t("settings.shortcut.title"),
      icon: Keyboard,
    },
    {
      id: "updates" as SettingSection,
      label: t("settings.updates.title"),
      icon: RefreshCw,
    },
  ];

  const handleCheckUpdate = () => {
    setShowNoUpdate(false);
    setIsCheckingUpdate(true);
    setManualCheckToken((current) => current + 1);
  };

  const handleUpdateCheckResult = (result: UpdateCheckResult) => {
    setIsCheckingUpdate(false);
    if (result.status === "up-to-date") {
      setShowNoUpdate(true);
      return;
    }

    if (result.status === "error") {
      toast.error(t("updater.checkFailed"));
    }
  };

  return (
    <WindowFrame
      titleBar={<TitleBar title={t("settings.title")} showMaximize={false} />}
      contentClassName="flex flex-1 overflow-hidden"
    >
      <Toaster />
      <UpdaterDialog manualCheckToken={manualCheckToken} onCheckResult={handleUpdateCheckResult} />
      <aside className="border-border flex w-40 flex-col border-r p-4">
        <nav className="flex-1 space-y-1">
          {menuItems.map((item) => {
            const Icon = item.icon;
            return (
              <button
                key={item.id}
                onClick={() => setActiveSection(item.id)}
                className={cn(
                  "flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors",
                  activeSection === item.id
                    ? "bg-accent text-accent-foreground font-medium"
                    : "text-muted-foreground hover:bg-accent/50 hover:text-foreground"
                )}
              >
                <Icon className="h-4 w-4" />
                {item.label}
              </button>
            );
          })}
        </nav>
      </aside>

      <div className="flex-1 overflow-auto">
        <div className="max-w-3xl p-4">
          {activeSection === "appearance" && (
            <div className="space-y-4">
              <div>
                <h2 className="mb-1 text-lg font-semibold">{t("settings.appearance.title")}</h2>
                <p className="text-muted-foreground text-sm">
                  {t("settings.appearance.description")}
                </p>
              </div>

              <div className="space-y-0">
                <div className="flex items-center justify-between py-2.5">
                  <label className="text-sm font-medium">{t("settings.appearance.theme")}</label>
                  <div className="flex gap-2">
                    <Button
                      variant={theme === "light" ? "default" : "outline"}
                      size="sm"
                      onClick={() => setTheme("light")}
                      className="flex items-center gap-1.5"
                    >
                      <Sun className="h-3.5 w-3.5" />
                      {t("settings.appearance.light")}
                    </Button>
                    <Button
                      variant={theme === "dark" ? "default" : "outline"}
                      size="sm"
                      onClick={() => setTheme("dark")}
                      className="flex items-center gap-1.5"
                    >
                      <Moon className="h-3.5 w-3.5" />
                      {t("settings.appearance.dark")}
                    </Button>
                    <Button
                      variant={theme === "system" ? "default" : "outline"}
                      size="sm"
                      onClick={() => setTheme("system")}
                      className="flex items-center gap-1.5"
                    >
                      <Monitor className="h-3.5 w-3.5" />
                      {t("settings.appearance.system")}
                    </Button>
                  </div>
                </div>

                <div className="border-t" />

                <div className="flex items-center justify-between py-2.5">
                  <label className="text-sm font-medium">{t("settings.appearance.language")}</label>
                  <LanguageToggle />
                </div>
              </div>
            </div>
          )}

          {activeSection === "shortcut" && (
            <div className="space-y-4">
              <div>
                <h2 className="mb-1 text-lg font-semibold">{t("settings.shortcut.title")}</h2>
                <p className="text-muted-foreground text-sm">
                  {t("settings.shortcut.description")}
                </p>
              </div>

              <div className="space-y-0">
                <div className="flex items-center justify-between py-2.5">
                  <div className="flex-1">
                    <label className="text-sm font-medium">{t("settings.shortcut.showMain")}</label>
                    <p className="text-muted-foreground mt-0.5 text-xs">
                      {t("settings.shortcut.showMainDesc")}
                    </p>
                  </div>
                  <ShortcutInput value={shortcut} onChange={handleShortcutChange} />
                </div>
              </div>
            </div>
          )}

          {activeSection === "updates" && (
            <div className="space-y-4">
              <div>
                <h2 className="mb-1 text-lg font-semibold">{t("settings.updates.title")}</h2>
                <p className="text-muted-foreground text-sm">{t("settings.updates.description")}</p>
              </div>

              <div className="space-y-0">
                <div className="flex items-center justify-between py-2.5">
                  <div>
                    <label className="text-sm font-medium">
                      {t("settings.updates.currentVersion")}
                    </label>
                    <p className="text-muted-foreground mt-0.5 text-xs">{appVersion || "-"}</p>
                  </div>
                  <Button variant="outline" onClick={handleCheckUpdate} disabled={isCheckingUpdate}>
                    <RefreshCw
                      className={`mr-2 h-4 w-4 ${isCheckingUpdate ? "animate-spin" : ""}`}
                    />
                    {isCheckingUpdate ? t("updater.checking") : t("updater.checkForUpdates")}
                  </Button>
                </div>
                {showNoUpdate ? (
                  <>
                    <div className="border-t" />
                    <div className="text-muted-foreground py-2.5 text-sm">
                      {t("updater.upToDate")}
                    </div>
                  </>
                ) : null}
              </div>
            </div>
          )}
        </div>
      </div>
    </WindowFrame>
  );
}
