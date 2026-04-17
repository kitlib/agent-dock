import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

import { cn } from "@/lib/utils";

type MarkdownContentProps = {
  content: string;
  className?: string;
};

function isSafeHref(href?: string) {
  if (!href) {
    return false;
  }

  try {
    const url = new URL(href, "https://example.com");
    return url.protocol === "http:" || url.protocol === "https:";
  } catch {
    return false;
  }
}

export function MarkdownContent({ content, className }: MarkdownContentProps) {
  if (!content.trim()) {
    return null;
  }

  return (
    <div
      className={cn(
        "prose prose-sm prose-neutral dark:prose-invert max-w-none",
        "prose-headings:text-foreground prose-p:text-foreground prose-strong:text-foreground",
        "prose-a:text-primary prose-a:break-all",
        "prose-blockquote:border-border prose-blockquote:text-muted-foreground",
        "prose-hr:border-border prose-li:text-foreground",
        "prose-code:text-foreground prose-code:before:content-none prose-code:after:content-none",
        "prose-pre:bg-muted prose-pre:text-foreground prose-pre:overflow-x-auto",
        "prose-th:border prose-th:border-border prose-th:bg-muted/60 prose-th:px-3 prose-th:py-2",
        "prose-td:border prose-td:border-border prose-td:px-3 prose-td:py-2",
        "[&_pre]:rounded-lg [&_pre]:border [&_pre]:px-3 [&_pre]:py-3 [&_pre]:text-xs",
        "[&_table]:block [&_table]:w-full [&_table]:overflow-x-auto",
        "[&_tbody]:table-row-group [&_thead]:table-header-group [&_tr]:table-row",
        "[&_td]:table-cell [&_th]:table-cell",
        "[&_code:not(pre_*)]:bg-muted [&_code:not(pre_*)]:rounded [&_code:not(pre_*)]:px-1 [&_code:not(pre_*)]:py-0.5 [&_code:not(pre_*)]:text-[0.85em]",
        className
      )}
    >
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          a: ({ href, ...props }) => {
            if (!isSafeHref(href)) {
              return <span {...props} />;
            }

            return <a href={href} rel="noreferrer" target="_blank" {...props} />;
          },
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}
