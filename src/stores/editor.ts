import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { AvailableEditor, InstalledAppInfo } from "@/types";
import {
  listEditors,
  rescanEditors,
  listAppCandidates,
  confirmCustomEditor,
  listInstalledApps as apiListInstalledApps,
  importCustomApp as apiImportCustomApp,
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

  /**
   * 手动导入候选列表（v0.3）：仅 `cli` + `open_a` 的可打开编辑器，合并已确认的自定义项。
   * 与 `editors`（自动列表）区分：此列表用于 Settings「手动导入应用」与 Welcome 引导。
   */
  const candidates = ref<AvailableEditor[]>([]);
  const candidatesLoading = ref(false);

  /** 加载手动导入候选列表 */
  async function loadCandidates() {
    candidatesLoading.value = true;
    try {
      candidates.value = await listAppCandidates();
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
    } finally {
      candidatesLoading.value = false;
    }
  }

  /**
   * 确认导入自定义编辑器（持久化用户确认到 `custom_editors`）。
   * 返回是否成功；失败时在调用方展示错误。
   */
  async function confirmCustom(editorId: string): Promise<boolean> {
    try {
      await confirmCustomEditor(editorId);
      // 记录为手动导入项（用于「手动加入」标识）
      customEditorIds.value = new Set(customEditorIds.value).add(editorId);
      // 确认后同步刷新 editors（Settings 下拉框数据源）与候选列表，
      // 确保 Welcome 引导导入的编辑器即时出现在下拉框中（与 importApp 一致）。
      await Promise.all([loadEditors(), loadCandidates()]);
      return true;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
      return false;
    }
  }

  /**
   * 已安装应用列表（V03-MANUAL-IMPORT）：`/Applications` + `~/Applications` 下全部 `.app`
   * （含图标 base64），供 Settings 手动导入面板展示与点选。
   */
  const installedApps = ref<InstalledAppInfo[]>([]);
  const installedAppsLoading = ref(false);
  /** 正在导入的 app path（按钮 loading） */
  const importingApp = ref<string | null>(null);
  /** 本会话已手动导入（custom）的编辑器 id 集合，用于「手动加入」标识 */
  const customEditorIds = ref<Set<string>>(new Set());
  /** 本会话已导入的 app 路径集合，用于手动导入面板标记「已导入」 */
  const importedAppPaths = ref<Set<string>>(new Set());

  /**
   * 从 `editors`（含后端合并的 custom 项）推导的「已导入 app path」集合。
   * 后端 `list_editors` 已合并 custom_editors，故重启后仍含历史手动导入项；
   * 用此集合判断已导入可跨会话生效（比仅本会话的 importedAppPaths 更可靠）。
   */
  const importedAppPathsFromEditors = computed<Set<string>>(
    () =>
      new Set(
        editors.value
          .map((e) => e.app_path)
          .filter((p): p is string => !!p)
      )
  );

  /**
   * 模块级缓存：`/Applications` + `~/Applications` 全量 app（含 icon）扫描较慢，
   * 懒加载只在首次展开面板时拉一次，内存缓存复用，避免每次打开都重新遍历 + 取 icon。
   */
  let installedAppsCache: InstalledAppInfo[] | null = null;

  /**
   * 加载全部已安装应用（带内存缓存 + 懒加载）。
   * - 非强制：有缓存直接复用，不重新调后端。
   * - `force = true`：强制重新拉取（手动「刷新」按钮）。
   */
  async function loadInstalledApps(force = false) {
    if (!force && installedAppsCache) {
      installedApps.value = installedAppsCache;
      return;
    }
    installedAppsLoading.value = true;
    try {
      const list = await apiListInstalledApps();
      installedAppsCache = list;
      installedApps.value = list;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
    } finally {
      installedAppsLoading.value = false;
    }
  }

  /**
   * 判断某编辑器 id 是否为手动导入项。
   * 主判定：在 `editors` 中查找该 id 的 `source === "custom"`（后端已可靠标记，
   * 跨会话 / 重启后仍生效，不再依赖易失的内存 customEditorIds）。
   * 兜底：本会话刚导入但 editors 尚未刷新时，用 customEditorIds 内存 Set。
   */
  function isCustomEditor(editorId: string): boolean {
    const found = editors.value.find((e) => e.id === editorId);
    if (found) return found.source === "custom";
    return customEditorIds.value.has(editorId);
  }

  /**
   * 判断某 app 路径是否已导入（手动导入面板「已导入」标记用）。
   * 优先用 `editors`（含历史 custom，跨会话可靠）；本会话刚导入但未刷新时兜底 importedAppPaths。
   */
  function isAppImported(appPath: string): boolean {
    return (
      importedAppPathsFromEditors.value.has(appPath) ||
      importedAppPaths.value.has(appPath)
    );
  }

  /**
   * 导入自定义应用为编辑器（写入 custom_editors，幂等）。
   * 成功返回导入的编辑器；失败返回 null 并在调用方提示。
   */
  async function importApp(path: string): Promise<AvailableEditor | null> {
    importingApp.value = path;
    try {
      const editor = await apiImportCustomApp(path);
      // 记录为手动导入项（用于「手动加入」标识 + 面板「已导入」标记）
      customEditorIds.value = new Set(customEditorIds.value).add(editor.id);
      if (editor.app_path) {
        importedAppPaths.value = new Set(importedAppPaths.value).add(editor.app_path);
      }
      // 导入成功后刷新编辑器列表（新增 custom 项）
      await Promise.all([loadEditors(), loadCandidates()]);
      return editor;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
      return null;
    } finally {
      importingApp.value = null;
    }
  }

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

  /** 清空编辑器相关内存状态（登出 / reset 时同步后端已清的 settings）。 */
  function reset() {
    editors.value = [];
    candidates.value = [];
    defaultEditorId.value = null;
    loaded.value = false;
    error.value = null;
    opening.value = null;
    customEditorIds.value = new Set();
    installedApps.value = [];
    installedAppsLoading.value = false;
    importedAppPaths.value = new Set();
    importingApp.value = null;
    installedAppsCache = null;
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

  /** 静默设置默认编辑器偏好（无 toast，供 Welcome 引导用）；返回是否成功 */
  async function setDefaultSilent(editorId: string): Promise<boolean> {
    try {
      await apiSetEditorPreference(editorId);
      defaultEditorId.value = editorId;
      return true;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : String(e);
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
    candidates,
    candidatesLoading,
    installedApps,
    installedAppsLoading,
    importingApp,
    customEditorIds,
    importedAppPaths,
    init,
    loadEditors,
    rescan,
    loadCandidates,
    confirmCustom,
    loadInstalledApps,
    importApp,
    reset,
    isCustomEditor,
    isAppImported,
    loadPreference,
    setDefault,
    setDefaultSilent,
    defaultEditorAvailable,
    resolveDefaultEditorId,
    openEditor,
    openFileManager,
  };
});
