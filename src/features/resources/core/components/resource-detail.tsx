import type { ReactNode } from "react";
import { MarkdownContent } from "@/components/markdown-content";
import { Button } from "@/components/ui/button";
import type { AgentDiscoveryItem, MarketplaceDiscoveryFields } from "@/features/agents/types";
import { installStateKey } from "@/features/shared/constants";

type ResourceDetailContentProps = {
  resource: AgentDiscoveryItem;
  onUpdateMarketplaceInstallState: (id: string) => void;
  t: (key: string, options?: Record<string, unknown>) => string;
};

type MarketplaceResource = AgentDiscoveryItem & MarketplaceDiscoveryFields;
type LocalResource = Exclude<AgentDiscoveryItem, MarketplaceResource>;

function StatCard({
  label,
  value,
}: {
  label: string;
  value: string | number;
}) {
  return (
    <div className="bg-background rounded-lg border p-3">
      <div className="text-muted-foreground text-xs">{label}</div>
      <div className="mt-1 font-medium">{value}</div>
    </div>
  );
}

function TextSection({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="space-y-2">
      <h3 className="text-sm font-semibold">{title}</h3>
      {children}
    </section>
  );
}

function ListSection({
  title,
  items,
}: {
  title: string;
  items: string[];
}) {
  return (
    <section className="space-y-2">
      <h3 className="text-sm font-semibold">{title}</h3>
      <ul className="space-y-2 text-sm">
        {items.map((item) => (
          <li key={item} className="bg-muted/40 rounded-lg border px-3 py-2">
            {item}
          </li>
        ))}
      </ul>
    </section>
  );
}

function MarketplaceResourceDetail({
  resource,
  onUpdateMarketplaceInstallState,
  t,
}: {
  resource: MarketplaceResource;
  onUpdateMarketplaceInstallState: (id: string) => void;
  t: ResourceDetailContentProps["t"];
}) {
  const installStateLabel = t(installStateKey[resource.installState]);

  return (
    <div className="space-y-4">
      <section className="grid grid-cols-2 gap-3 text-sm">
        <StatCard label={t("prototype.detail.source")} value={resource.sourceLabel} />
        <StatCard label={t("prototype.detail.version")} value={resource.version} />
      </section>

      <section className="grid grid-cols-2 gap-3 text-sm">
        <StatCard label={t("prototype.detail.author")} value={resource.author} />
        <StatCard label={t("prototype.detail.downloads")} value={resource.downloads} />
      </section>

      <ListSection title={t("prototype.detail.highlights")} items={resource.highlights} />

      <div className="bg-background space-y-3 rounded-lg border p-4">
        <div className="flex items-center justify-between gap-3">
          <div>
            <div className="text-sm font-medium">{installStateLabel}</div>
            <div className="text-muted-foreground mt-1 text-xs">{resource.sourceLabel}</div>
          </div>
          <Button onClick={() => onUpdateMarketplaceInstallState(resource.id)}>{installStateLabel}</Button>
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
    const markdownContent = resource.markdown ?? "";

    return (
      <div className="space-y-4">
        <TextSection title="Description">
          <div className="bg-muted/40 rounded-lg border p-3 text-sm whitespace-pre-wrap">
            {resource.description ?? resource.summary}
          </div>
        </TextSection>
        <TextSection title="Markdown">
          {markdownContent.trim() ? (
            <div className="bg-muted/40 rounded-lg border p-3 text-sm">
              <MarkdownContent content={markdownContent} />
            </div>
          ) : (
            <div className="text-muted-foreground bg-muted/40 rounded-lg border p-3 text-sm">
              No markdown content available.
            </div>
          )}
        </TextSection>
        {(resource.warnings?.length || resource.errors?.length) ? (
          <TextSection title="Diagnostics">
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
          </TextSection>
        ) : null}
      </div>
    );
  }

  if (resource.kind === "mcp") {
    return (
      <div className="space-y-4">
        <TextSection title={t("prototype.detail.document")}>
          <div className="bg-muted/40 rounded-lg border p-3 text-sm whitespace-pre-wrap">
            {resource.document}
          </div>
        </TextSection>
        <TextSection title={t("prototype.detail.config")}>
          <pre className="bg-muted/40 overflow-x-auto rounded-lg border p-3 text-xs">
            {resource.config}
          </pre>
        </TextSection>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <TextSection title={t("prototype.detail.prompt")}>
        <div className="bg-muted/40 rounded-lg border p-3 text-sm whitespace-pre-wrap">
          {resource.prompt}
        </div>
      </TextSection>
      <ListSection title={t("prototype.detail.capabilities")} items={resource.capabilities} />
    </div>
  );
}

export function AgentResourceDetail({
  resource,
  onUpdateMarketplaceInstallState,
  t,
}: ResourceDetailContentProps) {
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
