<script setup lang="ts">
import { computed } from "vue";

interface ActivityPoint {
  day: string;
  commits: number;
}

/**
 * 工作区活动图（本周 commits）。
 *
 * ⚠️ 当前为 mock 数据（后端暂无 get_stats 接口），待接后端后替换。
 */
const props = defineProps<{
  data?: ActivityPoint[];
}>();

const ACTIVITY: ActivityPoint[] = [
  { day: "Mon", commits: 4 },
  { day: "Tue", commits: 7 },
  { day: "Wed", commits: 11 },
  { day: "Thu", commits: 6 },
  { day: "Fri", commits: 14 },
  { day: "Sat", commits: 3 },
  { day: "Sun", commits: 8 },
];

const data = computed(() => props.data ?? ACTIVITY);
const maxCommits = computed(() => Math.max(...data.value.map((d) => d.commits), 1));

const WIDTH = 560;
const HEIGHT = 120;
const PAD = 8;

/** 计算折线路径（近似面积图填充） */
function points() {
  const step = (WIDTH - PAD * 2) / (data.value.length - 1);
  return data.value.map((d, i) => {
    const x = PAD + i * step;
    const y = HEIGHT - PAD - (d.commits / maxCommits.value) * (HEIGHT - PAD * 2);
    return { x, y, ...d };
  });
}

const linePath = computed(() => {
  const pts = points();
  return pts.map((p, i) => `${i === 0 ? "M" : "L"}${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(" ");
});

const areaPath = computed(() => {
  const pts = points();
  if (pts.length === 0) return "";
  const first = pts[0];
  const last = pts[pts.length - 1];
  const line = pts.map((p, i) => `${i === 0 ? "M" : "L"}${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(" ");
  return `${line} L${last.x.toFixed(1)},${HEIGHT - PAD} L${first.x.toFixed(1)},${HEIGHT - PAD} Z`;
});
</script>

<template>
  <div>
    <svg :viewBox="`0 0 ${WIDTH} ${HEIGHT}`" class="w-full" preserveAspectRatio="none">
      <defs>
        <linearGradient id="actGrad" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-color="#2563EB" stop-opacity="0.15" />
          <stop offset="100%" stop-color="#2563EB" stop-opacity="0" />
        </linearGradient>
      </defs>
      <path :d="areaPath" fill="url(#actGrad)" />
      <path :d="linePath" fill="none" stroke="#2563EB" stroke-width="1.5" stroke-linecap="round" />
      <circle
        v-for="(p, i) in points()"
        :key="i"
        :cx="p.x"
        :cy="p.y"
        r="2.5"
        fill="#2563EB"
      />
    </svg>

    <div class="mt-3 flex justify-between">
      <span v-for="d in data" :key="d.day" class="text-[11px] text-[#B0B7C3]">
        {{ d.day }}
      </span>
    </div>
  </div>
</template>
