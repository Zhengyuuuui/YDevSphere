# YDevSphere · P2-P3 非阻塞遗留项 Backlog

> 维护人：总负责人 · 用途：统一跟踪**非阻塞**（P2/P3 级）技术债与待验证项
> 说明：阻塞级（P0/P1）问题不进本清单，直接在 `DEVELOPMENT_ORDER.md` 或专项文档跟踪。
> 本清单项**不阻塞当前版本交付**，按优先级/需要择机处理。

---

## 一、待验证项（需 GUI 实测）

### P2-1 Tauri 错误码序列化 E2E 实测（🟡-1 from V02-ERR-CODE）
- **状态**：待实测
- **背景**：`scan_projects` 已改为返回结构化错误 `ScanCommandError { code, message }`。前端 `parseError` 已有双保险（Tauri 若将错误降级为字符串，前端 JSON.parse 兜底提取 code）。但「Tauri command reject → 前端 invoke catch」的**运行时链路**尚未用 `pnpm tauri dev` 实测。
- **实测步骤**：
  1. `pnpm tauri dev` 启动。
  2. 把某工作区路径改为**无效目录**（不存在/非目录）。
  3. 触发「扫描」。
  4. 预期：toast 提示「工作区目录已失效」→ 跳回 Welcome（`errorCode === "INVALID_DIRECTORY"` 命中）。
  5. DevTools console 查看 `scanner.errorCode` 是否为 `"INVALID_DIRECTORY"`。
- **验收**：功能正确（跳回 Welcome）；确认走「结构化」还是「JSON 兜底」路径。
- **优先级**：P2（有兜底，不阻塞，但需验证闭环）

### P2-2 升级迁移 GUI 实测（from V02-WS 审计）
- **状态**：待实测
- **背景**：工作区集合权威源已切到后端 `settings.json`，localStorage 仅过渡兜底。需实测「旧 localStorage 数据 → 推后端 → 清 localStorage → 重启从后端恢复」。
- **实测步骤**：
  1. 在 localStorage（`ydevsphere.workspaces`）预置 Documents + Desktop。
  2. 启动新版 → 确认集合推入后端、localStorage 被清除。
  3. 重启 → 从后端恢复，默认编辑器偏好不变。
- **优先级**：P2

---

## 二、技术债（可选，无紧迫需求）

### P3-1 其他 command 错误统一结构化（🟢-1 from V02-ERR-CODE）
- **状态**：待做（可选）
- **背景**：当前仅 `scan_projects` 用 `ScanCommandError` 结构化错误。`get_projects` / `get_project_detail` / `get_scan_history` 等仍返回 `Result<_, String>`（裸字符串）。
- **触发条件**：未来若这些接口需前端按错误分支（如「数据库损坏」）时再统一。
- **建议**：暂不做，避免过度设计。

### P3-2 `ScanCommandError` 补 `impl std::error::Error`（🟢-2 from V02-ERR-CODE）
- **状态**：待做（一行）
- **背景**：`ScanCommandError` 已 `Serialize` + `Display`，但缺 `impl Error`，限制其在 `Box<dyn Error>` 等泛型场景复用。
- **改动**：`impl std::error::Error for ScanCommandError {}`。
- **建议**：可顺手在下次后端改动时一并补。

### P3-9 `loadEditors` 并发竞态（from V03-SELECT-MISSING 排查）
- **状态**：待做（可选）
- **背景**：排查「下拉框不显示手动导入」时发现，`stores/editor.ts` 的 `loadEditors()` 无请求序号/防抖。Settings 手动导入路径（`importApp`）与 `init()` 若同时触发两个 `loadEditors`，后发起者若延迟 resolve，可能被先发起的旧结果覆盖，导致手动导入项偶发不显示。
- **建议**：`loadEditors` 内加请求序号/防抖，或改为「最后发起的请求结果为准」。属次要，当前 `confirmCustom` 补 `loadEditors` 已覆盖主路径，可择机做。

---

## 三、历史遗留（早期记录，仍有效）

### P3-3 编辑器执行迁移到 `tauri-plugin-shell` 权限模型
- **状态**：待做
- **背景**：`open_in_editor` 用 `std::process::Command::spawn` 直接启动（不走 Tauri 插件权限系统）。当前因「白名单 + 无 shell + 只 spawn 已知路径」风险可控。
- **建议**：迁移到 `tauri-plugin-shell`，纳入 Tauri 能力边界。

### P3-4 localStorage 过渡代码移除
- **状态**：保留至少一个发布周期
- **背景**：`readLegacyWorkspaces` / `clearLegacyWorkspaces` 过渡逻辑。为防「长期未启动用户」首次升级丢数据，**保留一个发布周期后再移除**。
- **建议**：发布周期后删除，并同步清理相关代码。

### P3-5 文件监听（P1-4）
- **状态**：未做（v0.2+）
- **背景**：PRD P1-4，项目变化自动更新索引，作为 Scanner 迭代后的增量维护补充。

### P3-6 AI 项目分析 / 记忆 / MCP（P2）
- **状态**：未做
- **背景**：需先定 AI Provider / API Key / 数据范围。

### P3-7 自定义工作区分类（v0.2 规划）
- **状态**：未做
- **背景**：筛选标签栏已预留横向滚动扩展。

### P3-8 Overview 图表 mock 待接真实接口（`get_stats`）
- **状态**：待接
- **背景**：活动图（commits/week）、本周活动统计用 mock，需后端新增 `get_stats` 聚合接口。

---

## 四、语言包完整性检查脚本（长期建议）

### P4-1 语言包 key 完整性校验脚本（from V02-I18N 审计）
- **状态**：待做（可选）
- **背景**：本次审计用「zh/en key 集合 + 占位符一致性」人工校验。建议沉淀为 CI 脚本，防止后续新增文案漏译/占位符错配。
- **建议**：可在 CI 里跑「zh/en key 集合 + 占位符一致性」校验。

---

## 五、v0.4 规划

> v0.4 拆两条线：**a. 识别迭代（当前主线，先做）** + **b. 商业化 License（归档，延后）**。
> 定价方向未定，先不急；`v0.4b` 以下为规划存档，非本期执行项。

### v0.4a 识别迭代（主线，先做）
- **目标**：增强**项目/技术栈识别**，支持**多技术栈 / 全栈分区**。
- **痛点实例**：`~/Desktop/藏蓝闪送`——前端(Vue/Node) + 后端(Express/SQLite) 全栈项目，当前只能打扁平 `language`+`framework`，无法表达「前端架构」+「后端架构」。
- **方向**：
  - 一个项目识别出**多个技术栈分区**（前端 / 后端 / 子包），而非单一 language+framework。
  - 结合已有聚合根 / 子项目模型，识别项目内部 `frontend/` + `backend/` 子目录并归组。
  - `ProjectDetail` 展示「前端：Vue+Node / 后端：Express+SQLite」分区。
- **状态**：待排期（v0.4 主线，先做）。

### v0.4b 商业化 License（归档，延后）
- **目标**：**买断制**授权 + **反逆向加密**。
- **定位**：面向**普通用户 / 个人开发者**，本地优先、不引入 AI 能力（无云端）。
- **待定**：定价区间（不急，后续定）。
- **关键取舍 / 待办**：
  - 基础 License 校验：机器码 + key 签名校验（防简单复制分发）。
  - 授权基础设施：key 生成、机器绑定、离线/在线校验、试用期、升级策略。
  - **现实预期管理**：纯本地客户端无法 100% 防逆向（Rust 难逆向但非不可能）；真正保护依赖「服务端授权 + 核心价值放云端」。当前纯本地无云端，加密只能**增加逆向成本**，不能彻底阻止。
  - **建议**：v0.4 先做基础 License 校验，**不过度投入**高级反逆向（性价比低）；付费价值做成「高级功能」，未来引入 AI/云同步时天然难被完全本地破解。
- **状态**：归档待办（定价定后启动）。

---

## 处理原则

- 本清单项**均不阻塞当前版本**。
- 每项含：触发条件 / 建议 / 优先级。
- 处理某项时，勾选标记并从「待做」改「已处理」。
- 由总负责人决定处理时机，不主动排队（除非用户要求）。
