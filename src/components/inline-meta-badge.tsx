import { cn } from "@/lib/utils";

type InlineMetaBadgeProps = {
  label: string;
  value: string | number;
  tone?: "neutral" | "blue" | "green" | "amber";
  className?: string;
};

export function InlineMetaBadge({
  label,
  value,
  tone = "neutral",
  className,
}: InlineMetaBadgeProps) {
  const toneClassName =
    tone === "blue"
      ? "border-sky-200/80 bg-sky-50/70 dark:border-sky-900/70 dark:bg-sky-950/30"
      : tone === "green"
        ? "border-emerald-200/80 bg-emerald-50/70 dark:border-emerald-900/70 dark:bg-emerald-950/30"
        : tone === "amber"
          ? "border-amber-200/80 bg-amber-50/70 dark:border-amber-900/70 dark:bg-amber-950/30"
          : "border-border/70 bg-background";
  const labelToneClassName =
    tone === "blue"
      ? "bg-sky-100/80 text-sky-700 border-sky-200/80 dark:bg-sky-900/40 dark:text-sky-200 dark:border-sky-800/70"
      : tone === "green"
        ? "bg-emerald-100/80 text-emerald-700 border-emerald-200/80 dark:bg-emerald-900/40 dark:text-emerald-200 dark:border-emerald-800/70"
        : tone === "amber"
          ? "bg-amber-100/80 text-amber-700 border-amber-200/80 dark:bg-amber-900/40 dark:text-amber-200 dark:border-amber-800/70"
          : "bg-muted/70 text-muted-foreground border-border/70";

  return (
    <span
      className={cn(
        "inline-flex items-stretch overflow-hidden rounded-md border shadow-sm",
        toneClassName,
        className
      )}
    >
      <span
        className={cn(
          "inline-flex items-center border-r px-2 py-1 text-[11px] leading-none font-medium",
          labelToneClassName
        )}
      >
        {label}
      </span>
      <span className="text-foreground inline-flex items-center px-2.5 py-1 text-[11px] leading-none font-semibold">
        {value}
      </span>
    </span>
  );
}
