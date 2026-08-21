<script setup lang="ts">
defineProps<{
  value: string | number;
  label: string;
}>();
</script>

<template>
  <div class="flex min-w-0 flex-col gap-1">
    <span class="text-[28px] font-semibold leading-none tracking-tight tabular-nums text-ink">
      {{ value }}
    </span>
    <span class="text-[13px] text-faint">{{ label }}</span>
  </div>
</template>
