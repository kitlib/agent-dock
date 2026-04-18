import type { ReactNode } from "react";
import { MarkdownContent } from "@/components/markdown-content";
import type { AgentDiscoveryItem, MarketplaceDiscoveryFields } from "@/features/agents/types";

type ResourceDetailContentProps = {
  isMarketplaceDetailLoading?: boolean;
  resource: AgentDiscoveryItem;
  t: (key: string, options?: Record<string, unknown>) => string;
};

type MarketplaceResource = AgentDiscoveryItem & MarketplaceDiscoveryFields;
type LocalResource = Exclude<AgentDiscoveryItem, MarketplaceResource>;

function TextSection({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="space-y-2">
      <h3 className="text-sm font-semibold">{title}</h3>
      {children}
    </section>
  );
}

function MarkdownSkeleton() {
  return (
    <div className="bg-muted/40 rounded-lg border p-3">
      <div className="animate-pulse space-y-3">
        <div className="bg-muted h-4 w-2/5 rounded" />
        <div className="bg-muted h-4 w-full rounded" />
        <div className="bg-muted h-4 w-11/12 rounded" />
        <div className="bg-muted h-4 w-4/5 rounded" />
        <div className="bg-muted h-24 w-full rounded" />
        <div className="bg-muted h-4 w-3/5 rounded" />
        <div className="bg-muted h-4 w-5/6 rounded" />
      </div>
    </div>
  );
}

function DescriptionSkeleton() {
  return (
    <div className="bg-muted/40 rounded-lg border p-3">
      <div className="animate-pulse space-y-2">
        <div className="bg-muted h-4 w-full rounded" />
        <div className="bg-muted h-4 w-10/12 rounded" />
        <div className="bg-muted h-4 w-8/12 rounded" />
      </div>
    </div>
  );
}

function ListSection({ title, items }: { title: string; items: string[] }) {
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

function InfoSection({
  title,
  items,
}: {
  title: string;
  items: Array<{ label: string; value: string }>;
}) {
  return (
    <section className="space-y-2">
      <h3 className="text-sm font-semibold">{title}</h3>
      <div className="space-y-2">
        {items.map((item) => (
          <div key={item.label} className="bg-muted/40 rounded-lg border px-3 py-2">
            <div className="text-muted-foreground text-[11px] uppercase tracking-wide">
              {item.label}
            </div>
            <div className="mt-1 text-sm break-all">{item.value}</div>
          </div>
        ))}
      </div>
    </section>
  );
}

function MarketplaceResourceDetail({
  isMarketplaceDetailLoading = false,
  resource,
  t,
}: {
  isMarketplaceDetailLoading?: boolean;
  resource: MarketplaceResource;
  t: ResourceDetailContentProps["t"];
}) {
  return (
    <div className="space-y-4">
      <TextSection title={t("prototype.detail.description")}>
        {isMarketplaceDetailLoading ? (
          <DescriptionSkeleton />
        ) : (
          <div className="bg-muted/40 rounded-lg border p-3 text-sm whitespace-pre-wrap">
            {resource.description}
          </div>
        )}
      </TextSection>

      {resource.kind === "skill" ? (
        <TextSection title="SKILL.md">
          {resource.markdown?.trim() ? (
            <div className="bg-muted/40 rounded-lg border p-3 text-sm">
              <MarkdownContent content={resource.markdown} />
            </div>
          ) : isMarketplaceDetailLoading ? (
            <MarkdownSkeleton />
          ) : null}
        </TextSection>
      ) : null}
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
        <TextSection title={t("prototype.detail.description")}>
          <div className="bg-muted/40 rounded-lg border p-3 text-sm whitespace-pre-wrap">
            {resource.description ?? resource.summary}
          </div>
        </TextSection>
        <TextSection title="SKILL.md">
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
        {resource.warnings?.length || resource.errors?.length ? (
          <TextSection title="Diagnostics">
            <div className="space-y-2 text-sm">
              {resource.warnings?.map((warning) => (
                <div
                  key={warning}
                  className="rounded-lg border border-yellow-500/30 bg-yellow-500/10 p-3"
                >
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
    const readResultItems = [
      { label: "Transport", value: resource.transport },
      { label: "Source Agent", value: resource.agentName ?? resource.sourceLabel ?? "Unknown" },
      { label: "Scope", value: resource.scope ?? "Unknown" },
      { label: "Updated", value: resource.updatedAt },
      ...(resource.endpoint ? [{ label: "Endpoint", value: resource.endpoint }] : []),
      ...(resource.projectPath ? [{ label: "Project Path", value: resource.projectPath }] : []),
      ...(resource.configPath ? [{ label: "Config Path", value: resource.configPath }] : []),
    ];

    return (
      <div className="space-y-4">
        <TextSection title="Summary">
          <div className="bg-muted/40 rounded-lg border p-3 text-sm whitespace-pre-wrap">
            {resource.summary}
          </div>
        </TextSection>
        <InfoSection title="Read Result" items={readResultItems} />
        <TextSection title="Notes">
          <div className="bg-muted/40 rounded-lg border p-3 text-sm whitespace-pre-wrap">
            <MarkdownContent content={resource.document} />
          </div>
        </TextSection>
        <TextSection title="Masked Config">
          <pre className="bg-muted/40 overflow-x-auto rounded-lg border p-3 text-xs">
            {resource.config}
          </pre>
        </TextSection>
        {resource.warnings?.length || resource.errors?.length ? (
          <TextSection title="Diagnostics">
            <div className="space-y-2 text-sm">
              {resource.warnings?.map((warning) => (
                <div
                  key={warning}
                  className="rounded-lg border border-yellow-500/30 bg-yellow-500/10 p-3"
                >
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
  isMarketplaceDetailLoading = false,
  resource,
  t,
}: ResourceDetailContentProps) {
  if (resource.origin === "marketplace") {
    return (
      <MarketplaceResourceDetail
        isMarketplaceDetailLoading={isMarketplaceDetailLoading}
        resource={resource}
        t={t}
      />
    );
  }

  return <LocalResourceDetail resource={resource} t={t} />;
}
