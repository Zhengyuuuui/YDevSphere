import { defineStore } from "pinia";
import { computed, ref } from "vue";
import {
  selectWorkspace,
  ApiError,
} from "@/api/project";
import { getWorkspaces, setWorkspaces as persistWorkspaces } from "@/api/workspaces";
import { resetAppState } from "@/api/settings";
import { useEditorStore } from "@/stores/editor";

/**
 * 旧 localStorage 键（仅用于过渡期读取兜底）。
 *
 * 迁移说明（任务 V02-WS-FRONTEND）：
 * 后端已提供 `get_workspaces` / `set_workspaces` 集合接口（权威源）。
 * 前端持久化主路径已改为后端；localStorage 仅保留为**过渡读取兜底**——
 * 升级后首次启动若后端无集合而 localStorage 有，则推给后端后清除，可后续完全移除。
 */
const LEGACY_WORKSPACES_KEY = "ydevsphere.workspaces";

/** 读取旧 localStorage 工作区集合（过渡兼容；损坏/不可用返回空） */
function readLegacyWorkspaces(): string[] {
  try {
    const raw = localStorage.getItem(LEGACY_WORKSPACES_KEY);
    if (!raw) return [];
    const arr = JSON.parse(raw);
    if (Array.isArray(arr)) {
      return arr.filter((s): s is string => typeof s === "string" && s.trim().length > 0);
    }
  } catch {
    // 损坏 / 不可解析时忽略，回退空列表
  }
  return [];
}

/** 清除旧 localStorage 工作区集合（迁移完成后调用） */
function clearLegacyWorkspaces() {
  try {
    localStorage.removeItem(LEGACY_WORKSPACES_KEY);
  } catch {
    // localStorage 不可用时静默失败
  }
}

/** 应用设置：多值工作区集合（后端权威源）+ 启动恢复 / 持久化 */
export const useSettingsStore = defineStore("settings", () => {
  /** 工作区集合（多值，Documents + Desktop + 手动目录；以后端 get_workspaces 为权威） */
  const workspaces = ref<string[]>([]);
  const selecting = ref(false);
  /** 启动恢复中的 loading（避免闪烁 / 误跳转） */
  const restoring = ref(true);
  const error = ref<string | null>(null);

  /** 兼容派生：当前主工作区（集合首项；无集合时为 null）。既有页面引用保留。 */
  const workspacePath = computed<string | null>(() => workspaces.value[0] ?? null);

  /**
   * 从后端恢复工作区集合（权威源）。返回恢复到的首个路径（可能为 null）。
   * 过渡兼容：后端无集合时读旧 localStorage，若有则推给后端并清除 localStorage。
   */
  async function restore(): Promise<string | null> {
    restoring.value = true;
    error.value = null;
    try {
      // 1) 优先读后端权威源
      const backend = await getWorkspaces();
      if (backend.length > 0) {
        workspaces.value = backend;
        // 后端已有集合 → 清除旧 localStorage（过渡数据不再需要）
        clearLegacyWorkspaces();
        return backend[0];
      }

      // 2) 后端为空 → 尝试旧 localStorage 过渡迁移
      const legacy = readLegacyWorkspaces();
      if (legacy.length > 0) {
        workspaces.value = legacy;
        // 推给后端（setWorkspaces 整表替换 + 镜像 workspace_path），成功后清除 localStorage
        try {
          await persistWorkspaces(legacy);
          clearLegacyWorkspaces();
        } catch (e) {
          // 推送失败：保留内存集合（本次会话可用），不阻断，仅记录错误
          error.value = e instanceof ApiError ? e.message : String(e);
        }
        return legacy[0];
      }

      // 3) 前后端都无集合
      workspaces.value = [];
      return null;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
      workspaces.value = [];
      return null;
    } finally {
      restoring.value = false;
    }
  }

  /** 添加一个工作区（去重），更新内存并持久化到后端 */
  async function addWorkspace(path: string) {
    const p = path.trim();
    if (!p) return;
    if (!workspaces.value.includes(p)) {
      workspaces.value.push(p);
      await persistAll();
    }
  }

  /** 移除一个工作区，更新内存并持久化到后端 */
  async function removeWorkspace(path: string) {
    workspaces.value = workspaces.value.filter((w) => w !== path);
    await persistAll();
  }

  /** 整表替换工作区集合，更新内存并持久化到后端 */
  async function setWorkspaces(list: string[]) {
    workspaces.value = [...new Set(list.map((s) => s.trim()).filter(Boolean))];
    await persistAll();
  }

  /** 把当前内存集合整表持久化到后端（setWorkspaces 整表替换 + 镜像 workspace_path） */
  async function persistAll() {
    try {
      await persistWorkspaces(workspaces.value);
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
    }
  }

  /** 选择工作区：选择成功后加入集合并持久化（后端） */
  async function selectWorkspacePath(): Promise<boolean> {
    selecting.value = true;
    error.value = null;
    try {
      const picked = await selectWorkspace();
      if (picked) {
        await addWorkspace(picked);
        return true;
      }
      return false;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
      return false;
    } finally {
      selecting.value = false;
    }
  }

  /** 清除所有工作区（路径失效时降级回 Welcome 用），并持久化到后端 */
  async function invalidateWorkspace() {
    workspaces.value = [];
    error.value = null;
    await persistAll();
  }

  /**
   * 登出：重置本地状态（清空 settings.json 全部工作区 / 编辑器 / 偏好 / 缓存，保留数据库），
   * 并同步清空前端内存态（settings + editor store），避免残留。
   * 返回是否成功；失败时由调用方展示错误。
   */
  async function logout(): Promise<boolean> {
    error.value = null;
    try {
      await resetAppState();
      // 后端已清空 settings.json，同步清内存态避免残留
      workspaces.value = [];
      selecting.value = false;
      restoring.value = false;
      useEditorStore().reset();
      return true;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
      return false;
    }
  }

  /** 兼容：把 path 设为集合中唯一工作区 */
  function setWorkspace(path: string) {
    void setWorkspaces([path]);
  }

  return {
    workspaces,
    workspacePath,
    selecting,
    restoring,
    error,
    restore,
    addWorkspace,
    removeWorkspace,
    setWorkspaces,
    selectWorkspacePath,
    invalidateWorkspace,
    logout,
    setWorkspace,
  };
});
