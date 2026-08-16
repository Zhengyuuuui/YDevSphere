<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import type { SystemWorkspace } from "@/types";
import { getSystemWorkspaces } from "@/api/project";
import { useSettingsStore } from "@/stores/settings";
import { useScannerStore } from "@/stores/scanner";
import { toast } from "@/lib/toast";

const router = useRouter();
const { t } = useI18n();
const settings = useSettingsStore();
const scanner = useScannerStore();

/** 系统工作区入口（Documents / Desktop） */
const systemWorkspaces = ref<SystemWorkspace[]>([]);
/** 「其他选项」折叠状态 */
const moreOpen = ref(false);
/** 正在导入的入口 label（用于按钮 loading） */
const importingLabel = ref<string | null>(null);
/** 一键导入所有入口的 loading 标志 */
const importingAll = ref(false);

const documentsEntry = computed(() =>
  systemWorkspaces.value.find((w) => w.kind === "documents")
);
const desktopEntry = computed(() =>
  systemWorkspaces.value.find((w) => w.kind === "desktop")
);

onMounted(async () => {
  try {
    systemWorkspaces.value = await getSystemWorkspaces();
  } catch (e) {
    toast.error(t("workspace.loadSystemFailed", { msg: e instanceof Error ? e.message : String(e) }));
  }
});

/** 一键导入单个：扫描路径 → 加入工作区集合 → 持久化 → 跳 Overview */
async function importWorkspace(ws: SystemWorkspace | undefined) {
  if (!ws?.exists || !ws.path) {
    toast.error(t("workspace.notExistImport", { name: ws?.label ?? "" }));
    return;
  }
  if (scanner.status === "scanning") return;

  importingLabel.value = ws.label;
  await scanner.scan(ws.path);
  importingLabel.value = null;

  if (scanner.status === "error") {
    toast.error(t("scan.importFailed", { msg: scanner.error }));
    return;
  }

  // 加入工作区集合（去重，不覆盖已有工作区）+ 持久化到后端
  await settings.addWorkspace(ws.path);
  router.push("/overview");
}

/** 一键导入所有可用系统工作区（Documents + Desktop 都加入集合，不覆盖） */
async function importAll() {
  // 1) 筛选出「存在且有路径」的入口（path 已确认为非空字符串）
  const entries: { label: string; path: string }[] = [];
  for (const w of [documentsEntry.value, desktopEntry.value]) {
    if (w?.exists && w.path) entries.push({ label: w.label, path: w.path });
  }
  // 2) 一个都不存在 → 明确提示，不进入扫描
  if (entries.length === 0) {
    toast.error(t("workspace.bothNotExist"));
    return;
  }
  // 3) 防重复：扫描中直接返回
  if (scanner.status === "scanning") return;

  importingAll.value = true;
  let scannedAny = false;
  try {
    // 4) 先把所有入口都加入工作区集合（不覆盖已有，addWorkspace 已持久化到后端）
    for (const entry of entries) {
      await settings.addWorkspace(entry.path);
    }
    // 5) 依次扫描所有已加入的工作区（汇总）
    for (const entry of entries) {
      await scanner.scan(entry.path);
      if (scanner.status === "error") {
        toast.error(
          t("scan.importFailedAt", {
            name: entry.label,
            msg: scanner.error ?? t("scan.unknownError"),
          })
        );
        continue; // 单个失败不阻断其余工作区
      }
      scannedAny = true;
    }
    // 6) 全部扫描成功则进入 Overview（工作区集合已在第 4 步持久化到后端）
    if (scannedAny) {
      router.push("/overview");
    }
  } finally {
    importingAll.value = false;
  }
}

/** 手动选择工作区（保留） */
async function chooseWorkspace() {
  const selected = await settings.selectWorkspacePath();
  if (selected) {
    router.push("/overview");
  }
}

function toggleMore() {
  moreOpen.value = !moreOpen.value;
}
</script>

<template>
  <div class="flex min-h-screen flex-col items-center justify-center px-6">
    <div class="w-full max-w-2xl">
      <!-- Logo + 标题 -->
      <div class="text-center">
        <img
          src="/logo.png"
          alt="YDevSphere"
          class="mx-auto h-24 w-auto object-contain"
        />
        <h1 class="mt-6 text-4xl font-bold tracking-tight">YDevSphere</h1>
        <p class="mt-4 text-lg text-gray-600">
          {{ t("workspace.welcomeTagline") }}
        </p>
      </div>

      <!-- 一键导入所有系统工作区 -->
      <div class="mt-10">
        <button
          class="group w-full rounded-lg border border-gray-200 bg-white p-6 text-left shadow-sm transition hover:border-blue-200 hover:shadow-md disabled:cursor-not-allowed disabled:opacity-50"
          :disabled="
            (!documentsEntry?.exists && !desktopEntry?.exists) ||
            scanner.status === 'scanning'
          "
          @click="importAll"
        >
          <div class="flex items-center gap-3">
            <span class="flex h-10 w-10 items-center justify-center rounded-lg bg-blue-50 text-blue-600">
              📁
            </span>
            <div>
              <div class="font-medium text-gray-900">{{ t("workspace.importAll") }}</div>
              <div class="mt-0.5 text-xs text-gray-500">
                {{ t("workspace.importAllDesc") }}
              </div>
            </div>
          </div>
          <div v-if="importingAll" class="mt-3 text-xs text-blue-600">
            {{ t("workspace.importing") }}
          </div>
        </button>
      </div>

      <!-- 其他选项 -->
      <div class="mt-4 text-center">
        <button
          class="text-sm text-gray-500 hover:text-gray-700"
          @click="toggleMore"
        >
          {{ moreOpen ? t("workspace.collapseOptions") : t("workspace.moreOptions") }}
        </button>

        <div
          v-if="moreOpen"
          class="mx-auto mt-3 max-w-sm overflow-hidden rounded-lg border border-gray-200 bg-white shadow-sm"
        >
          <button
            class="flex w-full items-center justify-between px-4 py-3 text-left text-sm text-gray-700 hover:bg-gray-50"
            @click="chooseWorkspace"
          >
            <span>{{ t("workspace.select") }}</span>
            <span class="text-gray-400">{{ t("workspace.manualSelect") }}</span>
          </button>
          <button
            class="flex w-full items-center justify-between border-t border-gray-100 px-4 py-3 text-left text-sm text-gray-700 hover:bg-gray-50 disabled:opacity-50"
            :disabled="!documentsEntry?.exists || scanner.status === 'scanning'"
            @click="importWorkspace(documentsEntry)"
          >
            <span>{{ t("workspace.importDocuments") }}</span>
            <span class="text-gray-400">{{ documentsEntry?.exists ? t("workspace.available") : t("workspace.notExist") }}</span>
          </button>
          <button
            class="flex w-full items-center justify-between border-t border-gray-100 px-4 py-3 text-left text-sm text-gray-700 hover:bg-gray-50 disabled:opacity-50"
            :disabled="!desktopEntry?.exists || scanner.status === 'scanning'"
            @click="importWorkspace(desktopEntry)"
          >
            <span>{{ t("workspace.importDesktop") }}</span>
            <span class="text-gray-400">{{ desktopEntry?.exists ? t("workspace.available") : t("workspace.notExist") }}</span>
          </button>
        </div>
      </div>

      <p v-if="settings.error" class="mt-4 text-center text-sm text-red-600">
        {{ settings.error }}
      </p>
    </div>
  </div>
</template>
