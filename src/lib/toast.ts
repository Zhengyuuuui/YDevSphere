import { reactive } from "vue";

export type ToastType = "info" | "success" | "error";

export interface ToastItem {
  id: number;
  type: ToastType;
  message: string;
}

/** 全局轻量 toast 状态（非 Pinia，保持简单） */
export const toastState = reactive<{ toasts: ToastItem[] }>({
  toasts: [],
});

let seed = 0;

function push(type: ToastType, message: string) {
  const id = ++seed;
  toastState.toasts.push({ id, type, message });
  // 3 秒后自动消失
  setTimeout(() => remove(id), 3000);
}

export function remove(id: number) {
  const idx = toastState.toasts.findIndex((t) => t.id === id);
  if (idx !== -1) toastState.toasts.splice(idx, 1);
}

export const toast = {
  info: (m: string) => push("info", m),
  success: (m: string) => push("success", m),
  error: (m: string) => push("error", m),
};
