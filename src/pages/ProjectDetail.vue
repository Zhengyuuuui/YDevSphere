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

/** 健康度颜色分级（≥60 绿、40-59 黄、<40 灰） */
const healthColor = computed(() => {
  const s = projectStore.detail?.health_score ?? 0;
  if (s >= 60) return "#16A34A";
  if (s >= 40) return "#D97706";
  return "#9CA3AF";
});

/** 编辑状态 */
const editing = ref(false);
const editPackageManager = ref("");
const editStackText = ref("");

function load() {
  projectStore.fetchProjectDetail(projectId.value);
  memoryStore.fetchMemory(projectId.value);
  gitStore.fetchGit(projectId.value);
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
  <div class="min-h-full bg-[#F7F8FA]">
    <main class="mx-auto max-w-4xl px-8 py-8">
      <!-- 返回按钮 -->
      <button class="text-sm text-blue-600 hover:underline" @click="goBack">
        {{ t("detail.back") }}
      </button>

      <!-- 加载中 -->
      <div
        v-if="projectStore.detailLoading"
        class="mt-8 rounded-lg border border-gray-100 bg-white p-10 text-center text-sm text-gray-500 shadow-sm"
      >
        {{ t("detail.loading") }}
      </div>

      <!-- 错误 -->
      <div
        v-else-if="projectStore.error && !projectStore.detail"
        class="mt-8 rounded-lg border border-red-200 bg-red-50 p-10 text-center text-sm text-red-700"
      >
        {{ projectStore.error }}
      </div>

      <!-- 项目不存在（后端返回 null） -->
      <div
        v-else-if="!projectStore.detail"
        class="mt-8 rounded-lg border border-dashed border-gray-300 p-10 text-center text-gray-500"
      >
        {{ t("detail.notFound") }}
        <div class="mt-4">
          <RouterLink to="/projects" class="text-sm text-blue-600 hover:underline">
            {{ t("detail.backToList") }}
          </RouterLink>
        </div>
      </div>

      <!-- 项目详情 -->
      <div
        v-else
        class="mt-4 rounded-lg border border-gray-200 bg-white p-6 shadow-sm"
      >
        <div class="flex flex-wrap items-center justify-between gap-4">
          <div class="min-w-0">
            <h2 class="text-2xl font-semibold text-gray-900">{{ projectStore.detail.name }}</h2>
            <div class="mt-2 flex flex-wrap gap-2">
              <TechnologyBadge :label="projectStore.detail.language" />
              <TechnologyBadge :label="projectStore.detail.framework" />
            </div>
          </div>
          <div class="flex flex-wrap gap-2">
            <button
              class="rounded-lg bg-blue-600 px-3 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-60"
              :disabled="editorStore.opening?.projectId === projectId"
              :title="t('detail.openDefaultTitle')"
              @click="handleOpenEditor"
            >
              {{ editorStore.opening?.projectId === projectId ? t("detail.opening") : t("detail.openEditor") }}
            </button>
            <button
              class="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100"
              :disabled="editorStore.opening?.projectId === projectId"
              @click="handleOpenFileManager"
            >
              {{ t("detail.openFileManager") }}
            </button>
          </div>
        </div>

        <!-- 完整路径 -->
        <div class="mt-4 rounded-lg border border-gray-100 bg-gray-50 px-4 py-3">
          <div class="text-xs text-gray-500">{{ t("detail.fullPath") }}</div>
          <div class="mt-1 break-all text-sm text-gray-800">{{ projectStore.detail.path }}</div>
        </div>

        <dl class="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div>
            <dt class="text-sm text-gray-500">{{ t("detail.fileCount") }}</dt>
            <dd class="mt-1 text-gray-900">{{ projectStore.detail.file_count }}</dd>
          </div>
          <div>
            <dt class="text-sm text-gray-500">{{ t("detail.lastScan") }}</dt>
            <dd class="mt-1 text-gray-900">{{ formatDateTime(projectStore.detail.last_scan_at) }}</dd>
          </div>
          <div>
            <dt class="text-sm text-gray-500">{{ t("detail.createdAt") }}</dt>
            <dd class="mt-1 text-gray-900">{{ formatDateTime(projectStore.detail.created_at) }}</dd>
          </div>
          <div>
            <dt class="text-sm text-gray-500">{{ t("detail.updatedAt") }}</dt>
            <dd class="mt-1 text-gray-900">{{ formatDateTime(projectStore.detail.updated_at) }}</dd>
          </div>
          <div>
            <dt class="text-sm text-gray-500">{{ t("detail.kind") }}</dt>
            <dd class="mt-1">
              <span
                class="inline-flex items-center rounded px-2 py-0.5 text-sm font-medium"
                :class="
                  projectStore.detail.kind === 'aggregated_root'
                    ? 'bg-indigo-50 text-indigo-700'
                    : projectStore.detail.kind === 'category'
                    ? 'bg-amber-50 text-amber-700'
                    : 'bg-green-50 text-green-700'
                "
              >
                {{ kindLabel }}
              </span>
            </dd>
          </div>
          <div>
            <dt class="text-sm text-gray-500">{{ t("detail.health") }}</dt>
            <dd class="mt-1 flex items-center gap-2">
              <div class="h-[6px] w-[60px] overflow-hidden rounded-full bg-gray-200">
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

      <!-- 项目记忆区块 -->
      <div
        v-if="projectStore.detail"
        class="mt-6 rounded-lg border border-gray-200 bg-white p-6 shadow-sm"
      >
        <div class="flex items-center justify-between">
          <h3 class="font-medium text-gray-900">{{ t("detail.memory") }}</h3>
          <span
            class="rounded-full px-2.5 py-0.5 text-xs font-medium"
            :class="memoryStore.enabled ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-600'"
          >
            {{ memoryStore.enabled ? t("detail.enabled") : t("detail.disabled") }}
          </span>
        </div>

        <div v-if="memoryStore.loading" class="mt-4 text-sm text-gray-500">
          {{ t("detail.memoryLoading") }}
        </div>

        <!-- 未启用 -->
        <div v-else-if="!memoryStore.enabled" class="mt-4">
          <p class="text-sm text-gray-600">
            {{ t("detail.memoryDesc") }}
            <code class="rounded bg-gray-100 px-1 py-0.5 text-xs">.ydevsphere/project.json</code>
            {{ t("detail.memoryDescSuffix") }}
          </p>
          <button
            class="mt-4 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-60"
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
              <label class="text-sm text-gray-500">{{ t("detail.packageManager") }}</label>
              <input
                v-model="editPackageManager"
                class="mt-1 w-full rounded-lg border border-gray-300 px-3 py-2 text-sm outline-none focus:border-blue-400"
                :placeholder="t('detail.pmPlaceholder')"
              />
            </div>
            <div>
              <label class="text-sm text-gray-500">{{ t("detail.techStack") }}</label>
              <input
                v-model="editStackText"
                class="mt-1 w-full rounded-lg border border-gray-300 px-3 py-2 text-sm outline-none focus:border-blue-400"
                :placeholder="t('detail.stackPlaceholder')"
              />
            </div>
            <div class="flex gap-3">
              <button
                class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-60"
                :disabled="memoryStore.loading"
                @click="handleUpdate"
              >
                {{ memoryStore.loading ? t("detail.saving") : t("detail.save") }}
              </button>
              <button
                class="rounded-lg border border-gray-300 px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
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
                <dt class="text-sm text-gray-500">{{ t("detail.techStackLabel") }}</dt>
                <dd class="mt-1 flex flex-wrap gap-2">
                  <template v-if="memoryStore.memory?.stack.length">
                    <TechnologyBadge v-for="s in memoryStore.memory.stack" :key="s" :label="s" />
                  </template>
                  <span v-else class="text-sm text-gray-400">{{ t("detail.none") }}</span>
                </dd>
              </div>
              <div>
                <dt class="text-sm text-gray-500">{{ t("detail.packageManager") }}</dt>
                <dd class="mt-1 text-gray-900">
                  {{ memoryStore.memory?.packageManager ?? t("detail.notDetected") }}
                </dd>
              </div>
            </dl>
            <button
              class="mt-4 text-sm text-blue-600 hover:underline"
              @click="startEdit"
            >
              {{ t("detail.edit") }}
            </button>
          </div>

          <p v-if="memoryStore.error" class="mt-3 text-sm text-red-600">
            {{ memoryStore.error }}
          </p>
        </div>
      </div>

      <!-- Git 信息区块 -->
      <div
        v-if="projectStore.detail"
        class="mt-6 rounded-lg border border-gray-200 bg-white p-6 shadow-sm"
      >
        <h3 class="font-medium text-gray-900">{{ t("detail.gitInfo") }}</h3>

        <!-- 加载中 -->
        <div v-if="gitStore.loading" class="mt-4 text-sm text-gray-500">
          {{ t("detail.gitLoading") }}
        </div>

        <!-- 失败 -->
        <div
          v-else-if="gitStore.error"
          class="mt-4 rounded-lg border border-red-100 bg-red-50 px-4 py-3 text-sm text-red-700"
        >
          {{ t("detail.gitFailed", { msg: gitStore.error }) }}
        </div>

        <!-- 非 git 仓库空态 -->
        <div
          v-else-if="!gitStore.info"
          class="mt-4 rounded-lg border border-dashed border-gray-200 px-4 py-6 text-center text-sm text-gray-400"
        >
          {{ t("detail.notGitRepo") }}
        </div>

        <!-- Git 仓库信息 -->
        <div v-else class="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div>
            <dt class="text-sm text-gray-500">{{ t("detail.branch") }}</dt>
            <dd class="mt-1 text-gray-900">
              <span
                v-if="gitStore.info.branch"
                class="inline-flex items-center gap-1 rounded bg-gray-100 px-2 py-0.5 text-sm font-medium text-gray-800"
              >
                <span class="text-gray-400">⎇</span>{{ gitStore.info.branch }}
              </span>
              <span v-else class="text-gray-500">{{ t("detail.headDetached") }}</span>
            </dd>
          </div>
          <div>
            <dt class="text-sm text-gray-500">{{ t("detail.worktreeStatus") }}</dt>
            <dd class="mt-1">
              <span
                class="inline-flex rounded px-2 py-0.5 text-sm font-medium"
                :class="isGitDirty(gitStore.info.status)
                  ? 'bg-amber-50 text-amber-700'
                  : 'bg-green-50 text-green-700'"
              >
                {{ formatGitStatus(gitStore.info.status) }}
              </span>
            </dd>
          </div>

          <div v-if="gitStore.info.last_commit" class="sm:col-span-2">
            <dt class="text-sm text-gray-500">{{ t("detail.lastCommit") }}</dt>
            <dd class="mt-2 rounded-lg border border-gray-100 bg-gray-50 px-4 py-3">
              <div class="flex flex-wrap items-center gap-2">
                <code class="rounded bg-gray-200 px-1.5 py-0.5 text-xs text-gray-700">
                  {{ gitStore.info.last_commit.hash }}
                </code>
                <span class="text-sm font-medium text-gray-900">
                  {{ gitStore.info.last_commit.message }}
                </span>
              </div>
              <div class="mt-1 text-xs text-gray-500">
                {{ gitStore.info.last_commit.author }} · {{ formatDateTime(gitStore.info.last_commit.time) }}
              </div>
            </dd>
          </div>
          <div v-else>
            <dt class="text-sm text-gray-500">{{ t("detail.lastCommit") }}</dt>
            <dd class="mt-1 text-gray-500">{{ t("detail.noCommit") }}</dd>
          </div>

          <div>
            <dt class="text-sm text-gray-500">{{ t("detail.lastUpdate") }}</dt>
            <dd class="mt-1 text-gray-900">
              {{ gitStore.info.last_update ? formatDateTime(gitStore.info.last_update) : "—" }}
            </dd>
          </div>
        </div>
      </div>

      <!-- 目录结构区块（懒加载目录树） -->
      <div
        v-if="projectStore.detail"
        class="mt-6 rounded-lg border border-gray-200 bg-white p-6 shadow-sm"
      >
        <h3 class="font-medium text-gray-900">{{ t("detail.dirTree") }}</h3>
        <p class="mt-1 text-xs text-gray-500">
          {{ t("detail.dirTreeHint") }}
        </p>
        <div class="mt-4">
          <DirTree :root-path="projectStore.detail.path" />
        </div>
      </div>
    </main>
  </div>
</template>
