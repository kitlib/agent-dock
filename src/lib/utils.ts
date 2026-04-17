import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatInstallCount(value: number, language: string) {
  const isChinese = language.toLowerCase().startsWith("zh");

  if (isChinese) {
    if (value >= 10_000) {
      return `${(value / 10_000).toFixed(1)}万`;
    }

    if (value >= 1_000) {
      return `${(value / 1_000).toFixed(1)}千`;
    }
  } else {
    if (value >= 1_000_000) {
      return `${(value / 1_000_000).toFixed(1)}M`;
    }

    if (value >= 1_000) {
      return `${(value / 1_000).toFixed(1)}K`;
    }
  }

  return value.toLocaleString(isChinese ? "zh-CN" : "en-US");
}
