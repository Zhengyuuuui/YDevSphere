/**
 * 最近打开项目：基于 localStorage 持久化的轻量工具。
 * 仅记录项目 id 与时间戳，具体项目数据仍来自后端 getProjects / getProjectDetail。
 */

const RECENT_KEY = "ydevsphere:recent-projects";
const MAX_RECENT = 8;

export interface RecentEntry {
  id: number;
  openedAt: number;
}

function read(): RecentEntry[] {
  try {
    const raw = localStorage.getItem(RECENT_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as RecentEntry[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function write(entries: RecentEntry[]) {
  try {
    localStorage.setItem(RECENT_KEY, JSON.stringify(entries));
  } catch {
    // localStorage 不可用（隐私模式等）时静默失败
  }
}

/** 获取最近打开的项目 id 列表（按最近优先） */
export function getRecentIds(): number[] {
  return read().map((e) => e.id);
}

/** 获取某项目最近打开的时间戳（毫秒）；未记录返回 null */
export function getRecentOpenedAt(id: number): number | null {
  const entry = read().find((e) => e.id === id);
  return entry ? entry.openedAt : null;
}

/** 记录一次项目打开（去重，把最近打开移到最前） */
export function recordRecent(id: number) {
  const entries = read().filter((e) => e.id !== id);
  entries.unshift({ id, openedAt: Date.now() });
  write(entries.slice(0, MAX_RECENT));
}
