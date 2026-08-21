<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useEditorStore } from "@/stores/editor";

const props = defineProps<{ projectId: number }>();

const editorStore = useEditorStore();
const { t } = useI18n();
const menuOpen = ref(false);
const menuEl = ref<HTMLElement | null>(null);

onMounted(() => editorStore.init());

/** 默认编辑器按钮文案 */
const defaultEditorName = computed(() => {
  const id = editorStore.defaultEditorId;
  if (id) {
    const ed = editorStore.editors.find((e) => e.id === id);
    if (ed && ed.open_method !== "unsupported") return t("detail.openIn", { name: ed.name });
  }
  const firstOpenable = editorStore.editors.find((e) => e.open_method !== "unsupported");
  if (firstOpenable) {
    return t("detail.openIn", { name: firstOpenable.name });
  }
  return null;
});

/** 可打开的编辑器（排除 open_method === "unsupported"） */
const openableEditors = computed(() =>
  editorStore.editors.filter((e) => e.open_method !== "unsupported")
);

/** 无可用编辑器时降级为文件管理器 */
async function handleDefaultOpen() {
  await editorStore.openEditor(props.projectId, null);
}

async function handleOpenWith(editorId: string) {
  menuOpen.value = false;
  await editorStore.openEditor(props.projectId, editorId);
}

async function handleFileManager() {
  menuOpen.value = false;
  await editorStore.openFileManager(props.projectId);
}

function toggleMenu() {
  menuOpen.value = !menuOpen.value;
}

// 点击外部关闭菜单
function onDocClick(e: MouseEvent) {
  if (menuEl.value && !menuEl.value.contains(e.target as Node)) {
    menuOpen.value = false;
  }
}
onMounted(() => document.addEventListener("click", onDocClick));
onUnmounted(() => document.removeEventListener("click", onDocClick));
</script>

<template>
  <div ref="menuEl" class="relative inline-flex items-center" @click.stop>
    <!-- 打开（默认编辑器 / 文件管理器）按钮 -->
    <button
      class="inline-flex items-center gap-1 rounded border border-line bg-surface px-2 py-1 text-xs font-medium text-ink hover:bg-surface-2 disabled:opacity-60"
      :disabled="editorStore.opening?.projectId === projectId"
      :title="defaultEditorName ?? t('detail.openFileManager')"
      @click="handleDefaultOpen"
    >
      {{ defaultEditorName ?? t("detail.open") }}
    </button>

    <!-- 下拉箭头 -->
    <button
      class="ml-1 rounded border border-line bg-surface px-1 py-1 text-xs text-muted hover:bg-surface-2"
      :title="t('table.more')"
      @click="toggleMenu"
    >
      ▾
    </button>

    <!-- 下拉菜单 -->
    <div
      v-if="menuOpen"
      class="absolute right-0 top-full z-20 mt-1 w-56 overflow-hidden rounded-lg border border-line-3 bg-surface py-1 shadow-lg"
      @click.stop
    >
      <button
        class="block w-full px-3 py-2 text-left text-sm text-ink hover:bg-surface-3"
        @click="handleFileManager"
      >
        {{ t("table.openFileManager") }}
      </button>
      <div v-if="openableEditors.length" class="my-1 border-t border-line-2"></div>
      <button
        v-for="ed in openableEditors"
        :key="ed.id"
        class="block w-full px-3 py-2 text-left text-sm text-ink hover:bg-surface-3"
        @click="handleOpenWith(ed.id)"
      >
        {{ t("table.openIn", { name: ed.name }) }}
      </button>
      <div v-if="!openableEditors.length" class="px-3 py-2 text-xs text-faint">
        {{ t("table.noEditor") }}
      </div>
    </div>
  </div>
</template>
