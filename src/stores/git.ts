import { defineStore } from "pinia";
import { ref } from "vue";
import type { GitInfo } from "@/types";
import { getProjectGitInfo } from "@/api/git";
import { ApiError } from "@/api/project";

/**
 * Git 分析状态。
 *
 * ⚠️ 性能：Dashboard 卡片分支标识**不**批量拉取 git（避免 N 次磁盘/git 调用），
 * 仅在进入详情页 `fetchGit` 时写入 `branchCache`，卡片据此按需显示分支。
 */
export const useGitStore = defineStore("git", () => {
  const info = ref<GitInfo | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  /** 项目 id → 分支名缓存（仅供卡片轻量展示，非持久来源） */
  const branchCache = ref<Record<number, string | null>>({});
  /** 项目 id → GitInfo 缓存（供列表按需展示 Clean/Dirty 徽标，未获取过返回 undefined） */
  const infoCache = ref<Record<number, GitInfo | null>>({});

  /** 拉取指定项目的 git 信息（只读） */
  async function fetchGit(projectId: number) {
    loading.value = true;
    error.value = null;
    try {
      info.value = await getProjectGitInfo(projectId);
      infoCache.value[projectId] = info.value;
      if (info.value?.is_git_repo) {
        branchCache.value[projectId] = info.value.branch ?? null;
      } else {
        branchCache.value[projectId] = null;
      }
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  /** 取某项目已缓存的分支名（未获取过返回 undefined） */
  function branchOf(projectId: number): string | null | undefined {
    return branchCache.value[projectId];
  }

  /** 取某项目已缓存的 GitInfo（未获取过返回 undefined，供适配层用） */
  function infoOf(projectId: number): GitInfo | null | undefined {
    return infoCache.value[projectId];
  }

  function clear() {
    info.value = null;
    loading.value = false;
    error.value = null;
  }

  return {
    info,
    loading,
    error,
    branchOf,
    infoOf,
    fetchGit,
    clear,
  };
});
