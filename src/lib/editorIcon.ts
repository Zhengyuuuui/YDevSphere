/**
 * 编辑器 / 已安装应用图标处理共享 helper。
 *
 * 后端图标为 PNG 的 base64 编码（256px），前端统一转成 data URL 供 <img> 渲染。
 * 无图标返回 undefined（调用方走占位）。与 Settings「手动导入应用」面板逻辑一致。
 */
export function iconSrc(iconBase64?: string | null): string | undefined {
  return iconBase64 ? `data:image/png;base64,${iconBase64}` : undefined;
}
