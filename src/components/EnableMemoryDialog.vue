<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import type { Project } from "@/types";
import { useMemoryStore } from "@/stores/memory";
import { useProjectStore } from "@/stores/project";
import { toast } from "@/lib/toast";

const { t } = useI18n();

const props = defineProps<{
  open: boolean;
  projects: Project[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const memoryStore = useMemoryStore();
const projectStore = useProjectStore();
const working = ref(false);

async function handleEnable() {
  if (working.value) return;
  working.value = true;
  let enabledCount = 0;
  for (const p of props.projects) {
    const ok = await memoryStore.enable(p.id, null);
    if (ok) enabledCount++;
  }
  working.value = false;
  if (enabledCount > 0) {
    toast.success(t("memoryDialog.enabledToast", { count: enabledCount }));
    await projectStore.fetchProjects();
  }
  emit("close");
}

function handleSkip() {
  // 安全红线：跳过不触发任何写入
  emit("close");
}
</script>

<template>
  <div
    v-if="open"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
  >
    <div class="w-full max-w-md rounded-lg bg-white p-6 shadow-xl">
      <h3 class="text-lg font-semibold text-gray-900">{{ t("memoryDialog.title") }}</h3>
      <p class="mt-2 text-sm text-gray-600">
        {{ t("memoryDialog.body", { count: projects.length }) }}
        <code class="rounded bg-gray-100 px-1 py-0.5 text-xs">.ydevsphere/project.json</code>
        {{ t("memoryDialog.bodySuffix") }}
      </p>
      <p class="mt-2 text-xs text-gray-400">
        {{ t("memoryDialog.note") }}<code>.ydevsphere/</code>{{ t("memoryDialog.noteSuffix") }}
      </p>

      <div class="mt-6 flex justify-end gap-3">
        <button
          class="rounded-lg border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100"
          :disabled="working"
          @click="handleSkip"
        >
          {{ t("memoryDialog.skip") }}
        </button>
        <button
          class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-60"
          :disabled="working"
          @click="handleEnable"
        >
          {{ working ? t("memoryDialog.enabling") : t("memoryDialog.enable") }}
        </button>
      </div>
    </div>
  </div>
</template>
