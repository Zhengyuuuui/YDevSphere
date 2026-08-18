# YDevSphere v0.3 总负责人交付文档

> 交付对象：v0.3 总负责人 / 后续执行 agents
> 交付版本：v0.2 收尾 → v0.3 迭代起点
> 文档目的：替代本次长对话，提供可独立接手项目的上下文、现状、约束和执行入口。
> **重要：本文件所有"现状"均以当前代码为准，已核对代码，不是历史文档的转述。**

---

## 1. 项目定位

YDevSphere 是一个 **local-first AI developer workspace intelligence application**：运行在本地桌面端，扫描、索引并理解开发者的项目资产。

它不是 IDE、代码编辑器、Git 客户端或云端项目管理系统，而是位于以下链路中的 Developer Intelligence Layer：

```text
本地代码 → 项目结构 → 技术栈 → Git 历史 → 项目记忆 → AI 上下文
```

产品长期方向：

- 本地项目扫描与索引
- 项目边界识别、技术栈和健康度分析
- Git 状态与历史分析
- 项目打开和开发环境入口
- 项目长期记忆、AI 分析、MCP / Agent 能力

---

## 2. 当前仓库与 Git 状态

- 本地路径：`/Users/zhengyusheng/Documents/YDevSphere-0.0.1`
- 远程仓库：`https://github.com/Zhengyuuuui/YDevSphere`
- 当前分支：`main`
- 当前 HEAD：`4c86096 chore: complete v0.2 development`
- 远程状态：`main` 已推送至 `origin/main`
- 交接时工作区：干净，无未提交改动

> 注：本次新增的 `docs/V03_HANDOFF.md` 尚未提交，属新增文件。

开始任何开发前先执行：

```bash
git status --short --branch
git --no-pager log -5 --oneline --decorate
```

不要重写历史，不要强制推送，不要在未经用户明确要求时提交代码。

---

## 3. 技术栈与运行方式

### 主工程

- 前端：Vue 3 + TypeScript + Vite + TailwindCSS
- 状态：Pinia
- 路由：Vue Router
- 桌面运行时：Tauri 2
- 后端：Rust
- 数据库：SQLite / `rusqlite` bundled
- IPC：前端统一通过 Tauri `invoke()` 调用后端

### 环境要求

- Node >= 20
- pnpm
- Rust stable

### 常用命令

```bash
pnpm install
pnpm dev
pnpm tauri dev
pnpm build
cd src-tauri && cargo build
cd src-tauri && cargo test
```

`pnpm build` 是前端类型检查和生产构建的基本门禁；Rust 改动必须运行 `cargo build` 与 `cargo test`。

---

## 4. 目录入口（已核对当前代码）

```text
src/pages/                    Welcome / Overview / Projects / Recent / Settings / ProjectDetail
src/components/               AppSidebar、ProjectTable、ProjectCard、OpenActions、DirTree、
                              ActivityChart、StatCard、TechStackList、TechnologyBadge、
                              GitStatusBadge、ToggleSwitch、ToastContainer、EnableMemoryDialog
src/stores/                   editor / git / i18n / layout / memory / project / scanner / settings
src/api/                      editor / git / ignoreRules / language / memory / project / workspaces
src/lib/                      view 适配、排序、最近项目、toast、格式化
src/types/                    前后端数据类型

src-tauri/src/commands/       editor / git / memory / project / workspace
src-tauri/src/core/           纯 Rust 业务核心
  ├── scanner/mod.rs          扫描边界识别（真项目/聚合根/分类目录）
  ├── parser/                 技术栈解析
  ├── database/               connection / crud / migrations
  ├── editor/                 discover / detect / settings / open
  ├── git/analyzer.rs         Git 只读分析
  ├── memory/                 project_memory（.ydevsphere/project.json）
  ├── workspace/              系统工作区（Documents/Desktop）
  └── models/                 所有数据结构
src-tauri/capabilities/       Tauri 权限配置

doc/                          PRD、架构、规格等基础文档
docs/                         开发顺序、前端树、Scanner 方案和审计记录、V03 交接
YDevSphere-v0.2frontend/      v0.2 前端参考/设计目录，修改前确认是否属于主运行链路
```

优先阅读：

1. `docs/DEVELOPMENT_ORDER.md`（全局总览，持续更新）
2. `docs/v0.2-scanner-plan.md`
3. `docs/v0.2-scanner-audit-blockers.md`
4. `docs/v0.2-scanner-bugfix.md`
5. `docs/FRONTEND_TREE.md`
6. `doc/prd.md`、`doc/architecture.md`、`doc/spec.md`

---

## 5. v0.2 已完成内容（已核对代码）

### 前端

- Vue 3 + Tauri 2 基础工程、响应式 App Shell（Sidebar 220px / 72px 两档）
- 页面：Welcome / Overview / Projects / Recent / Settings / ProjectDetail
- Projects：搜索、排序、列显隐、技术栈单行、窄屏 More 操作
- 中英文语言包、全局 Toast、布局状态持久化
- 多工作区集合模型（Documents + Desktop + 手动目录并存）

### 后端 Scanner（**已实现，非"待做"**）

- `scanner/mod.rs` 四类项目边界识别：真项目 / 聚合根 / 分类目录 / 普通目录
- 深度上限默认 6 层（`ScanOptions.max_depth`）
- 预设忽略 + 用户自定义忽略（`ignore_dirs` 持久化）
- `health_score` 健康度评分
- 聚合根父子关系：子项目带 `parent_id` 入库；`get_projects` 默认只返回顶层（`parent_id IS NULL`），子项目经 `parent_id_filter` 或 `get_dir_children` 按需获取
- `delete_missing_projects` 清理磁盘已删除项目（含父/子级联）
- `list_dir_children` 懒加载目录树

### 编辑器（**v0.3 核心关注点，见 §7**）

- 白名单 + 动态发现合并：`list_available_editors()`
- 三档检测：product.json 指纹（VS Code Fork）/ Info.plist 兜底 / 白名单
- 打开方式：`cli` / `open_a` / `unsupported`
- 编辑器发现缓存：`~/.ydevsphere/settings.json` 的 `editor_cache`
- 默认编辑器偏好持久化

### 其他

- Git 只读分析、文件管理器打开（tauri-plugin-opener）
- 项目记忆 `.ydevsphere/project.json` 基础能力
- `~/.ydevsphere/settings.json` 持久化：default_editor / workspace_path / workspaces / ignore_dirs / language / editor_cache

---

## 6. v0.3 首要工作方向

### P0：先验证 v0.2 真实闭环

在设计新功能前，用 `pnpm tauri dev` 做人工 GUI 验收：

- Welcome → 一键导入 Documents / Desktop / 手动选择
- 多工作区恢复与切换
- Projects 页面筛选、搜索、排序
- 真实扫描、扫描状态、错误提示
- 项目详情跳转、最近打开、Git 信息加载
- 默认编辑器和文件管理器打开
- 设置页持久化
- 窄窗口 / 宽窗口布局

### P0：编辑器发现误判治理（本次对话核心诉求，见 §7）

这是 v0.3 最优先项，详见 §7 的完整方案与约束。

### P1：Scanner 继续完善（多数"审计项"已落地，核验剩余）

`docs/v0.2-scanner-audit-blockers.md` 中**多数项已在当前代码落地**（聚合根 parent_id、顶层过滤、get_dir_children、get_ignore_rules、delete_missing_projects 级联、health_score 依据清单）。接手时应：

1. 对照审计文档逐项**复核现状**，已实现的不重复返工。
2. 补全聚合根父子删除 / 同步清理的级联**单测覆盖**（`crud.rs` 已有部分，确认完整）。
3. 健康度"+20 清单分"确认基于实际清单存在，而非仅 `ProjectKind::Real`。
4. 保持 `core/` 无 `use tauri`。

### P1：文件监听

Scanner 稳定后再做文件监听 / 增量更新，避免在边界语义未稳定前叠加复杂度。

### P2：Overview 真实统计

`ActivityChart.vue` 目前使用 mock；需要后端新增 `get_stats` 聚合接口，至少支持：

- commits/week 活动图
- 本周活动统计
- 与 Overview 统计卡联动

### P2：AI 项目分析 / 记忆 / MCP

依赖先确定：AI Provider、API Key 存储策略、可发送的数据范围、本地隐私边界、项目记忆数据结构和更新时机、MCP Server / Agent 调用协议。未明确隐私和 Provider 方案前不接入外部 AI 服务。

---

## 7. 编辑器打开方式与发现误判治理（**v0.3 核心，用户最新诉求**）

### 7.1 现状与问题（已核对代码）

**后端**：`core/editor/discover.rs` 的 `detect_app()` 三档检测：

- product.json 指纹（`applicationName` 存在）→ 判定 VS Code Fork，`cli` 打开（最可靠）
- Info.plist 兜底：声明 `public.folder` → `open_a`；否则 → `unsupported`
- **关键缺陷**：`discover_editors()` 不丢弃 `unsupported` 的 app（只要有 bundleId），全部进入 `list_available_editors()` 返回，并写入 `editor_cache`。

**前端**：`src/pages/Settings.vue` 下拉框 `v-for editorStore.editors`，对 `open_method === 'unsupported'` 仅 `:disabled` 置灰，**仍全部展示**。

**结果**：/Applications 里任何"有 bundleId 但非编辑器"的 app（如 Burp Suite、Docker、Figma、WeChat、Clash Party、剪映、Chrome 应用等）都会被列进下拉框，堆满大量非编辑器。

### 7.2 v0.3 编辑器方案（用户拍板方向）

产品哲学：**不替用户猜测，把选择权交给用户；明确不是编辑器的给提醒。**

1. **默认 = VS Code Fork 系**：product.json 指纹识别的最可靠（Cursor / Trae / Qoder / CodeBuddy / WorkBuddy 等）。
2. **用户主动选择编辑器**：不自动猜测，让用户从"过滤后的干净列表"里选自己的编辑器 app。
3. **明确非编辑器的提醒**：对 Info.plist 兜底误判出来的"可能不是编辑器"，给用户显式提醒（如"此应用可能不是编辑器"）。
4. **v0.3 Welcome 引导**：在 Welcome 页"导入工作区（Documents / Desktop）"之后，新增一步"选择你常用的编辑器"引导。

### 7.3 推荐落地方案（待 v0.3 执行）

**第一步（立即修，收紧发现）：**

- `discover_editors()` / `list_available_editors()` **过滤掉 `open_method = Unsupported` 的项**，下拉框只展示"能打开"的编辑器（`cli` + `open_a`）。
- 收紧 Info.plist 兜底：仅匹配"明确编辑器关键词"且可打开的 app 才列出；unsupported 一律不进列表。
- 前端 Settings 下拉框同步：不再渲染 unsupported 项（而不是 disabled 置灰）。

**第二步（v0.3 引导，用户已规划）：**

- Welcome 导入工作区后加"选择常用编辑器"引导，展示过滤后的干净列表。

**需要 v0.3 确认的决策点：**

- unsupported 的 app 是否彻底不列（推荐不列，因为打不开且堆满下拉框）？
- ChatGPT / Claude（`open_a`，能打开文件夹但非编辑器）是否移除（推荐移除）？
- 动态发现最终收敛为：product.json 一定列 + Info.plist 仅匹配明确编辑器关键词且可打开才列？

### 7.4 相关代码位置

- 后端：`src-tauri/src/core/editor/discover.rs`（`detect_app` / `discover_editors` / `build_infoplist_fallback`）
- 后端：`src-tauri/src/core/editor/detect.rs`（`list_available_editors` / `is_available_editor`）
- 后端：`src-tauri/src/core/editor/settings.rs`（`editor_cache` 持久化）
- 后端：`src-tauri/src/core/models/editor.rs`（`OpenMethod` / `EditorSource` / `EditorCategory` / `AvailableEditor`）
- 命令：`src-tauri/src/commands/editor.rs`（`list_editors` / `rescan_editors` / `open_in_editor`）
- 前端：`src/pages/Settings.vue`（下拉框渲染）
- 前端：`src/stores/editor.ts`（editorStore）
- 前端：`src/api/editor.ts`、`src/types/editor.ts`
- 前端：`src/pages/Welcome.vue`（后续加引导步骤）

---

## 8. 已知问题与历史上下文

### 多工作区数据问题

历史上出现过：

- Documents 项目存在数据库，但旧数据 `workspace` 为 `NULL`
- 单值 workspace 模型导致 Documents / Desktop 互相覆盖
- `importAll()` 曾存在最后一个工作区覆盖前一个的问题
- 曾出现 `menuEl.value.contains is not a function`

**当前已改为多工作区集合**：`~/.ydevsphere/settings.json` 的 `workspaces` 数组是权威源，`workspace_path` 仅作冗余镜像；`get_workspaces` 支持旧单值迁移。处理相关问题时以当前 `src/` 和 `src-tauri/src/core/editor/settings.rs` 为准。

### 待 GUI 实测的 P2 项

见 `docs/BACKLOG-P2P3.md`：

- 无效目录触发 `INVALID_DIRECTORY` 的 Tauri → 前端错误链路
- 旧 localStorage 工作区迁移到后端后，清理 localStorage 并重启恢复

### 技术债

- 其他 command 的裸字符串错误统一结构化
- 给 `ScanCommandError` 实现 `std::error::Error`
- 编辑器执行迁移到 `tauri-plugin-shell` 权限模型
- 发布一个周期后移除 localStorage 过渡代码
- 自定义工作区分类
- 语言包 key / 占位符自动校验脚本

---

## 9. 不可破坏的架构约束

1. `src-tauri/src/core/` 禁止依赖 Tauri，保持纯 Rust，可供未来 CLI / MCP 复用。
2. `src-tauri/src/commands/` 只做参数解析、状态获取和转发，不放业务逻辑。
3. 前端禁止直接访问文件系统、执行 shell、操作 SQLite；统一通过 `invoke()`。
4. 默认 Read Only；持久化只允许写入 `~/.ydevsphere/` 下的数据库、`settings.json`、`project.json` 等应用数据。
5. 必须跨平台，不写 macOS-only 业务逻辑。
6. 编辑器打开必须经过白名单 / 已知编辑器校验，未知 editor id 拒绝执行。
7. Scanner 的父项目边界优先规则必须有代码注释、文档和单测共同保证。
8. 新增 IPC 时同步更新 Rust model、command、前端 API、TS 类型、store、页面和测试。
9. 不要为了 v0.3 重写大文件；优先小范围修改并保留已完成 v0.2 行为。

---

## 10. 推荐接手流程

### 第一步：建立真实基线

```bash
pnpm install
pnpm build
cd src-tauri && cargo test
cd .. && git status --short --branch
```

### 第二步：确认代码现状（不要依赖历史文档假设）

```bash
git --no-pager log -5 --oneline --decorate
find src-tauri/src -maxdepth 3 -type f | sort
find src -maxdepth 3 -type f | sort
```

重点核对：

- 编辑器：`discover_editors` / `detect_app` / `build_infoplist_fallback` / `list_available_editors` / `editor_cache` / Settings 下拉框
- Scanner：`classify_dir` / `health_score` / `delete_missing_projects` / `list_dir_children` / `get_ignore_rules` / `set_ignore_rules` / `parent_id`
- 工作区：`workspaces` 集合 / `get_workspaces` / `set_workspaces` / `importAll` / `menuEl`

### 第三步：优先顺序建议

1. 人工 GUI 回归
2. **编辑器发现误判治理（§7，本次核心）**
3. Scanner 审计项复核（多数已落地，补单测）
4. 多工作区和旧数据迁移回归
5. 文件监听
6. Overview `get_stats`
7. AI / MCP 方案设计与实现

每个阶段完成后都更新 `docs/DEVELOPMENT_ORDER.md` 的全局总览，并追加交付记录，不覆盖历史记录。

---

## 11. 总负责人的角色与边界

本项目采用**双角色协作**：总负责人 + 执行 agent。角色定位清晰，避免越权。

### 11.1 总负责人是谁

总负责人是 v0.3 的**规划者、调度者、验收者**，**不是写代码的执行者**。它：

- 阅读本交接文档和专项文档，建立全局认知。
- 把 v0.3 目标拆解成可执行的任务单（prompt），**下发给执行 agent**。
- 对每个执行结果做验收核对（跑测试、看 diff、复核是否违反约束红线）。
- 维护 `docs/DEVELOPMENT_ORDER.md` 与交付记录，保持可追溯。

### 11.2 总负责人 vs 执行 agent 的分工

| 角色 | 职责 | 不做什么 |
|---|---|---|
| **总负责人** | 规划、拆解任务、生成任务单、验收、维护总览 | 不直接改代码、不直接跑写操作 |
| **执行 agent** | 依据任务单实现代码、跑测试、处理报错 | 不擅自扩大范围、不擅自做方向性决策 |
| **用户（你）** | 最终决策者、授权者、验收拍板 | 需要掌握所有写操作和方向性改动的最终决定权 |

### 11.3 权限边界（红线）

1. **修改代码必须先获得用户授权**：任何对代码的修改（改文件、新建文件、删除文件），都必须先征得用户明确同意。未授权前只做只读勘察、规划、生成任务单。
2. **写操作默认禁止**：未授权不执行提交、推送、安装依赖、运行可能产生副作用或写用户文件的命令。
3. **总负责人不越权改码**：即使总负责人本身有改码能力，也应把改动方案提交用户审阅、或派发给执行 agent，而不是自己动手。
4. **用户是最终决策者**：所有方向性改动（编辑器发现方案、隐私边界、AI Provider、数据范围等）必须用户拍板后才能进入实现。
5. **授权是每次性的**：一次授权只覆盖对应任务，新任务需重新确认；不默认"授权一次全程生效"。

### 11.4 边界内的自主决策

在**不涉及写操作、不改变方向**的前提下，总负责人可以自主决定：

- 只读勘察用哪些命令 / 读哪些文件。
- 任务优先级与拆解粒度。
- 任务单（prompt）的写法与分发方式。
- 验收核对的具体步骤。

一旦进入"要改代码 / 要写文件 / 要定方向"，立即回到 §11.3 的授权流程。

---

## 12. 交接结论

v0.2 已完成收尾并推送到 GitHub；Scanner 的边界识别、聚合根父子关系、清理、目录树、忽略规则等**核心能力已在当前代码落地**（不再是"待做"）。

v0.3 的核心任务：

1. **编辑器发现误判治理**：过滤 unsupported 项、收紧 Info.plist 兜底、下拉框只展示可打开的编辑器，并在 Welcome 导入后加"选择常用编辑器"引导（用户本轮拍板方向，§7）。
2. 验证并稳定 Scanner、多工作区和真实桌面闭环。
3. 逐步推进文件监听、Overview 统计、AI 记忆与 MCP。

本文件是 v0.3 总负责人的第一阅读入口；具体实现以当前代码为准，专项文档作为设计和历史依据。

---

## 13. 给 v0.3 的接手 Prompt（新对话直接复制）

> **前置：如何建立"总负责人 + 执行 agent"的工作模式**
>
> 本项目采用**双角色协作**：总负责人负责规划、分发任务、验收，本身**不直接改代码**；执行 agent 才动手写代码，且修改前必须获得用户授权。新对话可先以"总负责人"身份启动，需要落地代码时再派发执行 agent（或让用户在授权后切换执行身份）。

新对话开始时，直接把这整段发给新的 agent，即可无缝接管：

> 请先完整阅读 `docs/V03_HANDOFF.md`。你的角色是 **YDevSphere v0.3 总负责人**，职责边界见文档 §11（角色与权限）。
>
> **你的职责（总负责人）**：
> - 规划 v0.3 任务、拆解优先级、生成执行任务单（prompt）。
> - 把"改代码"的任务分发给执行 agent，你本身**不直接修改代码**。
> - 对每个执行结果做验收核对（跑测试、看 diff、复核是否符合约束红线）。
> - 维护 `docs/DEVELOPMENT_ORDER.md` 与交付记录，保持可追溯。
>
> **你的权限边界（红线）**：
> - **任何对代码的修改，都必须先征得用户明确授权**，未授权前只做只读勘察、规划、生成任务单。
> - 未经授权不执行任何写操作（改文件、提交、推送、安装依赖、运行可能产生副作用或写用户文件的命令）。
> - 不以"总负责人"身份绕过授权自行改代码；把改动方案先给用户审阅。
> - 用户是最终决策者；所有方向性改动（编辑器方案、隐私边界、AI Provider 等）需用户拍板。
>
> 接手原则：
> 1. 以**当前代码为唯一真实依据**，不要信任历史文档里"待做 / 未实现"的说法——多数 Scanner 能力已在代码落地，先复核再动手，避免重复返工。
> 2. 先跑基线（只读/无副作用命令）：`pnpm build` → `cd src-tauri && cargo test` → `git status --short --branch`。
> 3. 不要重写大文件；优先小范围修改，保留 v0.2 已完成行为。
>
> v0.3 优先顺序：
> 1. **P0 编辑器发现误判治理（文档 §7）**：过滤 `unsupported` 项、收紧 Info.plist 兜底、下拉框只展示可打开的编辑器，并在 Welcome 导入工作区后加"选择常用编辑器"引导。
> 2. 人工 GUI 回归（导入工作区、多工作区切换、Projects 筛选/搜索/排序、真实扫描、项目详情、Git 加载、默认编辑器与文件管理器打开、设置持久化、窄/宽布局）。
> 3. Scanner 审计项复核（多数已落地，补全聚合根父子删除的级联单测）。
> 4. 多工作区和旧数据迁移回归。
> 5. 文件监听。
> 6. Overview `get_stats`。
> 7. AI 记忆 / MCP（先定 Provider 与隐私边界）。
>
> 约束红线：
> - `src-tauri/src/core/` 禁止 `use tauri`；`commands/` 只做转发不放业务逻辑。
> - 前端统一走 `invoke()`，禁止直连文件系统 / shell / SQLite。
> - 默认 Read Only，只写 `~/.ydevsphere/` 下的应用数据。
> - 编辑器打开必须经过已知编辑器校验，未知 id 拒绝执行。
> - 新增 IPC 时同步更新 Rust model / command / 前端 API / TS 类型 / store / 页面 / 测试。
>
> 每个阶段完成后更新 `docs/DEVELOPMENT_ORDER.md` 全局总览并追加交付记录（不覆盖历史）。遇到不确定的现状，先用只读工具核对代码，不要猜。

### Prompt 风格说明

- **开头点名身份与定位**：让 agent 明确自己是"v0.3 总负责人"，并指引其先读 §11 角色与边界。
- **先给"职责 + 权限边界"**：讲清总负责人"规划/分发/验收、不改码"的职责，以及"改码前必须获授权"的红线，从源头防止越权。
- **再给"接手原则"**：强调以代码为准、先复核、不返工。
- **再给"优先顺序"**：P0/P1/P2 分层，先做核心再做扩展。
- **再给"约束红线"**：不可破坏的架构边界，防止 agent 跑偏。
- **最后给"交付纪律"**：每个阶段更新总览并记录，保持可追溯。

### 边界使用说明

- **"总负责人"与"执行 agent"是同一模型的两个状态**：同一套模型既可当总负责人（规划/分发），也可当执行 agent（改码）。本文档用 prompt 显式约束当前状态，避免"既当裁判又当运动员"。
- 切换方式：新对话以总负责人身份启动 → 需要改码时，生成任务单 → 用户授权后，可派发执行 agent（或用户授权下切换执行身份）→ 结果回传总负责人验收。
- 所有写操作的决定权始终在用户；总负责人负责把"要改什么、为什么、影响面"讲清楚，等用户拍板。
