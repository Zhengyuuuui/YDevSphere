import { defineStore } from "pinia";
import { ref } from "vue";
import type { ProjectMemory } from "@/types";
import {
  ensureProjectMemory,
  getProjectMemory,
  updateProjectMemory,
} from "@/api/memory";
import { ApiError } from "@/api/project";

/** 内存中记录「已启用记忆」的项目 id 集合（用于卡片标识；非持久来源） */
const enabledProjectIds = ref<Set<number>>(new Set());

/**
 * 项目记忆状态。
 *
 * ⚠️ 安全红线：`enable` / `update` 仅在用户显式点击「启用/更新」时调用，
 * 并置 `authorized: true`；`skip` 不触发任何写操作。
 */
export const useMemoryStore = defineStore("memory", () => {
  const memory = ref<ProjectMemory | null>(null);
  const enabled = ref(false);
  const loading = ref(false);
  const error = ref<string | null>(null);

  /** 读取指定项目的记忆状态（只读） */
  async function fetchMemory(projectId: number) {
    loading.value = true;
    error.value = null;
    try {
      const result = await getProjectMemory(projectId);
      memory.value = result;
      enabled.value = Boolean(result);
      if (result) enabledProjectIds.value.add(projectId);
      else enabledProjectIds.value.delete(projectId);
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  /** 启用项目记忆（用户点击「启用」后调用，authorized = true） */
  async function enable(projectId: number, packageManager: string | null) {
    loading.value = true;
    error.value = null;
    try {
      memory.value = await ensureProjectMemory(projectId, packageManager, true);
      enabled.value = true;
      enabledProjectIds.value.add(projectId);
      return true;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
      return false;
    } finally {
      loading.value = false;
    }
  }

  /** 跳过启用（不调用任何写接口） */
  function skip() {
    memory.value = null;
    enabled.value = false;
    error.value = null;
  }

  /** 更新项目记忆（用户点击「更新」后调用，authorized = true） */
  async function update(
    projectId: number,
    packageManager: string | null,
    stack: string[] | null
  ) {
    loading.value = true;
    error.value = null;
    try {
      memory.value = await updateProjectMemory(projectId, packageManager, stack, true);
      enabledProjectIds.value.add(projectId);
      return true;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
      return false;
    } finally {
      loading.value = false;
    }
  }

  /** 项目是否已启用记忆（供卡片标识；仅覆盖本次会话中读取/启用过的项目） */
  function hasMemory(projectId: number): boolean {
    return enabledProjectIds.value.has(projectId);
  }

  function clear() {
    memory.value = null;
    enabled.value = false;
    error.value = null;
    loading.value = false;
  }

  return {
    memory,
    enabled,
    loading,
    error,
    hasMemory,
    fetchMemory,
    enable,
    skip,
    update,
    clear,
  };
});
