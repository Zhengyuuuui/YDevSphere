import { defineStore } from "pinia";
import { ref } from "vue";
import type { AvailableEditor } from "@/types";
import {
  listEditors,
  rescanEditors,
  openInEditor as apiOpenInEditor,
  openInFileManager as apiOpenInFileManager,
  getEditorPreference as apiGetEditorPreference,
  setEditorPreference as apiSetEditorPreference,
} from "@/api/editor";
import { ApiError } from "@/api/project";
import { toast } from "@/lib/toast";
import { t } from "@/lib/i18n";

/**
 * 编辑器状态：可用编辑器列表 / 默认偏好 / 打开动作（含降级）。
 *
 * 降级策略：打开编辑器失败 → toast 提示 + 自动降级到文件管理器。
 */
export const useEditorStore = defineStore("editor", () => {
  const editors = ref<AvailableEditor[]>([]);
  const defaultEditorId = ref<string | null>(null);
  const loaded = ref(false);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const opening = ref<{ projectId: number | null; editorId: string | null } | null>(null);

  /** 加载可用编辑器列表 */
  async function loadEditors() {
    loading.value = true;
    error.value = null;
    try {
      editors.value = await listEditors();
      loaded.value = true;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  /** 重新扫描编辑器（清缓存 → 扫描 → 刷新列表） */
  async function rescan() {
    loading.value = true;
    error.value = null;
    try {
      editors.value = await rescanEditors();
      loaded.value = true;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  /** 加载默认编辑器偏好 */
  async function loadPreference() {
    try {
      defaultEditorId.value = await apiGetEditorPreference();
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
    }
  }

  /** 初始化：加载编辑器列表 + 偏好 */
  async function init() {
    if (loaded.value) return;
    await loadEditors();
    await loadPreference();
  }

  /** 设置默认编辑器偏好（持久化） */
  async function setDefault(editorId: string) {
    try {
      await apiSetEditorPreference(editorId);
      defaultEditorId.value = editorId;
      toast.success(t("settings.editorSaved"));
      return true;
    } catch (e) {
      toast.error(
        t("settings.saveFailed", { msg: e instanceof ApiError ? e.message : String(e) })
      );
      return false;
    }
  }

  /** 默认编辑器是否可用（在检测列表中） */
  function defaultEditorAvailable(): boolean {
    if (!defaultEditorId.value) return false;
    return editors.value.some((e) => e.id === defaultEditorId.value);
  }

  /** 解析默认编辑器 id；无可用的返回 null */
  function resolveDefaultEditorId(): string | null {
    if (defaultEditorId.value && defaultEditorAvailable()) {
      return defaultEditorId.value;
    }
    // 无默认偏好时，回退到第一个可用编辑器
    return editors.value[0]?.id ?? null;
  }

  /** 用指定编辑器打开项目；失败自动降级到文件管理器 */
  async function openEditor(projectId: number, editorId: string | null) {
    const targetEditorId = editorId ?? resolveDefaultEditorId();
    if (!targetEditorId) {
      // 无任何可用编辑器 → 直接降级到文件管理器
      return openFileManager(projectId);
    }
    opening.value = { projectId, editorId: targetEditorId };
    try {
      await apiOpenInEditor(projectId, targetEditorId);
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : String(e);
      toast.error(t("settings.openEditorFailed", { msg }));
      await openFileManager(projectId);
    } finally {
      opening.value = null;
    }
  }

  /** 用文件管理器打开项目目录 */
  async function openFileManager(projectId: number) {
    opening.value = { projectId, editorId: null };
    try {
      await apiOpenInFileManager(projectId);
    } catch (e) {
      toast.error(
        t("settings.openFileManagerFailed", { msg: e instanceof ApiError ? e.message : String(e) })
      );
    } finally {
      opening.value = null;
    }
  }

  return {
    editors,
    defaultEditorId,
    loaded,
    loading,
    error,
    opening,
    init,
    loadEditors,
    rescan,
    loadPreference,
    setDefault,
    defaultEditorAvailable,
    resolveDefaultEditorId,
    openEditor,
    openFileManager,
  };
});
