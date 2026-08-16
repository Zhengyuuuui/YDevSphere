<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/stores/settings";
import { useEditorStore } from "@/stores/editor";
import { useI18nStore } from "@/stores/i18n";
import { DATABASE_LOCATION_DESC } from "@/lib/constants";
import { getIgnoreRules, setIgnoreRules } from "@/api/ignoreRules";
import { toast } from "@/lib/toast";

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
const { t } = useI18n();

const activeSection = ref<Section>("general");

const SECTIONS = computed<{ key: Section; label: string }[]>(() => [
  { key: "general", label: t("settings.general") },
  { key: "workspace", label: t("settings.workspace") },
  { key: "editor", label: t("settings.editor") },
  { key: "ignore", label: t("settings.ignore") },
  { key: "privacy", label: t("settings.privacy") },
  { key: "database", label: t("settings.database") },
  { key: "about", label: t("settings.about") },
]);

onMounted(() => editorStore.init());

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
</script>

<template>
  <div class="min-h-full bg-[#F7F8FA]">
    <div class="mx-auto max-w-[1140px] px-8 py-7">
      <div class="mb-6">
        <h1 class="text-[22px] font-semibold leading-tight tracking-tight text-[#17191C]">
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
              class="flex w-full items-center rounded-[6px] px-3 py-[7px] text-left text-[13px] transition-colors"
              :class="
                activeSection === s.key
                  ? 'bg-[#EEF2FF] font-medium text-[#2563EB]'
                  : 'text-[#6B7280] hover:bg-[#F3F4F6] hover:text-[#374151]'
              "
              @click="onSelectSection(s.key)"
            >
              {{ s.label }}
            </button>
          </nav>
        </div>

        <!-- 设置内容 -->
        <div class="flex-1 rounded-[8px] border border-[#E5E7EB] bg-white px-6 py-5">
          <!-- General -->
          <template v-if="activeSection === 'general'">
            <h2 class="mb-4 text-[14px] font-semibold text-[#17191C]">{{ t("settings.general") }}</h2>
            <div class="py-3.5">
              <div class="text-[13px] font-medium text-[#17191C]">{{ t("settings.language") }}</div>
              <div class="mt-1">
                <select
                  class="w-full max-w-sm rounded-lg border border-[#E5E7EB] bg-white px-3 py-2 text-[13px] focus:border-[#2563EB] focus:outline-none"
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
            <div class="border-b border-[#F3F4F6]" />
            <div class="py-3.5">
              <div class="text-[13px] font-medium text-[#17191C]">{{ t("settings.appName") }}</div>
              <div class="mt-0.5 text-[12px] text-[#9CA3AF]">YDevSphere</div>
            </div>
            <div class="border-b border-[#F3F4F6]" />
            <div class="py-3.5">
              <div class="text-[13px] font-medium text-[#17191C]">{{ t("settings.appDesc") }}</div>
              <div class="mt-0.5 text-[12px] text-[#9CA3AF]">
                {{ t("settings.appDescText") }}
              </div>
            </div>
          </template>

          <!-- Workspace -->
          <template v-else-if="activeSection === 'workspace'">
            <h2 class="mb-4 text-[14px] font-semibold text-[#17191C]">{{ t("settings.workspace") }}</h2>
            <p class="text-[13px] text-[#9CA3AF]">
              {{ t("settings.workspaceDesc") }}
            </p>

            <button
              class="mt-4 rounded-[7px] bg-[#2563EB] px-4 py-2 text-[13px] font-medium text-white transition-colors hover:bg-[#1D4ED8] disabled:opacity-60"
              :disabled="settings.selecting"
              @click="chooseWorkspace"
            >
              {{ settings.selecting ? t("workspace.selecting") : t("workspace.selectDir") }}
            </button>

            <div class="mt-4 rounded-lg border border-[#F3F4F6] bg-[#FAFAFA] p-4">
              <div class="text-[12px] text-[#9CA3AF]">{{ t("workspace.added", { count: settings.workspaces.length }) }}</div>
              <div v-if="settings.workspaces.length === 0" class="mt-1 text-[13px] text-[#17191C]">
                {{ t("workspace.noneAdded") }}
              </div>
              <div v-else class="mt-2 space-y-1.5">
                <div
                  v-for="ws in settings.workspaces"
                  :key="ws"
                  class="flex items-center justify-between gap-2 rounded-md border border-[#E5E7EB] bg-white px-3 py-2"
                >
                  <span class="min-w-0 break-all text-[13px] text-[#17191C]">{{ ws }}</span>
                  <button
                    class="shrink-0 text-[#9CA3AF] transition-colors hover:text-[#DC2626]"
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

            <p v-if="settings.error" class="mt-4 text-[13px] text-[#DC2626]">
              {{ t("workspace.operationFailed", { msg: settings.error }) }}
            </p>
          </template>

          <!-- Editor -->
          <template v-else-if="activeSection === 'editor'">
            <h2 class="mb-4 text-[14px] font-semibold text-[#17191C]">{{ t("settings.editor") }}</h2>
            <p class="text-[13px] text-[#9CA3AF]">
              {{ t("settings.editorDesc") }}
            </p>

            <div v-if="editorStore.loading" class="mt-4 text-[13px] text-[#9CA3AF]">
              {{ t("settings.detectingEditors") }}
            </div>

            <div v-else-if="editorStore.editors.length > 0" class="mt-4">
              <div class="flex items-center gap-2">
                <select
                  class="w-full max-w-sm rounded-lg border border-[#E5E7EB] bg-white px-3 py-2 text-[13px] focus:border-[#2563EB] focus:outline-none"
                  :value="editorStore.defaultEditorId ?? editorStore.editors[0]?.id ?? ''"
                  @change="onEditorChange"
                >
                  <option
                    v-for="ed in editorStore.editors"
                    :key="ed.id"
                    :value="ed.id"
                    :disabled="ed.open_method === 'unsupported'"
                  >
                    {{ ed.name }}{{ ed.source === "discovered" ? `（${t("editor.autoDiscovered")}）` : "" }}{{ ed.open_method === "unsupported" ? `（${t("editor.selectDirManually")}）` : "" }}
                  </option>
                </select>
                <button
                  class="shrink-0 rounded-[7px] border border-[#E5E7EB] bg-white px-3 py-2 text-[13px] text-[#374151] transition-colors hover:bg-[#F9FAFB] disabled:opacity-60"
                  :disabled="editorStore.loading"
                  @click="editorStore.rescan()"
                >
                  {{ t("editor.rescan") }}
                </button>
              </div>
              <p v-if="!editorStore.defaultEditorId" class="mt-2 text-[12px] text-[#9CA3AF]">
                {{ t("settings.noDefaultEditor") }}
              </p>
            </div>

            <div
              v-else
              class="mt-4 rounded-lg border border-[#FDE68A] bg-[#FFFBEB] px-4 py-3 text-[13px] text-[#92400E]"
            >
              {{ t("settings.noEditorDetected") }}
            </div>

            <p v-if="editorStore.error" class="mt-4 text-[13px] text-[#DC2626]">
              {{ t("settings.editorDetectFailed", { msg: editorStore.error }) }}
            </p>
          </template>

          <!-- Ignore Rules -->
          <template v-else-if="activeSection === 'ignore'">
            <h2 class="mb-4 text-[14px] font-semibold text-[#17191C]">{{ t("settings.ignore") }}</h2>
            <p class="text-[13px] text-[#9CA3AF]">
              {{ t("settings.ignoreDesc") }}
            </p>

            <!-- 预设规则（只读说明） -->
            <div class="mt-4">
              <div class="text-[12px] font-medium text-[#6B7280]">{{ t("settings.presetIgnore") }}</div>
              <div class="mt-2 flex flex-wrap gap-1.5">
                <span
                  v-for="d in PRESET_IGNORE"
                  :key="d"
                  class="inline-flex items-center rounded-[4px] bg-[#F3F4F6] px-[8px] py-[3px] text-[12px] text-[#6B7280]"
                >
                  {{ d }}
                </span>
              </div>
            </div>

            <div class="my-5 border-b border-[#F3F4F6]" />

            <!-- 自定义规则 -->
            <div class="text-[12px] font-medium text-[#6B7280]">{{ t("settings.customIgnore") }}</div>
            <div v-if="ignoreLoading" class="mt-2 text-[13px] text-[#9CA3AF]">
              {{ t("settings.ignoreLoading") }}
            </div>
            <template v-else>
              <div class="mt-2 flex flex-wrap gap-1.5">
                <span
                  v-for="d in customIgnore"
                  :key="d"
                  class="inline-flex items-center gap-1 rounded-[4px] bg-[#EEF2FF] px-[8px] py-[3px] text-[12px] text-[#4338CA]"
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
                <span v-if="customIgnore.length === 0" class="text-[12px] text-[#9CA3AF]">
                  {{ t("settings.noCustomIgnore") }}
                </span>
              </div>

              <div class="mt-4 flex gap-2">
                <input
                  v-model="newIgnoreDir"
                  type="text"
                  :placeholder="t('settings.ignorePlaceholder')"
                  class="h-[36px] w-[280px] rounded-[7px] border border-[#E5E7EB] bg-white px-3 text-[13px] text-[#17191C] placeholder:text-[#B0B7C3] focus:border-[#2563EB] focus:outline-none"
                  @keyup.enter="addIgnoreDir"
                />
                <button
                  class="rounded-[7px] bg-[#2563EB] px-4 py-2 text-[13px] font-medium text-white transition-colors hover:bg-[#1D4ED8] disabled:opacity-60"
                  :disabled="!newIgnoreDir.trim()"
                  @click="addIgnoreDir"
                >
                  {{ t("settings.add") }}
                </button>
              </div>

              <p class="mt-3 text-[12px] text-[#9CA3AF]">
                {{ t("settings.ignoreTip") }}
              </p>
              <p v-if="ignoreError" class="mt-2 text-[13px] text-[#DC2626]">
                {{ ignoreError }}
              </p>
            </template>
          </template>

          <!-- Privacy -->
          <template v-else-if="activeSection === 'privacy'">
            <h2 class="mb-4 text-[14px] font-semibold text-[#17191C]">{{ t("settings.privacy") }}</h2>
            <div class="py-3.5">
              <div class="text-[13px] font-medium text-[#17191C]">{{ t("settings.privacyDataStorage") }}</div>
              <div class="mt-0.5 text-[12px] text-[#9CA3AF]">
                {{ t("settings.privacyDataStorageDesc") }}
              </div>
            </div>
            <div class="border-b border-[#F3F4F6]" />
            <div class="py-3.5">
              <div class="text-[13px] font-medium text-[#17191C]">{{ t("settings.privacyMemory") }}</div>
              <div class="mt-0.5 text-[12px] text-[#9CA3AF]">
                {{ t("settings.privacyMemoryDesc") }}
              </div>
            </div>
          </template>

          <!-- Database -->
          <template v-else-if="activeSection === 'database'">
            <h2 class="mb-4 text-[14px] font-semibold text-[#17191C]">{{ t("settings.database") }}</h2>
            <p class="text-[13px] text-[#9CA3AF]">
              {{ t("settings.dbDesc") }}
            </p>
            <div class="mt-4 rounded-lg border border-[#F3F4F6] bg-[#FAFAFA] p-4">
              <div class="text-[12px] text-[#9CA3AF]">{{ t("settings.dbLocation") }}</div>
              <div class="mt-1 text-[13px] text-[#17191C]">{{ DATABASE_LOCATION_DESC }}</div>
            </div>
          </template>

          <!-- About -->
          <template v-else-if="activeSection === 'about'">
            <h2 class="mb-4 text-[14px] font-semibold text-[#17191C]">{{ t("settings.about") }}</h2>
            <div class="py-3.5">
              <div class="text-[13px] font-medium text-[#17191C]">{{ t("settings.version") }}</div>
              <div class="mt-0.5 text-[12px] text-[#9CA3AF]">v0.2.0</div>
            </div>
            <div class="border-b border-[#F3F4F6]" />
            <div class="py-3.5">
              <div class="text-[13px] font-medium text-[#17191C]">{{ t("settings.techStack") }}</div>
              <div class="mt-0.5 text-[12px] text-[#9CA3AF]">
                Vue 3 · TypeScript · Tauri 2 · Rust
              </div>
            </div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>
