<script setup lang="ts">
import { onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import ToastContainer from "@/components/ToastContainer.vue";
import { useSettingsStore } from "@/stores/settings";
import { useI18nStore } from "@/stores/i18n";

const router = useRouter();
const route = useRoute();
const settings = useSettingsStore();
const i18nStore = useI18nStore();
const { t } = useI18n();

onMounted(async () => {
  // 先初始化语言（读后端/localStorage 偏好），再恢复工作区
  await i18nStore.init();
  // 启动自动恢复：读取已保存的工作区偏好。
  // 非空路径 → 直达 /overview；为空 → 保持 Welcome（首次使用引导）。
  const saved = await settings.restore();
  if (saved && route.name !== "overview") {
    router.replace({ name: "overview" });
  }
});
</script>

<template>
  <div class="min-h-screen bg-gray-50 text-gray-900">
    <ToastContainer />

    <!-- 启动恢复中的 loading 遮罩，避免闪烁 / 误跳转 -->
    <div
      v-if="settings.restoring"
      class="fixed inset-0 z-[60] flex items-center justify-center bg-gray-50"
    >
      <div class="flex flex-col items-center gap-3">
        <img src="/logo.png" alt="YDevSphere" class="h-16 w-auto object-contain" />
        <span class="text-sm text-gray-500">{{ t("app.restoring") }}</span>
      </div>
    </div>

    <RouterView />
  </div>
</template>
