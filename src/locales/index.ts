import zhCN from "./zh-CN";
import enUS from "./en-US";

/** 支持的语言 */
export const supportedLngs = ["zh-CN", "en-US"] as const;
export type SupportedLng = (typeof supportedLngs)[number];

/** 默认语言（首次启动、后端无偏好时使用） */
export const defaultLng: SupportedLng = "zh-CN";

/** 语言消息包 */
export const messages = {
  "zh-CN": zhCN,
  "en-US": enUS,
};
