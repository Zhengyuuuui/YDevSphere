<script setup lang="ts">
/**
 * CloudField · 点阵云团背景
 *
 * 复刻自「Yunex AI Tools Hub」LandingPage 的 InteractivePixelGrid 效果：
 * 用 2D Simplex 噪声生成多层密度场，形成云状团簇，用 canvas 圆角小格渲染，
 * 带缓动动画与鼠标「墨水涟漪」交互。
 *
 * 适配说明（本项目设计约束：克制、专业、浅色基调）：
 * - 点阵改为浅灰色系，强度较原版调低，避免喧宾夺主；
 * - 保持单 canvas + 单 rAF 循环，不引入额外依赖；
 * - pointer-events: none，不阻挡页面交互。
 */

import { onMounted, onUnmounted, ref } from "vue";

const canvasRef = ref<HTMLCanvasElement | null>(null);

let ctx: CanvasRenderingContext2D | null = null;
let animationId: number | null = null;
let cells: {
  x: number;
  y: number;
  opacity: number;
  targetOpacity: number;
}[] = [];
let mouseX = -1000;
let mouseY = -1000;
let time = 0;

/* ---------------- 2D Simplex Noise（取自原版） ---------------- */
const permutation = [151,160,137,91,90,15,131,13,201,95,96,53,194,233,7,225,140,36,103,30,69,142,8,99,37,240,21,10,23,190,6,148,247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,57,177,33,88,237,149,56,87,174,20,125,136,171,168,68,175,74,165,71,134,139,48,27,166,77,146,158,231,83,111,229,122,60,211,133,230,220,105,92,41,55,46,245,40,244,102,143,54,66,21,97,62,1,11,113,83,191,230,215,25,124,136,197,178,95,76,128,125,194,156,29,118,200,178,197,151,63,225,166,145,177,138,162,96,112,122,103,196,156,217,88,229,189,180,213,199,201,183,124,58,138,38,88,217,227,9,39,249,203,165,107,19,225,221,194,172,22,181,214,26,82,33,198,224,173,66,61,167,177,215,248,116,132,208,69,186,112,237,112,17,164,128,114,66,135,64,230,254,95,224,243,178,231,133,127,235,83,92,192,128,217,229,246,244,245,160,124,163,143,241,167,177,214,215,130,220,207,28,23,78,198,147,114,141,185,19,191,141,1,73,100,134,199,21,106,202,115,194,167,138,188,250,135,72,140,230,49,244,44,90,133,118,166,64,202,140,65,207,26,75,204,90,77,220,86,240,41,219,117,167,255,124,114,105,140,121,199,81,140,54,148,164,75,164,171,73,169,60,97,207,174,244,6,173,230,184,111,155,37,137,190,159,59,119,179,29,180,60,190,220,211,197,95,204,174,71,62,12,241,226,165,144,20,218,211,223,81,97,136,149,240,51,96,84,162,96,178,44,125,227,208,124,68,142,131,62,230,40,75,118,171,87,223,131,97,224,170,244,188,84,59,144,45,109,140,9,195,177,121,112,198,162,186,122,70,170,138,28,221,137,241,120,105,111,249,206,138,165,251,83,140,214,156,255,135,7,3,145,144,63,231,145,19,238,73,238,163,35,19,216,68,231,136,163,56,145,177,229,169,89,189,247,139,249,162,234,120,59,118,210,113,96,66,120,164,164,194,170,128,130,198,126,94,233,98,99,205,74,54,137,235,204,125,3,113,41,133,63,5,169,9,196,170,136,176,255,175,134,161,2,169,138,162,201,58,197,162,198,136,158,21,200,248,112,67,204,252,122,99,50,142,152,115,59,128,123,184,174,14,104,133,186,183,145,91,242,95,244,48,114,111,179,185,131,202,203,84,250,134,221,167,154,122,167,191,119,167,120,219,212,139,175,84,246,208,98,254,163,172,129,26,240,129,250,41,228,186,46,73,208,48,159,219,175,225,35,73,217,86,152,178,241,215,80,207,226,80,210,209,83,132,224,236,124,179,137,41,145,121,11,161,153,188,111,26,131,95,171,167,138,148,109,253,114,161,211,151,55,173,223,64,250,89,240,121,97,143,54,200,63,2,181,105,125,113,99,5,118,218,135,47,226,90,71,98,36,210,51,194,47,229,107,240,153,174,20,166,176,91,197,137,8,224,186,231,133,11,69,174,138,190,42,146,98,67,150,219,175,99,204,31,122,31,115,138,109,109,135,203,214,168,144,171,207,145,255,145,22,128,203,224,137,167,119,141,65,96,227,145,163,135,193,246,208,203,163,155,3,67,127,240,62,225,195,55,36,68,4,184,230,134,81,107,80,235,217,220,178,81,212,33,234,191,164,184,97,69,14,144,219,143,204,122,113,131,178,63,137,226,232,51,101,160,64,231,86,96,176,15,207,149,218,9,162,83,156,76,83,163,133,215,128,126,133,240,63,209,4,167,214,134,179,157,166,191,245,220,220,239,188,194,233,251,55,109,130,207,38,168,109,191,223,150,13,65,149,158,253,242,155,166,157,46,137,144,55,128,7,17,189,155,118,147,51,196,59,106,78,99,122,63,109,179,155,180,91,192,120,206,106,182,250,3,252,56,13,140,160,64,55,211,208,186,144,137,224,243,242,147,50,26,194,72,185,145,189,249,255];

const perm = new Array(512);
const gradP = new Array(512);

for (let i = 0; i < 512; i++) {
  perm[i] = permutation[i & 255];
  gradP[i] = permutation[(i + 256) & 255] % 12;
}

const grad3 = [
  [1, 1, 0], [-1, 1, 0], [1, -1, 0], [-1, -1, 0],
  [1, 0, 1], [-1, 0, 1], [1, 0, -1], [-1, 0, -1],
  [0, 1, 1], [0, -1, 1], [0, 1, -1], [0, -1, -1],
];

function simplex2D(x: number, y: number): number {
  const F2 = 0.5 * (Math.sqrt(3) - 1);
  const G2 = (3 - Math.sqrt(3)) / 6;

  let n0: number, n1: number, n2: number;

  const s = (x + y) * F2;
  const i = Math.floor(x + s);
  const j = Math.floor(y + s);

  const t = (i + j) * G2;
  const X0 = i - t;
  const Y0 = j - t;
  const x0 = x - X0;
  const y0 = y - Y0;

  let i1: number, j1: number;
  if (x0 > y0) {
    i1 = 1;
    j1 = 0;
  } else {
    i1 = 0;
    j1 = 1;
  }

  const x1 = x0 - i1 + G2;
  const y1 = y0 - j1 + G2;
  const x2 = x0 - 1 + 2 * G2;
  const y2 = y0 - 1 + 2 * G2;

  const ii = i & 255;
  const jj = j & 255;

  let t0 = 0.5 - x0 * x0 - y0 * y0;
  if (t0 < 0) n0 = 0;
  else {
    t0 *= t0;
    const gi0 = gradP[ii + perm[jj]];
    n0 = t0 * t0 * (grad3[gi0][0] * x0 + grad3[gi0][1] * y0);
  }

  let t1 = 0.5 - x1 * x1 - y1 * y1;
  if (t1 < 0) n1 = 0;
  else {
    t1 *= t1;
    const gi1 = gradP[ii + i1 + perm[jj + j1]];
    n1 = t1 * t1 * (grad3[gi1][0] * x1 + grad3[gi1][1] * y1);
  }

  let t2 = 0.5 - x2 * x2 - y2 * y2;
  if (t2 < 0) n2 = 0;
  else {
    t2 *= t2;
    const gi2 = gradP[ii + 1 + perm[jj + 1]];
    n2 = t2 * t2 * (grad3[gi2][0] * x2 + grad3[gi2][1] * y2);
  }

  return 70 * (n0 + n1 + n2);
}

/* ---------------- 网格配置 ---------------- */
interface GridConfig {
  cols: number;
  rows: number;
  cellSize: number;
  interactionRadius: number;
}

function getGridConfig(): GridConfig {
  const width = window.innerWidth;
  if (width < 768) return { cols: 65, rows: 39, cellSize: 6, interactionRadius: 70 };
  if (width < 1024) return { cols: 130, rows: 65, cellSize: 5, interactionRadius: 90 };
  return { cols: 185, rows: 92, cellSize: 4, interactionRadius: 110 };
}

function initializeGrid() {
  if (!canvasRef.value) return;
  const canvas = canvasRef.value;
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
  ctx = canvas.getContext("2d");
  if (!ctx) return;

  const { cols, rows } = getGridConfig();
  const cellWidth = canvas.width / cols;
  const cellHeight = canvas.height / rows;

  cells = [];
  for (let i = 0; i < cols; i++) {
    for (let j = 0; j < rows; j++) {
      cells.push({
        x: i * cellWidth + cellWidth / 2,
        y: j * cellHeight + cellHeight / 2,
        opacity: 0,
        targetOpacity: 0,
      });
    }
  }
}

const lerp = (start: number, end: number, factor: number) =>
  start + (end - start) * factor;

/** 密度场：多层大尺度云团（0 空 → 1 密） */
function calculateDensityField(x: number, y: number, timeOffset: number): number {
  const scale1 = 0.0012;
  const noise1 = simplex2D(x * scale1 + timeOffset * 0.6, y * scale1 + timeOffset * 0.4);

  const scale2 = 0.0025;
  const noise2 = simplex2D(x * scale2 + 500 + timeOffset * 0.5, y * scale2 + 500 + timeOffset * 0.35);

  const scale3 = 0.005;
  const noise3 = simplex2D(x * scale3 + 1000 + timeOffset * 0.45, y * scale3 + 1000 + timeOffset * 0.3);

  const scale4 = 0.008;
  const noise4 = simplex2D(x * scale4 + 2000 + timeOffset * 0.55, y * scale4 + 2000 + timeOffset * 0.4);

  const combined = noise1 * 0.45 + noise2 * 0.3 + noise3 * 0.15 + noise4 * 0.1;
  return (combined + 1) / 2;
}

/** 亮度场：局部细节变化 */
function calculateBrightnessField(x: number, y: number, timeOffset: number): number {
  const scale1 = 0.008;
  const noise1 = simplex2D(x * scale1 + 200 + timeOffset * 0.6, y * scale1 + 200 + timeOffset * 0.7);

  const scale2 = 0.025;
  const noise2 = simplex2D(x * scale2 + 300 + timeOffset * 0.5, y * scale2 + 300 + timeOffset * 0.6);

  const combined = noise1 * 0.6 + noise2 * 0.4;
  return (combined + 1) / 2;
}

/**
 * 颜色：浅色主题用低对比浅灰点阵；dark 主题用比背景略亮的暗蓝灰点阵，
 * 让云团在黑色背景上保持低调、不刺眼。
 */
const isDark = () =>
  typeof document !== "undefined" && document.documentElement.dataset.theme === "dark";

function getColor(opacity: number): string {
  if (isDark()) {
    if (opacity < 0.15) return "rgba(60, 68, 78, 0.45)";
    if (opacity < 0.35) return "rgba(74, 83, 94, 0.6)";
    if (opacity < 0.55) return "rgba(90, 100, 112, 0.72)";
    if (opacity < 0.75) return "rgba(108, 118, 131, 0.82)";
    if (opacity < 0.9) return "rgba(124, 134, 147, 0.9)";
    return "rgba(140, 150, 163, 0.97)";
  }
  if (opacity < 0.15) return "rgba(180, 186, 194, 0.40)";
  if (opacity < 0.35) return "rgba(160, 168, 178, 0.55)";
  if (opacity < 0.55) return "rgba(140, 149, 160, 0.68)";
  if (opacity < 0.75) return "rgba(120, 129, 141, 0.78)";
  if (opacity < 0.9) return "rgba(100, 109, 122, 0.86)";
  return "rgba(85, 94, 108, 0.95)";
}

function updateCells() {
  const { interactionRadius } = getGridConfig();
  const timeOffset = time * 0.0008;
  const baseThreshold = 0.52;

  cells.forEach((cell) => {
    const rawDensity = calculateDensityField(cell.x, cell.y, timeOffset);

    let density = 0;
    if (rawDensity > baseThreshold) {
      density = (rawDensity - baseThreshold) / (1 - baseThreshold);
      density = density * density * (3 - 2 * density);
    }

    // 鼠标墨水涟漪交互
    const dx = cell.x - mouseX;
    const dy = cell.y - mouseY;
    const distance = Math.sqrt(dx * dx + dy * dy);

    if (distance < interactionRadius && mouseX > 0) {
      const distanceRatio = distance / interactionRadius;
      const wave = Math.sin(distanceRatio * Math.PI * 4) * 0.15;
      const falloff = (1 - distanceRatio) * (1 - distanceRatio);
      const densityBoost = falloff * 0.7 + wave * falloff;
      density = Math.min(density + densityBoost, 1.0);
    }

    if (density < 0.08) {
      cell.targetOpacity = 0;
    } else {
      const brightness = calculateBrightnessField(cell.x, cell.y, timeOffset);
      cell.targetOpacity = Math.min(density * brightness, 1.0);
    }

    cell.opacity = lerp(cell.opacity, cell.targetOpacity, 0.06);
  });
}

function render() {
  if (!ctx || !canvasRef.value) return;
  const canvas = canvasRef.value;
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  const { cellSize } = getGridConfig();

  cells.forEach((cell) => {
    if (cell.opacity < 0.08) return;
    ctx!.fillStyle = getColor(cell.opacity);

    const size = cellSize * (0.85 + cell.opacity * 0.3);
    const halfSize = size / 2;

    ctx!.beginPath();
    ctx!.roundRect(cell.x - halfSize, cell.y - halfSize, size, size, 1.5);
    ctx!.fill();
  });
}

function animate() {
  time++;
  updateCells();
  render();
  animationId = requestAnimationFrame(animate);
}

function handleMouseMove(e: MouseEvent) {
  mouseX = e.clientX;
  mouseY = e.clientY;
}

function handleMouseLeave() {
  mouseX = -1000;
  mouseY = -1000;
}

function handleResize() {
  initializeGrid();
}

onMounted(() => {
  initializeGrid();
  animate();
  window.addEventListener("mousemove", handleMouseMove, { passive: true });
  document.addEventListener("mouseleave", handleMouseLeave);
  window.addEventListener("resize", handleResize);
});

onUnmounted(() => {
  if (animationId) cancelAnimationFrame(animationId);
  window.removeEventListener("mousemove", handleMouseMove);
  document.removeEventListener("mouseleave", handleMouseLeave);
  window.removeEventListener("resize", handleResize);
});
</script>

<template>
  <canvas
    ref="canvasRef"
    class="pointer-events-none fixed inset-0 h-full w-full"
    style="z-index: 0"
  ></canvas>
</template>
