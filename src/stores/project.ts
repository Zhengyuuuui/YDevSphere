import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { Project, ProjectDetail, ProjectKind } from "@/types";
import {
  getProjects,
  getProjectDetail,
  ApiError,
  type WorkspaceFilter,
  type ProjectSortBy,
} from "@/api/project";
import { getRecentIds, recordRecent } from "@/lib/recent";

/** 项目相关状态：列表 / 详情 / 加载 / 错误 */
export const useProjectStore = defineStore("project", () => {
  const projects = ref<Project[]>([]);
  const detail = ref<ProjectDetail | null>(null);
  const detailLoading = ref(false);
  const listLoading = ref(false);
  const error = ref<string | null>(null);

  /** 最近打开的项目（按最近优先，从 localStorage 记录 + 项目列表匹配） */
  const recentProjects = computed<Project[]>(() => {
    const recentIds = getRecentIds();
    if (recentIds.length === 0) return [];
    return recentIds
      .map((id) => projects.value.find((p) => p.id === id))
      .filter((p): p is Project => Boolean(p));
  });

  /** 拉取项目列表（可传排序 / 工作区筛选 / 类型筛选 / 父项目 id） */
  async function fetchProjects(
    sortBy?: ProjectSortBy,
    workspaceFilter?: WorkspaceFilter,
    kindFilter?: ProjectKind,
    parentIdFilter?: number | null
  ) {
    listLoading.value = true;
    error.value = null;
    try {
      projects.value = await getProjects(
        sortBy,
        workspaceFilter,
        kindFilter,
        parentIdFilter
      );
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
    } finally {
      listLoading.value = false;
    }
  }

  /** 拉取单个项目详情 */
  async function fetchProjectDetail(id: number) {
    detailLoading.value = true;
    detail.value = null;
    error.value = null;
    try {
      detail.value = await getProjectDetail(id);
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
    } finally {
      detailLoading.value = false;
    }
  }

  /** 记录一次项目打开（详情页调用） */
  function markOpened(id: number) {
    recordRecent(id);
  }

  /** 用扫描结果替换项目列表（扫描完成后调用） */
  function setProjects(list: Project[]) {
    projects.value = list;
  }

  return {
    projects,
    detail,
    detailLoading,
    listLoading,
    error,
    recentProjects,
    fetchProjects,
    fetchProjectDetail,
    markOpened,
    setProjects,
  };
});
