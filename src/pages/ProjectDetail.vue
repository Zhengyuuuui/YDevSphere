<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import TechnologyBadge from "@/components/TechnologyBadge.vue";
import DirTree from "@/components/DirTree.vue";
import { useProjectStore } from "@/stores/project";
import { useMemoryStore } from "@/stores/memory";
import { useGitStore } from "@/stores/git";
import { useEditorStore } from "@/stores/editor";
import { formatDateTime, formatGitStatus, isGitDirty } from "@/lib/format";
import { toast } from "@/lib/toast";
import { getProjects } from "@/api/project";
import type { Project } from "@/types";
import { stackTechnologies, techNameOf } from "@/lib/tech";

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const projectStore = useProjectStore();
const memoryStore = useMemoryStore();
const gitStore = useGitStore();
const editorStore = useEditorStore();

const projectId = computed(() => Number(route.params.id));

/** 项目类型文案 */
const kindLabel = computed(() => {
  switch (projectStore.detail?.kind) {
    case "aggregated_root":
      return t("detail.kindAggregatedRoot");
    case "category":
      return t("detail.kindCategory");
    default:
      return t("detail.kindReal");
  }
});

/** 健康度颜色分级（≥60 绿、40-59 黄、<40 灰），dark 下用更亮的色保证可读 */
const isDark = () =>
  typeof document !== "undefined" && document.documentElement.dataset.theme === "dark";
const healthColor = computed(() => {
  const s = projectStore.detail?.health_score ?? 0;
  if (s >= 60) return isDark() ? "#4ade80" : "#16A34A";
  if (s >= 40) return isDark() ? "#fbbf24" : "#D97706";
  return isDark() ? "#6b7280" : "#9CA3AF";
});

/** 编辑状态 */
const editing = ref(false);
const editPackageManager = ref("");
const editStackText = ref("");

/** 子项目列表（聚合根 / 分类目录详情的「前后端分区」基于 children 重算，Spec §7.2） */
const children = ref<Project[]>([]);
/** 子项目加载中 */
const childrenLoading = ref(false);
/** 子项目加载失败信息 */
const childrenError = ref("");

/** 是否聚合容器（聚合根 / 分类目录）——展示前后端分区 */
const isAggregate = computed(
  () =>
    projectStore.detail?.kind === "aggregated_root" ||
    projectStore.detail?.kind === "category"
);

/** 单项目（Real）直接展示自身技术栈（technologies 为空回退 language/framework） */
const ownStack = computed(() =>
  projectStore.detail ? stackTechnologies(projectStore.detail) : []
);

/** 前后端分区（Spec §7.2）：每个子项目一个分区，展示其 Source of Truth 技术栈 */
const sections = computed(() =>
  children.value.map((child) => ({
    id: child.id,
    name: child.name,
    path: child.path,
    technologies: stackTechnologies(child),
  }))
);

/** 加载聚合容器的直接子项目（用于前后端分区） */
async function loadChildren() {
  if (!isAggregate.value) {
    children.value = [];
    return;
  }
  childrenLoading.value = true;
  childrenError.value = "";
  try {
    children.value = await getProjects(undefined, undefined, undefined, projectId.value);
  } catch (e) {
    childrenError.value = e instanceof Error ? e.message : String(e);
  } finally {
    childrenLoading.value = false;
  }
}

function load() {
  projectStore.fetchProjectDetail(projectId.value);
  memoryStore.fetchMemory(projectId.value);
  gitStore.fetchGit(projectId.value);
  loadChildren();
}

function goBack() {
  router.back();
}

onMounted(() => editorStore.init());

/** 在默认编辑器打开（无可用编辑器时自动降级文件管理器） */
async function handleOpenEditor() {
  await editorStore.openEditor(projectId.value, null);
}

/** 在文件管理器打开 */
async function handleOpenFileManager() {
  await editorStore.openFileManager(projectId.value);
}

watch(projectId, (id) => {
  load();
  projectStore.markOpened(id);
});
onMounted(() => {
  load();
  projectStore.markOpened(projectId.value);
});

async function handleEnable() {
  const ok = await memoryStore.enable(projectId.value, null);
  if (ok) {
    toast.success(t("detail.enabledToast"));
  } else {
    toast.error(t("detail.enableFailed", { msg: memoryStore.error }));
  }
}

function startEdit() {
  editing.value = true;
  editPackageManager.value = memoryStore.memory?.packageManager ?? "";
  editStackText.value = memoryStore.memory?.stack.join(", ") ?? "";
}

async function handleUpdate() {
  const pm = editPackageManager.value.trim() || null;
  const stack =
    editStackText.value
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean) || null;
  const ok = await memoryStore.update(projectId.value, pm, stack);
  if (ok) {
    toast.success(t("detail.updatedToast"));
    editing.value = false;
  } else {
    toast.error(t("detail.updateFailed", { msg: memoryStore.error }));
  }
}

function cancelEdit() {
  editing.value = false;
}
</script>

<template>
  <div class="min-h-full bg-canvas">
    <main class="mx-auto max-w-4xl px-8 py-8">
      <!-- 返回按钮 -->
      <button class="text-sm text-primary hover:underline" @click="goBack">
        {{ t("detail.back") }}
      </button>

      <!-- 加载中 -->
      <div
        v-if="projectStore.detailLoading"
        class="mt-8 rounded-lg border border-line-2 bg-surface p-10 text-center text-sm text-muted shadow-sm"
      >
        {{ t("detail.loading") }}
      </div>

      <!-- 错误 -->
      <div
        v-else-if="projectStore.error && !projectStore.detail"
        class="mt-8 rounded-lg border border-red-200 bg-red-50 p-10 text-center text-sm text-red-700 dark:border-red-900 dark:bg-red-950 dark:text-red-400"
      >
        {{ projectStore.error }}
      </div>

      <!-- 项目不存在（后端返回 null） -->
      <div
        v-else-if="!projectStore.detail"
        class="mt-8 rounded-lg border border-dashed border-line p-10 text-center text-muted"
      >
        {{ t("detail.notFound") }}
        <div class="mt-4">
          <RouterLink to="/projects" class="text-sm text-primary hover:underline">
            {{ t("detail.backToList") }}
          </RouterLink>
        </div>
      </div>

      <!-- 项目详情 -->
      <div
        v-else
        class="mt-4 rounded-lg border border-line-3 bg-surface p-6 shadow-sm"
      >
        <div class="flex flex-wrap items-center justify-between gap-4">
          <div class="min-w-0">
            <h2 class="text-2xl font-semibold text-ink">{{ projectStore.detail.name }}</h2>
            <!-- 单项目 / 聚合根：直接展示技术栈（Spec §7.2；聚合根为 derived 概览） -->
            <div class="mt-2 flex flex-wrap gap-2">
              <TechnologyBadge
                v-for="tech in ownStack"
                :key="tech.id"
                :label="techNameOf(tech)"
              />
              <span
                v-if="ownStack.length === 0"
                class="text-sm text-faint"
              >
                {{ t("detail.noTechnologies") }}
              </span>
            </div>
          </div>
          <div class="flex flex-wrap gap-2">
            <button
              class="rounded-lg bg-primary px-3 py-2 text-sm font-medium text-white hover:bg-primary-hover disabled:opacity-60"
              :disabled="editorStore.opening?.projectId === projectId"
              :title="t('detail.openDefaultTitle')"
              @click="handleOpenEditor"
            >
              {{ editorStore.opening?.projectId === projectId ? t("detail.opening") : t("detail.openEditor") }}
            </button>
            <button
              class="rounded-lg border border-line bg-surface px-3 py-2 text-sm font-medium text-ink hover:bg-surface-2"
              :disabled="editorStore.opening?.projectId === projectId"
              @click="handleOpenFileManager"
            >
              {{ t("detail.openFileManager") }}
            </button>
          </div>
        </div>

        <!-- 完整路径 -->
        <div class="mt-4 rounded-lg border border-line-2 bg-surface-3 px-4 py-3">
          <div class="text-xs text-muted">{{ t("detail.fullPath") }}</div>
          <div class="mt-1 break-all text-sm text-ink">{{ projectStore.detail.path }}</div>
        </div>

        <dl class="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div>
            <dt class="text-sm text-muted">{{ t("detail.fileCount") }}</dt>
            <dd class="mt-1 text-ink">{{ projectStore.detail.file_count }}</dd>
          </div>
          <div>
            <dt class="text-sm text-muted">{{ t("detail.lastScan") }}</dt>
            <dd class="mt-1 text-ink">{{ formatDateTime(projectStore.detail.last_scan_at) }}</dd>
          </div>
          <div>
            <dt class="text-sm text-muted">{{ t("detail.createdAt") }}</dt>
            <dd class="mt-1 text-ink">{{ formatDateTime(projectStore.detail.created_at) }}</dd>
          </div>
          <div>
            <dt class="text-sm text-muted">{{ t("detail.updatedAt") }}</dt>
            <dd class="mt-1 text-ink">{{ formatDateTime(projectStore.detail.updated_at) }}</dd>
          </div>
          <div>
            <dt class="text-sm text-muted">{{ t("detail.kind") }}</dt>
            <dd class="mt-1">
              <span
                class="inline-flex items-center rounded px-2 py-0.5 text-sm font-medium"
                :class="
                  projectStore.detail.kind === 'aggregated_root'
                    ? 'bg-indigo-50 text-indigo-700'
                    : projectStore.detail.kind === 'category'
                    ? 'bg-amber-50 text-amber-700'
                    : 'bg-green-50 text-green-700 dark:bg-green-950 dark:text-green-400'
                "
              >
                {{ kindLabel }}
              </span>
            </dd>
          </div>
          <div>
            <dt class="text-sm text-muted">{{ t("detail.health") }}</dt>
            <dd class="mt-1 flex items-center gap-2">
              <div class="h-[6px] w-[60px] overflow-hidden rounded-full bg-line">
                <div
                  class="h-full rounded-full"
                  :style="{ width: `${projectStore.detail.health_score}%`, backgroundColor: healthColor }"
                />
              </div>
              <span class="text-sm font-medium" :style="{ color: healthColor }">
                {{ projectStore.detail.health_score }}
              </span>
            </dd>
          </div>
        </dl>
      </div>

      <!-- 前后端分区（Spec §7.2）：聚合根/分类目录基于 children 重算，子项目技术栈为 Source of Truth -->
      <div
        v-if="projectStore.detail && isAggregate"
        class="mt-6 rounded-lg border border-line-3 bg-surface p-6 shadow-sm"
      >
        <h3 class="font-medium text-ink">{{ t("detail.techSections") }}</h3>
        <!-- 加载中 -->
        <div v-if="childrenLoading" class="mt-4 text-sm text-muted">
          {{ t("detail.childrenLoading") }}
        </div>
        <!-- 加载失败 -->
        <div v-else-if="childrenError" class="mt-4 text-sm text-red-600 dark:text-red-400">
          {{ t("detail.childrenFailed", { msg: childrenError }) }}
        </div>
        <!-- 无子项目 -->
        <div v-else-if="sections.length === 0" class="mt-4 text-sm text-faint">
          {{ t("detail.noSubprojects") }}
        </div>
        <!-- 子项目分区 -->
        <div v-else class="mt-4 space-y-3">
          <div
            v-for="section in sections"
            :key="section.id"
            class="rounded-lg border border-line-2 bg-surface-3 px-4 py-3"
          >
            <div class="flex flex-wrap items-center justify-between gap-2">
              <span class="text-sm font-medium text-ink">{{ section.name }}</span>
              <span class="truncate text-xs text-faint" :title="section.path">
                {{ section.path }}
              </span>
            </div>
            <div class="mt-2 flex flex-wrap items-center gap-2">
              <TechnologyBadge
                v-for="tech in section.technologies"
                :key="tech.id"
                :label="techNameOf(tech)"
              />
              <span
                v-if="section.technologies.length === 0"
                class="text-xs text-faint"
              >
                {{ t("detail.noTechnologies") }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- 项目记忆区块 -->
      <div
        v-if="projectStore.detail"
        class="mt-6 rounded-lg border border-line-3 bg-surface p-6 shadow-sm"
      >
        <div class="flex items-center justify-between">
          <h3 class="font-medium text-ink">{{ t("detail.memory") }}</h3>
          <span
            class="rounded-full px-2.5 py-0.5 text-xs font-medium"
            :class="memoryStore.enabled ? 'bg-green-100 text-green-700 dark:bg-green-950 dark:text-green-400' : 'bg-surface-2 text-muted'"
          >
            {{ memoryStore.enabled ? t("detail.enabled") : t("detail.disabled") }}
          </span>
        </div>

        <div v-if="memoryStore.loading" class="mt-4 text-sm text-muted">
          {{ t("detail.memoryLoading") }}
        </div>

        <!-- 未启用 -->
        <div v-else-if="!memoryStore.enabled" class="mt-4">
          <p class="text-sm text-muted">
            {{ t("detail.memoryDesc") }}
            <code class="rounded bg-surface-2 px-1 py-0.5 text-xs">.ydevsphere/project.json</code>
            {{ t("detail.memoryDescSuffix") }}
          </p>
          <button
            class="mt-4 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-white hover:bg-primary-hover disabled:opacity-60"
            :disabled="memoryStore.loading"
            @click="handleEnable"
          >
            {{ t("detail.enableMemory") }}
          </button>
        </div>

        <!-- 已启用 -->
        <div v-else class="mt-4">
          <!-- 编辑态 -->
          <div v-if="editing" class="space-y-4">
            <div>
              <label class="text-sm text-muted">{{ t("detail.packageManager") }}</label>
              <input
                v-model="editPackageManager"
                class="mt-1 w-full rounded-lg border border-line px-3 py-2 text-sm outline-none focus:border-primary"
                :placeholder="t('detail.pmPlaceholder')"
              />
            </div>
            <div>
              <label class="text-sm text-muted">{{ t("detail.techStack") }}</label>
              <input
                v-model="editStackText"
                class="mt-1 w-full rounded-lg border border-line px-3 py-2 text-sm outline-none focus:border-primary"
                :placeholder="t('detail.stackPlaceholder')"
              />
            </div>
            <div class="flex gap-3">
              <button
                class="rounded-lg bg-primary px-4 py-2 text-sm font-medium text-white hover:bg-primary-hover disabled:opacity-60"
                :disabled="memoryStore.loading"
                @click="handleUpdate"
              >
                {{ memoryStore.loading ? t("detail.saving") : t("detail.save") }}
              </button>
              <button
                class="rounded-lg border border-line px-4 py-2 text-sm text-ink hover:bg-surface-2"
                @click="cancelEdit"
              >
                {{ t("detail.cancel") }}
              </button>
            </div>
          </div>

          <!-- 展示态 -->
          <div v-else>
            <dl class="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <div>
                <dt class="text-sm text-muted">{{ t("detail.techStackLabel") }}</dt>
                <dd class="mt-1 flex flex-wrap gap-2">
                  <template v-if="memoryStore.memory?.stack.length">
                    <TechnologyBadge v-for="s in memoryStore.memory.stack" :key="s" :label="s" />
                  </template>
                  <span v-else class="text-sm text-faint">{{ t("detail.none") }}</span>
                </dd>
              </div>
              <div>
                <dt class="text-sm text-muted">{{ t("detail.packageManager") }}</dt>
                <dd class="mt-1 text-ink">
                  {{ memoryStore.memory?.packageManager ?? t("detail.notDetected") }}
                </dd>
              </div>
            </dl>
            <button
              class="mt-4 text-sm text-primary hover:underline"
              @click="startEdit"
            >
              {{ t("detail.edit") }}
            </button>
          </div>

          <p v-if="memoryStore.error" class="mt-3 text-sm text-red-600 dark:text-red-400">
            {{ memoryStore.error }}
          </p>
        </div>
      </div>

      <!-- Git 信息区块 -->
      <div
        v-if="projectStore.detail"
        class="mt-6 rounded-lg border border-line-3 bg-surface p-6 shadow-sm"
      >
        <h3 class="font-medium text-ink">{{ t("detail.gitInfo") }}</h3>

        <!-- 加载中 -->
        <div v-if="gitStore.loading" class="mt-4 text-sm text-muted">
          {{ t("detail.gitLoading") }}
        </div>

        <!-- 失败 -->
        <div
          v-else-if="gitStore.error"
          class="mt-4 rounded-lg border border-red-100 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-900 dark:bg-red-950 dark:text-red-400"
        >
          {{ t("detail.gitFailed", { msg: gitStore.error }) }}
        </div>

        <!-- 非 git 仓库空态 -->
        <div
          v-else-if="!gitStore.info"
          class="mt-4 rounded-lg border border-dashed border-line px-4 py-6 text-center text-sm text-faint"
        >
          {{ t("detail.notGitRepo") }}
        </div>

        <!-- Git 仓库信息 -->
        <div v-else class="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div>
            <dt class="text-sm text-muted">{{ t("detail.branch") }}</dt>
            <dd class="mt-1 text-ink">
              <span
                v-if="gitStore.info.branch"
                class="inline-flex items-center gap-1 rounded bg-surface-2 px-2 py-0.5 text-sm font-medium text-ink"
              >
                <span class="text-faint">⎇</span>{{ gitStore.info.branch }}
              </span>
              <span v-else class="text-muted">{{ t("detail.headDetached") }}</span>
            </dd>
          </div>
          <div>
            <dt class="text-sm text-muted">{{ t("detail.worktreeStatus") }}</dt>
            <dd class="mt-1">
              <span
                class="inline-flex rounded px-2 py-0.5 text-sm font-medium"
                :class="isGitDirty(gitStore.info.status)
                  ? 'bg-amber-50 text-amber-700'
                  : 'bg-green-50 text-green-700 dark:bg-green-950 dark:text-green-400'"
              >
                {{ formatGitStatus(gitStore.info.status) }}
              </span>
            </dd>
          </div>

          <div v-if="gitStore.info.last_commit" class="sm:col-span-2">
            <dt class="text-sm text-muted">{{ t("detail.lastCommit") }}</dt>
            <dd class="mt-2 rounded-lg border border-line-2 bg-surface-3 px-4 py-3">
              <div class="flex flex-wrap items-center gap-2">
                <code class="rounded bg-line px-1.5 py-0.5 text-xs text-ink">
                  {{ gitStore.info.last_commit.hash }}
                </code>
                <span class="text-sm font-medium text-ink">
                  {{ gitStore.info.last_commit.message }}
                </span>
              </div>
              <div class="mt-1 text-xs text-muted">
                {{ gitStore.info.last_commit.author }} · {{ formatDateTime(gitStore.info.last_commit.time) }}
              </div>
            </dd>
          </div>
          <div v-else>
            <dt class="text-sm text-muted">{{ t("detail.lastCommit") }}</dt>
            <dd class="mt-1 text-muted">{{ t("detail.noCommit") }}</dd>
          </div>

          <div>
            <dt class="text-sm text-muted">{{ t("detail.lastUpdate") }}</dt>
            <dd class="mt-1 text-ink">
              {{ gitStore.info.last_update ? formatDateTime(gitStore.info.last_update) : "—" }}
            </dd>
          </div>
        </div>
      </div>

      <!-- 目录结构区块（懒加载目录树） -->
      <div
        v-if="projectStore.detail"
        class="mt-6 rounded-lg border border-line-3 bg-surface p-6 shadow-sm"
      >
        <h3 class="font-medium text-ink">{{ t("detail.dirTree") }}</h3>
        <p class="mt-1 text-xs text-muted">
          {{ t("detail.dirTreeHint") }}
        </p>
        <div class="mt-4">
          <DirTree :root-path="projectStore.detail.path" />
        </div>
      </div>
    </main>
  </div>
</template>
