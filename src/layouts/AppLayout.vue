<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import AppSidebar from "@/components/AppSidebar.vue";
import { useLayoutStore } from "@/stores/layout";

const layout = useLayoutStore();

/** 监听窗口尺寸变化，产出全局 appMode（单个 ResizeObserver，不做列宽测量） */
let ro: ResizeObserver | null = null;

onMounted(() => {
  // 初始同步一次
  layout.setWidth(window.innerWidth);
  // 用单个 ResizeObserver 监听视口宽度（等价 window.resize，一次回调即可）
  ro = new ResizeObserver(() => {
    layout.setWidth(window.innerWidth);
  });
  ro.observe(document.documentElement);
});

onUnmounted(() => {
  ro?.disconnect();
  ro = null;
});
</script>

<template>
  <div class="flex h-screen overflow-hidden bg-[#F7F8FA]">
    <AppSidebar />
    <main class="flex-1 overflow-y-auto">
      <RouterView />
    </main>
  </div>
</template>
