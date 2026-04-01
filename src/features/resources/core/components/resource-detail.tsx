import { Button } from "@/components/ui/button";
import { installStateKey } from "@/features/shared/constants";
import type { AgentDiscoveryItem, MarketplaceDiscoveryFields } from "@/features/agents/types";

type ResourceDetailContentProps = {
  resource: AgentDiscoveryItem;
  onOpenSkillFolder: (skillPath: string) => void;
  onUpdateMarketplaceInstallState: (id: string) => void;
  t: (key: string, options?: Record<string, unknown>) => string;
};

type MarketplaceResource = AgentDiscoveryItem & MarketplaceDiscoveryFields;
type LocalResource = Exclude<AgentDiscoveryItem, MarketplaceResource>;

function MarketplaceResourceDetail({
  resource,
  onUpdateMarketplaceInstallState,
  t,
}: {
  resource: MarketplaceResource;
  onUpdateMarketplaceInstallState: (id: string) => void;
  t: ResourceDetailContentProps["t"];
}) {
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
            <div className="text-sm font-medium">{t(installStateKey[resource.installState])}</div>
            <div className="text-muted-foreground mt-1 text-xs">{resource.sourceLabel}</div>
          </div>
          <Button onClick={() => onUpdateMarketplaceInstallState(resource.id)}>
            {t(installStateKey[resource.installState])}
          </Button>
        </div>
      </div>
    </div>
  );
}

function LocalResourceDetail({
  resource,
  t,
}: {
  resource: LocalResource;
  t: ResourceDetailContentProps["t"];
}) {
  if (resource.kind === "skill") {
    return (
      <div className="space-y-4">
        <section className="space-y-2">
          <h3 className="text-sm font-semibold">Description</h3>
          <div className="bg-muted/40 rounded-lg border p-3 text-sm whitespace-pre-wrap">
            {resource.description ?? resource.summary}
          </div>
        </section>
        <section className="space-y-2">
          <h3 className="text-sm font-semibold">Markdown</h3>
          <div className="bg-muted/40 rounded-lg border p-3 text-sm whitespace-pre-wrap">
            {resource.markdown}
          </div>
        </section>
        {(resource.warnings?.length || resource.errors?.length) ? (
          <section className="space-y-2">
            <h3 className="text-sm font-semibold">Diagnostics</h3>
            <div className="space-y-2 text-sm">
              {resource.warnings?.map((warning) => (
                <div key={warning} className="rounded-lg border border-yellow-500/30 bg-yellow-500/10 p-3">
                  {warning}
                </div>
              ))}
              {resource.errors?.map((error) => (
                <div key={error} className="rounded-lg border border-red-500/30 bg-red-500/10 p-3">
                  {error}
                </div>
              ))}
            </div>
          </section>
        ) : null}
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
}

export function AgentResourceDetail({
  resource,
  onUpdateMarketplaceInstallState,
  t,
}: Omit<ResourceDetailContentProps, "onOpenSkillFolder">) {
  if (resource.origin === "marketplace") {
    return (
      <MarketplaceResourceDetail
        resource={resource}
        onUpdateMarketplaceInstallState={onUpdateMarketplaceInstallState}
        t={t}
      />
    );
  }

  return <LocalResourceDetail resource={resource} t={t} />;
}
