import { defineStore } from "pinia";
import { ref } from "vue";
import type { ScanHistory } from "@/types";
import { scanProjects, ApiError } from "@/api/project";
import { useProjectStore } from "./project";

/** 扫描状态机状态 */
export type ScanStatus = "idle" | "scanning" | "done" | "error";

/** 最近一次扫描结果摘要 */
export interface ScanResultSummary {
  scannedCount: number;
  ignoredCount: number;
  history: ScanHistory | null;
  /** 扫描耗时（毫秒） */
  durationMs: number;
}

/** 扫描相关状态与动作 */
export const useScannerStore = defineStore("scanner", () => {
  const status = ref<ScanStatus>("idle");
  const error = ref<string | null>(null);
  /** 错误码（后端结构化错误 code，如 INVALID_DIRECTORY）；无则 null */
  const errorCode = ref<string | null>(null);
  const lastResult = ref<ScanResultSummary | null>(null);
  /** 是否已触发过「启用项目记忆」询问（首次扫描成功且有项目时置 true，避免重复弹窗） */
  const memoryPromptTriggered = ref(false);

  /**
   * 执行扫描。
   * - 成功后：状态置为 done，将识别出的项目写入 project store 并刷新列表。
   * - 失败后：状态置为 error，记录错误信息。
   */
  async function scan(workspacePath: string) {
    status.value = "scanning";
    error.value = null;
    errorCode.value = null;
    const startedAt = Date.now();
    try {
      const result = await scanProjects(workspacePath);
      const projectStore = useProjectStore();
      projectStore.setProjects(result.projects);
      lastResult.value = {
        scannedCount: result.scanned_count,
        ignoredCount: result.ignored_count,
        history: result.history,
        durationMs: Date.now() - startedAt,
      };
      status.value = "done";
      // 首次扫描识别到项目时，标记需要询问是否启用项目记忆
      if (!memoryPromptTriggered.value && result.projects.length > 0) {
        memoryPromptTriggered.value = true;
      }
    } catch (e) {
      status.value = "error";
      error.value = e instanceof ApiError ? e.message : String(e);
      errorCode.value = e instanceof ApiError ? e.code ?? null : null;
    }
  }

  /** 重置为初始状态 */
  function reset() {
    status.value = "idle";
    error.value = null;
    errorCode.value = null;
    lastResult.value = null;
    memoryPromptTriggered.value = false;
  }

  return {
    status,
    error,
    errorCode,
    lastResult,
    memoryPromptTriggered,
    scan,
    reset,
  };
});
