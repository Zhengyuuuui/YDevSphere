<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/stores/settings";
import { useEditorStore } from "@/stores/editor";
import { useI18nStore } from "@/stores/i18n";
import { useThemeStore, type ThemeMode } from "@/stores/theme";
import { DATABASE_LOCATION_DESC } from "@/lib/constants";
import { getIgnoreRules, setIgnoreRules } from "@/api/ignoreRules";
import type { InstalledAppInfo } from "@/types";
import { toast } from "@/lib/toast";
import { useRouter } from "vue-router";
import { iconSrc as editorIconSrc } from "@/lib/editorIcon";

type Section =
  | "general"
  | "workspace"
  | "editor"
  | "privacy"
  | "database"
  | "ignore"
  | "about";

const settings = useSettingsStore();
const editorStore = useEditorStore();
const i18nStore = useI18nStore();
const themeStore = useThemeStore();
const router = useRouter();
const { t } = useI18n();

/** 主题三态选项 */
const themeOptions: { value: ThemeMode; label: string }[] = [
  { value: "light", label: t("settings.themeLight") },
  { value: "dark", label: t("settings.themeDark") },
  { value: "system", label: t("settings.themeSystem") },
];

function onThemeChange(event: Event) {
  const v = (event.target as HTMLSelectElement).value as ThemeMode;
  themeStore.setMode(v);
}

const activeSection = ref<Section>("general");
/** 登出二次确认框是否打开 / 是否执行中 */
const logoutConfirmOpen = ref(false);
const logoutBusy = ref(false);
/** 手动导入面板是否展开 */
const importPanelOpen = ref(false);
/** 手动导入面板内按名称搜索过滤 */
const installedSearch = ref("");

/** 按名称过滤已安装应用 */
const filteredInstalledApps = computed(() => {
  const kw = installedSearch.value.trim().toLowerCase();
  if (!kw) return editorStore.installedApps;
  return editorStore.installedApps.filter((a) =>
    a.name.toLowerCase().includes(kw) || (a.bundle_id ?? "").toLowerCase().includes(kw)
  );
});

/** 打开手动导入面板时加载全部已安装应用 */
async function toggleImportPanel() {
  importPanelOpen.value = !importPanelOpen.value;
  if (importPanelOpen.value) {
    installedSearch.value = "";
    await editorStore.loadInstalledApps();
  }
}

/** 点选某个已安装应用 → 导入为自定义编辑器 */
async function importInstalledApp(app: InstalledAppInfo) {
  const editor = await editorStore.importApp(app.path);
  if (editor) {
    toast.success(t("editor.imported", { name: editor.name }));
  } else {
    toast.error(t("editor.importFailed", { msg: editorStore.error }));
  }
}

/** 图标 base64 → data URL（无图标时返回 undefined，走占位） */
function iconSrc(app: InstalledAppInfo): string | undefined {
  return editorIconSrc(app.icon_base64);
}

const SECTIONS = computed<{ key: Section; label: string }[]>(() => [
  { key: "general", label: t("settings.general") },
  { key: "workspace", label: t("settings.workspace") },
  { key: "editor", label: t("settings.editor") },
  { key: "ignore", label: t("settings.ignore") },
  { key: "privacy", label: t("settings.privacy") },
  { key: "database", label: t("settings.database") },
  { key: "about", label: t("settings.about") },
]);

onMounted(() => {
  editorStore.init();
});

async function chooseWorkspace() {
  await settings.selectWorkspacePath();
}

async function onEditorChange(event: Event) {
  const id = (event.target as HTMLSelectElement).value;
  if (!id) return;
  await editorStore.setDefault(id);
}

/* ------------------------- 语言切换 ------------------------- */

/** 语言切换选项 */
const languageOptions: { value: "zh-CN" | "en-US"; label: string }[] = [
  { value: "zh-CN", label: "中文" },
  { value: "en-US", label: "English" },
];

async function onLanguageChange(event: Event) {
  const lng = (event.target as HTMLSelectElement).value;
  if (lng !== "zh-CN" && lng !== "en-US") return;
  await i18nStore.setLocale(lng);
}

/* ------------------------- 忽略规则（v0.2） ------------------------- */

/** 预设忽略目录（只读说明） */
const PRESET_IGNORE = computed(() => [
  "node_modules",
  ".git",
  "target",
  "dist",
  "build",
  "vendor",
  ".cache",
  t("settings.hiddenDirs"),
]);

/** 用户自定义忽略目录名列表 */
const customIgnore = ref<string[]>([]);
const ignoreLoading = ref(false);
const newIgnoreDir = ref("");
const ignoreError = ref<string | null>(null);

async function loadIgnoreRules() {
  ignoreLoading.value = true;
  ignoreError.value = null;
  try {
    customIgnore.value = await getIgnoreRules();
  } catch (e) {
    ignoreError.value = e instanceof Error ? e.message : String(e);
  } finally {
    ignoreLoading.value = false;
  }
}

async function addIgnoreDir() {
  const dir = newIgnoreDir.value.trim();
  if (!dir) return;
  if (customIgnore.value.includes(dir)) {
    newIgnoreDir.value = "";
    return;
  }
  const next = [...customIgnore.value, dir];
  ignoreError.value = null;
  try {
    await setIgnoreRules(next);
    customIgnore.value = next;
    newIgnoreDir.value = "";
    toast.success(t("settings.ignoreAdded"));
  } catch (e) {
    ignoreError.value = e instanceof Error ? e.message : String(e);
  }
}

async function removeIgnoreDir(dir: string) {
  const next = customIgnore.value.filter((d) => d !== dir);
  ignoreError.value = null;
  try {
    await setIgnoreRules(next);
    customIgnore.value = next;
    toast.success(t("settings.ignoreRemoved"));
  } catch (e) {
    ignoreError.value = e instanceof Error ? e.message : String(e);
  }
}

/** 进入忽略规则分区时加载 */
function onSelectSection(key: Section) {
  activeSection.value = key;
  if (key === "ignore") {
    loadIgnoreRules();
  }
}

/* ------------------------- 登出（重置本地状态） ------------------------- */

/** 打开登出二次确认框 */
function openLogoutConfirm() {
  logoutConfirmOpen.value = true;
}

/** 取消登出 */
function cancelLogout() {
  logoutConfirmOpen.value = false;
}

/** 确认登出：调后端 reset → 清内存态 → 跳回 Welcome */
async function confirmLogout() {
  logoutBusy.value = true;
  try {
    const ok = await settings.logout();
    if (ok) {
      logoutConfirmOpen.value = false;
      toast.success(t("settings.loggedOut"));
      router.replace("/");
    } else {
      toast.error(
        t("settings.logoutFailed", {
          msg: settings.error ?? t("scan.unknownError"),
        })
      );
    }
  } finally {
    logoutBusy.value = false;
  }
}
</script>

<template>
  <div class="min-h-full bg-canvas">
    <div class="mx-auto max-w-[1140px] px-8 py-7">
      <div class="mb-6">
        <h1 class="text-[22px] font-semibold leading-tight tracking-tight text-ink">
          {{ t("settings.title") }}
        </h1>
      </div>

      <div class="flex gap-5">
        <!-- 设置分区导航 -->
        <div class="w-[172px] shrink-0">
          <nav class="space-y-0.5">
            <button
              v-for="s in SECTIONS"
              :key="s.key"
              class="font-display flex w-full items-center rounded-[6px] px-3 py-[7px] text-left text-[14px] transition-colors"
              :class="
                activeSection === s.key
                  ? 'bg-primary-soft font-medium text-primary'
                  : 'text-muted hover:bg-surface-2 hover:text-ink'
              "
              @click="onSelectSection(s.key)"
            >
              {{ s.label }}
            </button>
          </nav>
        </div>

        <!-- 设置内容 -->
        <div class="flex-1 rounded-[8px] border border-line-3 bg-surface px-6 py-5">
          <!-- General -->
          <template v-if="activeSection === 'general'">
            <h2 class="mb-4 text-[15px] font-semibold text-ink">{{ t("settings.general") }}</h2>
            <div class="py-3.5">
              <div class="font-display text-[14px] font-medium text-ink">{{ t("settings.language") }}</div>
              <div class="mt-1">
                <select
                  class="w-full max-w-sm rounded-lg border border-line bg-surface px-3 py-2 text-[14px] focus:border-primary focus:outline-none"
                  :value="i18nStore.locale"
                  @change="onLanguageChange"
                >
                  <option
                    v-for="opt in languageOptions"
                    :key="opt.value"
                    :value="opt.value"
                  >
                    {{ opt.label }}
                  </option>
                </select>
              </div>
            </div>
            <div class="border-b border-line-2" />
            <div class="py-3.5">
              <div class="font-display text-[14px] font-medium text-ink">{{ t("settings.appearance") }}</div>
              <div class="mt-1">
                <select
                  class="w-full max-w-sm rounded-lg border border-line bg-surface px-3 py-2 text-[14px] text-ink focus:border-primary focus:outline-none"
                  :value="themeStore.mode"
                  @change="onThemeChange"
                >
                  <option
                    v-for="opt in themeOptions"
                    :key="opt.value"
                    :value="opt.value"
                  >
                    {{ opt.label }}
                  </option>
                </select>
                <p class="mt-1 text-[13px] text-faint">{{ t("settings.themeDesc") }}</p>
              </div>
            </div>
            <div class="border-b border-line-2" />
            <div class="py-3.5">
              <div class="font-display text-[14px] font-medium text-ink">{{ t("settings.appName") }}</div>
              <div class="mt-0.5 text-[13px] text-faint">YDevSphere</div>
            </div>
            <div class="border-b border-line-2" />
            <div class="py-3.5">
              <div class="font-display text-[14px] font-medium text-ink">{{ t("settings.appDesc") }}</div>
              <div class="mt-0.5 text-[13px] text-faint">
                {{ t("settings.appDescText") }}
              </div>
            </div>
          </template>

          <!-- Workspace -->
          <template v-else-if="activeSection === 'workspace'">
            <h2 class="mb-4 text-[15px] font-semibold text-ink">{{ t("settings.workspace") }}</h2>
            <p class="text-[14px] text-faint">
              {{ t("settings.workspaceDesc") }}
            </p>

            <button
              class="mt-4 rounded-[7px] bg-primary px-4 py-2 text-[14px] font-medium text-white transition-colors hover:bg-primary-hover disabled:opacity-60"
              :disabled="settings.selecting"
              @click="chooseWorkspace"
            >
              {{ settings.selecting ? t("workspace.selecting") : t("workspace.selectDir") }}
            </button>

            <div class="mt-4 rounded-lg border border-line-2 bg-surface-3 p-4">
              <div class="text-[13px] text-faint">{{ t("workspace.added", { count: settings.workspaces.length }) }}</div>
              <div v-if="settings.workspaces.length === 0" class="mt-1 text-[14px] text-ink">
                {{ t("workspace.noneAdded") }}
              </div>
              <div v-else class="mt-2 space-y-1.5">
                <div
                  v-for="ws in settings.workspaces"
                  :key="ws"
                  class="flex items-center justify-between gap-2 rounded-md border border-line bg-surface px-3 py-2"
                >
                  <span class="min-w-0 break-all text-[14px] text-ink">{{ ws }}</span>
                  <button
                    class="shrink-0 text-faint transition-colors hover:text-red-600 dark:hover:text-red-400"
                    :title="t('workspace.remove')"
                    @click="settings.removeWorkspace(ws)"
                  >
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M18 6 6 18" />
                      <path d="M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              </div>
            </div>

            <p v-if="settings.error" class="mt-4 text-[14px] text-red-600 dark:text-red-400">
              {{ t("workspace.operationFailed", { msg: settings.error }) }}
            </p>
          </template>

          <!-- Editor -->
          <template v-else-if="activeSection === 'editor'">
            <h2 class="mb-4 text-[15px] font-semibold text-ink">{{ t("settings.editor") }}</h2>
            <p class="text-[14px] text-faint">
              {{ t("settings.editorDesc") }}
            </p>

            <div v-if="editorStore.loading" class="mt-4 text-[14px] text-faint">
              {{ t("settings.detectingEditors") }}
            </div>

            <div v-else-if="editorStore.editors.length > 0" class="mt-4">
              <div class="flex items-center gap-2">
                <select
                  class="w-full max-w-sm rounded-lg border border-line bg-surface px-3 py-2 text-[14px] focus:border-primary focus:outline-none"
                  :value="editorStore.defaultEditorId ?? editorStore.editors[0]?.id ?? ''"
                  @change="onEditorChange"
                >
                  <option
                    v-for="ed in editorStore.editors"
                    :key="ed.id"
                    :value="ed.id"
                  >
                    {{ ed.name }}
                  </option>
                </select>
                <button
                  class="shrink-0 rounded-[7px] border border-line bg-surface px-3 py-2 text-[14px] text-ink transition-colors hover:bg-surface-3 disabled:opacity-60"
                  :disabled="editorStore.loading"
                  @click="editorStore.rescan()"
                >
                  {{ t("editor.rescan") }}
                </button>
              </div>
              <p v-if="!editorStore.defaultEditorId" class="mt-2 text-[13px] text-faint">
                {{ t("settings.noDefaultEditor") }}
              </p>
            </div>

            <div
              v-else
              class="mt-4 rounded-lg border border-[#FDE68A] bg-[#FFFBEB] px-4 py-3 text-[14px] text-[#92400E] dark:border-yellow-900 dark:bg-yellow-950 dark:text-yellow-400"
            >
              {{ t("settings.noEditorDetected") }}
            </div>

            <!-- 手动导入应用 -->
            <div class="mt-5 border-t border-line-2 pt-4">
              <button
                class="flex items-center gap-1.5 rounded-[7px] border border-line bg-surface px-3 py-2 text-[14px] text-ink transition-colors hover:bg-surface-3"
                @click="toggleImportPanel"
              >
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-faint">
                  <path d="M12 5v14" />
                  <path d="M5 12h14" />
                </svg>
                {{ t("editor.manualImport") }}
              </button>

              <div v-if="importPanelOpen" class="mt-3">
                <!-- 顶部提示 + 刷新 -->
                <div class="mb-2 flex items-center justify-between gap-2">
                  <p class="text-[11px] text-fainter">{{ t("editor.panelHint") }}</p>
                  <button
                    class="shrink-0 rounded-[5px] px-1.5 py-0.5 text-[11px] text-muted transition-colors hover:bg-surface-2 hover:text-ink disabled:opacity-50"
                    :disabled="editorStore.installedAppsLoading"
                    @click="editorStore.loadInstalledApps(true)"
                  >
                    {{ t("editor.refresh") }}
                  </button>
                </div>

                <!-- 搜索过滤 -->
                <input
                  v-model="installedSearch"
                  type="text"
                  :placeholder="t('editor.searchInstalled')"
                  class="mb-2 h-[34px] w-full rounded-[7px] border border-line bg-surface px-3 text-[14px] text-ink placeholder:text-fainter focus:border-primary focus:outline-none"
                />

                <!-- 首次加载动画（骨架屏） -->
                <div v-if="editorStore.installedAppsLoading" class="space-y-1.5">
                  <div
                    v-for="i in 6"
                    :key="i"
                    class="flex animate-pulse items-center gap-2.5 rounded-[5px] bg-surface-2 px-2 py-2"
                  >
                    <span class="h-[24px] w-[24px] rounded-[5px] bg-line" />
                    <span class="h-[12px] flex-1 rounded bg-line" />
                  </div>
                </div>

                <!-- 无应用 -->
                <div
                  v-else-if="editorStore.installedApps.length === 0"
                  class="py-4 text-center text-[14px] text-faint"
                >
                  {{ t("editor.noInstalledApps") }}
                </div>

                <!-- 全部已安装应用列表（图标 + 名称 + 标识） -->
                <div
                  v-else
                  class="max-h-[260px] space-y-1 overflow-y-auto rounded-[8px] border border-line-3 p-1.5"
                >
                  <div v-if="filteredInstalledApps.length === 0" class="py-4 text-center text-[14px] text-faint">
                    {{ t("editor.noInstalledApps") }}
                  </div>
                  <button
                    v-for="app in filteredInstalledApps"
                    :key="app.path"
                    class="flex w-full items-center gap-2.5 rounded-[5px] px-2 py-1.5 text-left transition-colors hover:bg-surface-3 disabled:opacity-60"
                    :disabled="editorStore.importingApp === app.path || editorStore.isAppImported(app.path)"
                    @click="importInstalledApp(app)"
                  >
                    <!-- 图标或占位 -->
                    <img
                      v-if="iconSrc(app)"
                      :src="iconSrc(app)"
                      alt=""
                      class="h-[24px] w-[24px] shrink-0 rounded-[5px] object-contain"
                    />
                    <span
                      v-else
                      class="flex h-[24px] w-[24px] shrink-0 items-center justify-center rounded-[5px] bg-surface-2 text-[11px] font-medium text-faint"
                    >
                      {{ app.name.slice(0, 1).toUpperCase() }}
                    </span>

                    <!-- 名称 + 标识 -->
                    <span class="min-w-0 flex-1">
                      <span class="block truncate text-[14px] text-ink">{{ app.name }}</span>
                      <span v-if="app.bundle_id" class="block truncate text-[11px] text-fainter">{{ app.bundle_id }}</span>
                    </span>

                    <!-- Fork 标识 -->
                    <span
                      v-if="app.has_product_json"
                      class="shrink-0 rounded-[3px] bg-primary-soft px-1 py-[1px] text-[10px] text-[#4338CA] dark:text-blue-300"
                    >
                      {{ t("editor.vscodeFork") }}
                    </span>

                    <!-- 已导入 / 导入中 / 导入 -->
                    <span
                      v-if="editorStore.isAppImported(app.path)"
                      class="shrink-0 text-[13px] text-faint"
                    >
                      {{ t("editor.alreadyImported") }}
                    </span>
                    <span
                      v-else-if="editorStore.importingApp === app.path"
                      class="shrink-0 text-[13px] text-faint"
                    >
                      {{ t("editor.importing") }}
                    </span>
                    <span v-else class="shrink-0 text-[13px] text-primary">{{ t("editor.pickApp") }}</span>
                  </button>
                </div>
              </div>
            </div>

            <p v-if="editorStore.error" class="mt-4 text-[14px] text-red-600 dark:text-red-400">
              {{ t("settings.editorDetectFailed", { msg: editorStore.error }) }}
            </p>
          </template>

          <!-- Ignore Rules -->
          <template v-else-if="activeSection === 'ignore'">
            <h2 class="mb-4 text-[15px] font-semibold text-ink">{{ t("settings.ignore") }}</h2>
            <p class="text-[14px] text-faint">
              {{ t("settings.ignoreDesc") }}
            </p>

            <!-- 预设规则（只读说明） -->
            <div class="mt-4">
              <div class="text-[13px] font-medium text-muted">{{ t("settings.presetIgnore") }}</div>
              <div class="mt-2 flex flex-wrap gap-1.5">
                <span
                  v-for="d in PRESET_IGNORE"
                  :key="d"
                  class="inline-flex items-center rounded-[4px] bg-surface-2 px-[8px] py-[3px] text-[13px] text-muted"
                >
                  {{ d }}
                </span>
              </div>
            </div>

            <div class="my-5 border-b border-line-2" />

            <!-- 自定义规则 -->
            <div class="text-[13px] font-medium text-muted">{{ t("settings.customIgnore") }}</div>
            <div v-if="ignoreLoading" class="mt-2 text-[14px] text-faint">
              {{ t("settings.ignoreLoading") }}
            </div>
            <template v-else>
              <div class="mt-2 flex flex-wrap gap-1.5">
                <span
                  v-for="d in customIgnore"
                  :key="d"
                  class="inline-flex items-center gap-1 rounded-[4px] bg-primary-soft px-[8px] py-[3px] text-[13px] text-[#4338CA]"
                >
                  {{ d }}
                  <button
                    class="text-[#4338CA]/60 transition-colors hover:text-[#4338CA]"
                    :title="t('settings.remove')"
                    @click="removeIgnoreDir(d)"
                  >
                    <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M18 6 6 18" />
                      <path d="M6 6l12 12" />
                    </svg>
                  </button>
                </span>
                <span v-if="customIgnore.length === 0" class="text-[13px] text-faint">
                  {{ t("settings.noCustomIgnore") }}
                </span>
              </div>

              <div class="mt-4 flex gap-2">
                <input
                  v-model="newIgnoreDir"
                  type="text"
                  :placeholder="t('settings.ignorePlaceholder')"
                  class="h-[36px] w-[280px] rounded-[7px] border border-line bg-surface px-3 text-[14px] text-ink placeholder:text-fainter focus:border-primary focus:outline-none"
                  @keyup.enter="addIgnoreDir"
                />
                <button
                  class="rounded-[7px] bg-primary px-4 py-2 text-[14px] font-medium text-white transition-colors hover:bg-primary-hover disabled:opacity-60"
                  :disabled="!newIgnoreDir.trim()"
                  @click="addIgnoreDir"
                >
                  {{ t("settings.add") }}
                </button>
              </div>

              <p class="mt-3 text-[13px] text-faint">
                {{ t("settings.ignoreTip") }}
              </p>
              <p v-if="ignoreError" class="mt-2 text-[14px] text-[#DC2626]">
                {{ ignoreError }}
              </p>
            </template>
          </template>

          <!-- Privacy -->
          <template v-else-if="activeSection === 'privacy'">
            <h2 class="mb-4 text-[15px] font-semibold text-ink">{{ t("settings.privacy") }}</h2>
            <div class="py-3.5">
              <div class="font-display text-[14px] font-medium text-ink">{{ t("settings.privacyDataStorage") }}</div>
              <div class="mt-0.5 text-[13px] text-faint">
                {{ t("settings.privacyDataStorageDesc") }}
              </div>
            </div>
            <div class="border-b border-line-2" />
            <div class="py-3.5">
              <div class="font-display text-[14px] font-medium text-ink">{{ t("settings.privacyMemory") }}</div>
              <div class="mt-0.5 text-[13px] text-faint">
                {{ t("settings.privacyMemoryDesc") }}
              </div>
            </div>
          </template>

          <!-- Database -->
          <template v-else-if="activeSection === 'database'">
            <h2 class="mb-4 text-[15px] font-semibold text-ink">{{ t("settings.database") }}</h2>
            <p class="text-[14px] text-faint">
              {{ t("settings.dbDesc") }}
            </p>
            <div class="mt-4 rounded-lg border border-line-2 bg-surface-3 p-4">
              <div class="text-[13px] text-faint">{{ t("settings.dbLocation") }}</div>
              <div class="mt-1 text-[14px] text-ink">{{ DATABASE_LOCATION_DESC }}</div>
            </div>
          </template>

          <!-- About -->
          <template v-else-if="activeSection === 'about'">
            <h2 class="mb-4 text-[15px] font-semibold text-ink">{{ t("settings.about") }}</h2>
            <div class="py-3.5">
              <div class="font-display text-[14px] font-medium text-ink">{{ t("settings.version") }}</div>
              <div class="mt-0.5 text-[13px] text-faint">v0.2.0</div>
            </div>
            <div class="border-b border-line-2" />
            <div class="py-3.5">
              <div class="font-display text-[14px] font-medium text-ink">{{ t("settings.techStack") }}</div>
              <div class="mt-0.5 text-[13px] text-faint">
                Vue 3 · TypeScript · Tauri 2 · Rust
              </div>
            </div>
            <div class="border-b border-line-2" />
            <div class="pt-4">
              <button
                class="rounded-[7px] border border-[#FCA5A5] px-4 py-2 text-[14px] font-medium text-[#DC2626] transition-colors hover:bg-[#FEF2F2]"
                @click="openLogoutConfirm"
              >
                {{ t("settings.logout") }}
              </button>
              <p class="mt-2 text-[13px] text-faint">{{ t("settings.logoutConfirm") }}</p>
            </div>
          </template>
        </div>
      </div>
    </div>

    <!-- 登出二次确认框 -->
    <div
      v-if="logoutConfirmOpen"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/30"
      @click.self="cancelLogout"
    >
      <div class="w-[400px] rounded-[10px] border border-line-3 bg-surface p-5 shadow-lg">
        <h3 class="text-[15px] font-semibold text-ink">{{ t("settings.logoutConfirmTitle") }}</h3>
        <p class="mt-2 text-[14px] text-muted">{{ t("settings.logoutConfirm") }}</p>
        <div class="mt-5 flex justify-end gap-2">
          <button
            class="rounded-[7px] border border-line px-4 py-2 text-[14px] text-ink hover:bg-surface-2"
            :disabled="logoutBusy"
            @click="cancelLogout"
          >
            {{ t("editor.cancel") }}
          </button>
          <button
            class="rounded-[7px] bg-red-600 px-4 py-2 text-[14px] font-medium text-white hover:bg-red-700 disabled:opacity-60"
            :disabled="logoutBusy"
            @click="confirmLogout"
          >
            {{ logoutBusy ? t("settings.loggingOut") : t("settings.logout") }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
