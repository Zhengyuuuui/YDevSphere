import type { GitStatus } from "@/types";
import { t } from "./i18n";

/** 格式化毫秒为可读耗时（如 "1.2s" / "850ms"） */
export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

/** 简单日期格式化：YYYY-MM-DD HH:mm */
export function formatDateTime(value: string | null | undefined): string {
  if (!value) return t("common.none");
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) return value;
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

/**
 * 格式化 Git 工作区状态为可读文本。
 * Rust 枚举序列化为 `{ Clean: true }` 或 `{ Dirty: { changed_files } }`。
 */
export function formatGitStatus(status: GitStatus | null): string {
  if (!status) return "—";
  if ("Clean" in status && status.Clean) return t("git.clean");
  if ("Dirty" in status && status.Dirty) {
    return t("git.dirty", { count: status.Dirty.changed_files });
  }
  return "—";
}

/** 是否为 Dirty 状态 */
export function isGitDirty(status: GitStatus | null): boolean {
  return Boolean(status && "Dirty" in status && status.Dirty);
}
