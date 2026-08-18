# YDevSphere · DEVELOPMENT_ORDER

> 本文档由**总负责人统一维护**：顶部「全局总览」为项目开发入口；各执行 agent 完成 Sprint 后须回填对应状态。

---

# YDevSphere · 全局总览

> 维护人：总负责人 · 最后更新：v0.2 响应式布局完成（V02-RESPONSIVE）

## 一、项目里程碑总览

| Sprint | 内容 | 状态 |
|---|---|---|
| S1 | 基础工程（Tauri2 + core/ 分层 + 数据库骨架 + IPC 桩） | ✅ 完成 |
| S2 | Scanner（扫描 + 解析 + 数据库 CRUD + 原生选择器 + 前端页面 + logo） | ✅ 完成 |
| S3 | Dashboard 体验增强（搜索/排序/最近打开/扫描反馈 + 后端排序支撑） | ✅ 完成 |
| S4 | 项目记忆 `.ydevsphere/project.json` | ✅ 完成 |
| S5-01 | Git 分析（P1-1，只读） | ✅ 完成 |
| S5-02 | 项目打开（编辑器白名单 + 文件管理器 + 默认编辑器偏好） | ✅ 完成 |
| S5-03 | 工作区路径持久化（重启自动恢复，不再手动选择） | ✅ 完成 |
| S5-04 | v0.1 Plus：一键导入 Documents/Desktop 工作区 | ✅ 完成 |
| S5-05 | v0.1 Plus2：Dashboard 工作区筛选（全部/Documents/Desktop + 横向滚动标签） | ✅ 完成 |
| S6 | v0.2 前端重构：Figma 设计 → Vue3（Sidebar + Overview/Projects/Recent/Settings） | ✅ 完成 |
| S7 | v0.2 响应式布局：App Shell + Projects 三态（Sidebar 220/72 + ProjectTable GRID/列显隐/Tech 单行/Small More） | ✅ 完成 |

## 二、当前状态

- 开发位置：**v0.2 响应式布局已完成**（App Shell + Projects），进入后续迭代阶段。
- 版本：v0.2（前端重构 + 响应式布局：Layout Mode 三态，窗口 852/1072 切换自适应，保留全部现有功能）。
- `pnpm build` 0 类型错误；`pnpm dev` 正常。
- 下一步：scanner 逻辑迭代（主要）、文件监听、AI 分析、编辑器执行迁移、自定义分类。

## 三、关键架构约束（已落地）

- `core/` 无 `use tauri`；`commands/` 仅做适配转发。
- 前端不直接访问文件系统 / 命令 / 数据库，一律走 `invoke()`。
- Read Only 默认，仅写 `~/.ydevsphere/`（数据库 + settings.json + project.json）。
- 跨平台，无 macOS-only 逻辑。
- 编辑器打开采用白名单执行，未知 id 拒绝。

## 四、全局遗留项（跨 Sprint 统一跟踪）

- [x] 工作区路径持久化（重启自动恢复）。
- [x] 编辑器偏好持久化（跨重启生效）。
- [x] 工作区筛选（Documents/Desktop/全部）。
- [x] v0.2 前端重构（Sidebar + Overview/Projects/Recent/Settings，保留全部功能）。
- [ ] Overview 图表 mock 待接后端接口：活动图（commits/week）、本周活动统计（需新增 `get_stats` 聚合接口）。
- [ ] **v0.2 Scanner 迭代**（方案已定稿 `docs/v0.2-scanner-plan.md`）：智能项目边界 + 健康度评分 + 按需懒加载树 + 同步清理 + 忽略规则。
- [ ] 端到端人工 GUI 核验（真实扫描→打开编辑器→记忆全流程）。
- [ ] 编辑器执行迁移到 `tauri-plugin-shell` 权限模型（技术债，白名单已控风险）。
- [ ] 文件监听（P1-4，v0.2）。
- [ ] AI 项目分析 / 记忆 / MCP（P2，v0.2）。
- [ ] 自定义工作区分类（v0.2，筛选标签栏已预留横向滚动扩展）。
- [ ] Dashboard 可选增强（搜索历史、排序记忆）。

## 五、文档维护规范

- 本文档「全局总览」章节由总负责人维护，作为项目入口。
- 各执行 agent 完成 Sprint 后，必须回填对应 Sprint 状态、当前 Sprint、全局遗留项。
- 各 agent 的详细交付记录追加在文档后续章节，不覆盖全局总览。

---

## 一、Sprint 1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| 工程骨架 | ✅ 完成 | Tauri 2 + Vue3 + TypeScript + Rust |
| Rust core 分层 | ✅ 完成 | `core/{scanner,parser,database,models}`，`core/` 无 `use tauri` |
| 数据库骨架 | ✅ 完成 | SQLite 连接初始化 + `projects` / `scan_history` 建表迁移 |
| 4 个 IPC command 桩 | ✅ 完成 | `select_workspace` / `scan_projects` / `get_projects` / `get_project_detail` |
| 前端基础设施 | ✅ 完成 | Vue Router 4 路由 + Pinia 3 store + TailwindCSS + invoke 封装 |
| 类型对齐 | ✅ 完成 | 前端 TS 类型与 `core/models` 对齐 |
| 扫描 / 解析 / 数据库 CRUD | ⏳ 留 Sprint 2 | 仅占位，不含真实逻辑 |

**Sprint 1 边界遵守**：不实现扫描逻辑、AI 分析、项目解析、真实数据库读写；前端不直接访问文件系统 / 执行命令 / 操作数据库；后端默认 Read Only；无 macOS-only 业务逻辑。

---

## 二、目录结构说明

```
YDevSphere-0.0.1/
├── src/                        # Vue3 前端
│   ├── main.ts                 # 应用入口（Pinia + Router + 样式）
│   ├── App.vue                 # 根组件
│   ├── styles.css              # Tailwind 指令
│   ├── router/index.ts         # 4 路由（/ /dashboard /project/:id /settings）
│   ├── pages/                  # Welcome / Dashboard / ProjectDetail / Settings
│   ├── components/             # AppNav 等
│   ├── stores/                 # project.ts / settings.ts / scanner.ts（Pinia）
│   ├── api/                    # invoke() 封装：selectWorkspace/scanProjects/getProjects/getProjectDetail
│   └── types/                  # 前端 TS 类型（对齐 core/models）
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── main.rs             # 二进制入口
│   │   ├── lib.rs              # 库入口：初始化 core + 注册 command
│   │   ├── core/               # 纯业务核心（禁止 use tauri）
│   │   │   ├── scanner/        # 占位（Sprint 2 实现）
│   │   │   ├── parser/         # 占位（Sprint 2 实现）
│   │   │   ├── database/       # connection + migrations 骨架
│   │   │   └── models/         # Project / ProjectDetail
│   │   └── commands/           # 薄壳层：workspace.rs / project.rs
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   └── icons/                  # 应用图标（tauri icon 生成）
├── docs/                       # 本交付文档
├── doc/                        # 规格文档（spec/architecture/prd）
├── package.json / pnpm-workspace.yaml / vite.config.ts / tsconfig.json
└── README.md
```

**分层约束落地**：
- `core/` 仅依赖 `serde` / `rusqlite` / `dirs`，**无 `tauri` 依赖**，为未来 CLI / MCP 共享 core 做准备。
- `commands/` 仅做参数解析 + 转发，不含业务逻辑。
- `lib.rs` 负责依赖注入（`Mutex<Database>`）与 command 注册。

---

## 三、技术选型决策

| 项 | 决策 | 说明 |
|---|---|---|
| 数据库驱动 | `rusqlite`（bundled） | 比 `sqlx` 更轻量，无需 async runtime；Sprint 1 仅建表，足够 |
| 线程安全 | `Mutex<Database>` | `rusqlite::Connection` 非 `Sync`，用 Mutex 满足 Tauri state 约束 |
| 目录选择 | 占位（当前目录） | 真实原生 picker（`tauri-plugin-dialog`/`rfd`）留 Sprint 2 |
| pnpm 11 构建脚本 | `pnpm-workspace.yaml` `allowBuilds` | 显式允许 esbuild / vue-demi 执行 install 脚本 |

---

## 四、运行命令

前置：Node >= 20，Rust stable，已安装 `pnpm tauri` CLI。

```bash
# 安装依赖
pnpm install

# 仅启动前端 dev server（http://localhost:1420）
pnpm dev

# 启动完整桌面应用
pnpm tauri dev

# 生产构建
pnpm build              # 前端
cargo build             # Rust（在 src-tauri/ 下）

# 运行后端单元测试（含数据库迁移测试）
cargo test              # 在 src-tauri/ 下
```

---

## 五、待办（Sprint 2）

- [x] **Scanner**：`core/scanner` 实现目录遍历、项目识别（package.json / Cargo.toml / go.mod / requirements.txt / pyproject.toml）、忽略规则（node_modules / .git / target / dist / build / vendor / .cache）
- [x] **Parser**：`core/parser` 实现技术栈解析（Vue / React / Node / Go / Rust / Python 等）
- [x] **数据库 CRUD**：`core/database` 实现 `projects` / `scan_history` 的增删改查业务
- [x] **原生目录选择器**：接入 `tauri-plugin-dialog`，替换 `select_workspace` 占位
- [x] **IPC 真实闭环**：`scan_projects` 扫描后写入数据库，`get_projects` 从库读取渲染
- [ ] **Frontend 交互完善**：Dashboard 项目卡、搜索/排序、扫描按钮状态（由前端 agent 负责）

---

## 六、Sprint 2 · 后端交付（任务 SPRINT2-01Backend）

> 交付文档 · 任务编号：`SPRINT2-01Backend`

### 6.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `core/models` | ✅ 完成 | 补齐 `Project` / `ProjectDetail` / `ScanResult` / `ScanHistory` / `DetectedProject` |
| `core/scanner` | ✅ 完成 | 递归扫描 + 项目识别 + 忽略规则；只读 |
| `core/parser` | ✅ 完成 | package.json（Vue/React/Next/Nuxt/Svelte/Angular）/ Cargo.toml / go.mod / pyproject.toml / requirements.txt |
| `core/database` CRUD | ✅ 完成 | `upsert_projects`（批量事务）/ `get_projects` / `get_project_detail` / `insert_scan_history` |
| `commands/` 4 命令 | ✅ 完成 | 真实 IPC 闭环，替换 Sprint 1 桩 |
| 原生目录选择器 | ✅ 完成 | `tauri-plugin-dialog` 阻塞式目录选择 |
| Rust 单测 | ✅ 通过 | `cargo test` 20 passed, 0 failed |

**分层约束**：`core/` 无 `use tauri`；`commands/` 仅做参数解析与转发。

### 6.2 4 个 Command 输入输出签名（供前端对接）

> 前端 `src/api/project.ts` 需按此对齐。参数命名遵循 Tauri 2 规则：Rust 参数 `workspace_path` → 前端传 `workspacePath`；`project_id` → `projectId`。

| Command | Input | Output |
|---|---|---|
| `select_workspace` | 无 | `Option<String>`：用户选择的绝对路径；取消返回 `null` |
| `scan_projects` | `workspace_path: String` | `ScanResult { projects: Project[], history: ScanHistory, scanned_count: number, ignored_count: number }` |
| `get_projects` | 无 | `Project[]` |
| `get_project_detail` | `project_id: number` | `ProjectDetail \| null` |

**返回结构体字段**（`core/models`，前端 TS 需同步）：
- `Project`: `{ id, name, path, language, framework, created_at, updated_at }`
- `ProjectDetail`: `{ id, name, path, language, framework, created_at, updated_at, file_count, last_scan_at }`
- `ScanHistory`: `{ id, workspace, scan_time, status }`
- `ScanResult`: `{ projects, history, scanned_count, ignored_count }`

> ⚠️ 与 Sprint 1 差异（前端需适配）：
> 1. `scan_projects` 返回值由 `Project[]` 变为 `ScanResult`（项目列表在 `.projects` 中）。
> 2. `select_workspace` 返回 `Option<string>`（取消时 `null`，不再是必填路径）。
> 3. `get_project_detail` 返回 `ProjectDetail | null`（id 不存在时为 `null`）。

### 6.3 测试结果

```text
cargo build  ✅ 编译通过（无 warning）
cargo test   ✅ 20 passed; 0 failed
  - core::scanner::tests        （识别 / 忽略 / 非目录报错 / 忽略规则完整性）
  - core::parser::tests         （Rust / Go / Python / Vue / React / Node / 无清单 / 坏 JSON）
  - core::database::crud::tests （upsert 幂等 / 排序 / 详情 / 缺失 / 扫描历史 / 批量原子）
  - core::database::migrations  （projects / scan_history 表结构）
```

### 6.4 新增依赖

| 依赖 | 用途 |
|---|---|
| `tauri-plugin-dialog` | 原生目录选择器（`select_workspace`） |
| `toml` | 解析 `pyproject.toml` 读取项目名 |

`capabilities/default.json` 新增 `dialog:default` 权限。

### 6.5 待前端 Agent 事项

- 将 `src/api/project.ts` / `src/types/project.ts` 对齐 6.2 节的返回结构与新签名。
- `scan_projects` 的调用方（`stores/scanner.ts`）需读取 `result.projects`。
- Dashboard 项目卡、搜索/排序、扫描按钮状态等交互仍属前端 agent 范围。

### 6.6 边界遵守（SPRINT2-01Backend）

- Read Only（仅写 `~/.ydevsphere/database.sqlite` 数据库）。
- 跨平台，无 macOS-only 逻辑。
- 未做：Git / AI / 项目记忆 / 任何前端代码。

---

## 七、Sprint 2 · 前端交付（任务 SPRINT2-01Frontend）

> 交付文档 · 任务编号：`SPRINT2-01Frontend`

### 7.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `src/api/project.ts` | ✅ 完成 | 对齐后端 4 command 真实签名 + 统一错误处理（`ApiError`） |
| `src/types/` | ✅ 完成 | 新增 `ScanResult` / `ScanHistory`，对齐 `core/models` |
| `stores/project.ts` | ✅ 完成 | 项目列表 / 详情 / 加载状态（listLoading / detailLoading）/ 错误 |
| `stores/scanner.ts` | ✅ 完成 | 状态机 `idle / scanning / done / error` + 扫描结果（scanned/ignored counts + history） |
| `stores/settings.ts` | ✅ 完成 | 工作区路径（来自 `select_workspace`，取消返回 null 不覆盖） |
| Dashboard 页 | ✅ 完成 | 真实项目卡片 + 技术栈徽标 + 重新扫描按钮状态流转 + 点击跳详情 |
| ProjectDetail 页 | ✅ 完成 | 展示名称/路径/技术栈/文件数/最近扫描时间；处理空详情（未找到） |
| Settings 页 | ✅ 完成 | 更换工作区目录 + 展示数据库位置（只读说明文字） |
| Logo 接入 | ✅ 完成 | Welcome 顶部居中（h-24）+ AppNav 导航栏左侧 logo+项目名 |
| 前端构建 | ✅ 通过 | `pnpm dev`（HTTP 200，/logo.png 200）& `pnpm build`（vue-tsc + vite 通过） |

### 7.2 前端与后端的联调签名（已对齐）

> 前端 `src/api/project.ts` 已按 `SPRINT2-01Backend` 6.2 节签名对接完成。

| 前端 API | 后端 Command | 入参 | 返回 |
|---|---|---|---|
| `selectWorkspace()` | `select_workspace` | 无 | `Promise<string \| null>`（取消返回 null） |
| `scanProjects(path)` | `scan_projects` | `{ workspacePath }` | `Promise<ScanResult>` |
| `getProjects()` | `get_projects` | 无 | `Promise<Project[]>` |
| `getProjectDetail(id)` | `get_project_detail` | `{ projectId }` | `Promise<ProjectDetail \| null>` |

**错误处理**：所有调用经 `ApiError` 统一封装；后端 `Err(String)` / `{ success, message }` 字符串化格式被解析为可读 `message` 并抛给 store / 页面提示。

### 7.3 扫描按钮状态流转

- `idle` → 点击「重新扫描」→ `scanning`（按钮禁用，文案「扫描中...」，显示进度提示）
- `scanning` → 成功 → `done`（显示「识别 N 个项目，忽略 M 个目录」+ 最近扫描时间，刷新项目列表）
- `scanning` → 失败 → `error`（显示扫描失败信息，列表保留原状）

### 7.4 Logo 接入说明

- 源文件：`public/logo.png`（1536×1024，**未改动图片本身**）。
- Welcome 页顶部居中展示，`h-24`（96px）高。
- `AppNav.vue` 导航栏左侧 `logo + YDevSphere`。
- 均使用 `<img src="/logo.png" alt="YDevSphere" />` 根路径引用。
- 未改动 `src-tauri/icons/`（应用图标另发任务）。

### 7.5 待办 / 待联调事项

- [x] 后端 command 真实签名已对接（`scan_projects` 返回 `ScanResult`，`select_workspace` 返回 `Option`，`get_project_detail` 返回 `Option`）。
- [ ] **端到端人工核验**：启动 `pnpm tauri dev` 选择真实工作区，验证 Dashboard 渲染真实项目、扫描按钮流转、详情跳转与 logo 显示（本 agent 已完成 `pnpm build` / `pnpm dev` 静态验证，GUI 交互需人工/下一任务确认）。
- [ ] **可选增强**：Dashboard 搜索 / 排序功能（PRD 远期项，未在本 Sprint 实现）。
- [ ] **状态持久化**：工作区路径目前仅内存存储，重启后丢失；如需持久化可后续加 `localStorage` 或后端设置接口。

### 7.6 边界遵守（SPRINT2-01Frontend）

- 仅做 Vue 前端：调用层 / 状态 / 页面交互 / logo 接入。
- 未改动任何 Rust / 后端代码。
- 前端不直接访问文件系统 / 执行命令 / 操作数据库，一律走 `invoke()`。
- 未改动 `src-tauri/icons/`。

---

## 八、Sprint 3 · 前端体验增强（任务 SPRINT3-01Frontend）

> 交付文档 · 任务编号：`SPRINT3-01Frontend`

### 8.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| Dashboard 搜索 | ✅ | 顶栏搜索框，按项目名称 / 路径实时过滤；无结果友好空状态 |
| Dashboard 排序 | ✅ | 按最近更新（默认）/ 名称 A–Z，界面带当前排序高亮指示 |
| 最近打开 | ✅ | `localStorage` 持久化（`src/lib/recent.ts`），Dashboard 顶部「最近打开」区，点击直达详情 |
| 扫描反馈增强 | ✅ | 扫描摘要含项目数 / 忽略数 / 耗时；错误 toast / 提示条（`src/lib/toast.ts` + `ToastContainer.vue`） |
| 项目详情完善 | ✅ | 完整路径独立展示、返回按钮、友好加载/空详情状态 |
| 前端构建 | ✅ | `pnpm build`（vue-tsc + vite）通过，0 类型错误；`pnpm dev` 正常 |

### 8.2 新增前端模块

| 文件 | 用途 |
|---|---|
| `src/lib/recent.ts` | 最近打开项目（localStorage） |
| `src/lib/toast.ts` + `src/components/ToastContainer.vue` | 轻量全局 toast（info/success/error，3s 自动消失） |
| `src/lib/sort.ts` | 项目排序（updated / name） |
| `src/lib/format.ts` | 时间 / 耗时格式化 |

### 8.3 交互说明

- **搜索**：输入即实时过滤（名称 / 路径，大小写不敏感），顶部显示「找到 N 个匹配」；无匹配显示友好空状态。
- **排序**：默认「最近更新」（按 `updated_at` 倒序，无更新时间排后）；可切换「名称 A–Z」（`localeCompare` 中文排序）。当前选中项蓝色高亮。
- **最近打开**：每次进入详情页经 `projectStore.markOpened(id)` 记录，最多保留 8 条；Dashboard 按最近优先展示已存在于项目列表中的项。
- **扫描反馈**：成功后绿色提示条（识别数 / 忽略数 / 耗时）+ toast；失败红色提示条 + toast；扫描中禁用按钮并显示蓝色提示。

### 8.4 待办 / 说明

- [ ] **真实数据联调**：搜索 / 排序 / 最近打开均基于 `getProjects()` 返回的真实数据；需 `pnpm tauri dev` 选择真实工作区扫描后人工验证渲染。
- [ ] **可选增强**：最近打开条目的置顶 / 移除操作、搜索历史、排序记忆（当前排序不持久化，刷新回到「最近更新」）。
- [ ] **toast 样式**：当前为简单提示条，如需更强视觉反馈可后续扩展。

### 8.5 边界遵守（SPRINT3-01Frontend）

- 仅做前端体验增强：搜索 / 排序 / 最近打开 / 扫描反馈 / 详情完善。
- 未改动任何 Rust / 后端代码。
- 前端不直接访问文件系统 / 执行命令 / 操作数据库，一律走 `invoke()`；「最近打开」仅用 `localStorage` 存 id 与时间戳，项目数据仍来自后端。
- UI 保持简洁专业，与 logo / 整体风格一致。

---

## 九、Sprint 3 · 后端支撑（任务 SPRINT3-01Backend）

> 交付文档 · 任务编号：`SPRINT3-01Backend`

### 9.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `projects` 表新增列 | ✅ 完成 | 新增 `file_count`（INTEGER）/ `last_scan_at`（DATETIME），幂等迁移兼容旧库 |
| `Project` 结构扩展 | ✅ 完成 | 列表项新增 `file_count` / `last_scan_at`，供「最近更新」排序 |
| `last_scan_at` 准确性修复 | ✅ 完成 | 修复 Sprint 2 中 `get_project_detail` 用项目路径查工作区历史导致的恒为 `None` 的 bug |
| `upsert_projects` | ✅ 完成 | 扫描时统计 `file_count`、写入 `last_scan_at`（扫描时间），一次性落库 |
| 排序支撑 | ✅ 完成 | `get_projects(sort_by)` 可选参数：`"name"` / `"updated_at"`（默认，最近扫描倒序） |
| 扫描历史接口 | ✅ 完成 | 新增 `get_scan_history(limit)` 命令，供「最近扫描摘要」 |
| Rust 单测 | ✅ 通过 | `cargo build` 无 warning；`cargo test` 25 passed, 0 failed |

**分层约束**：`core/` 无 `use tauri`；`commands/` 仅做参数解析与转发。

### 9.2 Command 签名变化（供前端对接，向后兼容）

> 均为**追加可选参数 / 追加返回字段**，不破坏 Sprint 2 既有调用；前端 TS 类型建议同步补充新字段。

| Command | 变化 |
|---|---|
| `get_projects` | 新增可选参数 `sort_by: string \| null`（`"name"` / `"updated_at"`，缺省 `updated_at`）。不传 / `null` / 非法值均回退 `updated_at`。 |
| `get_scan_history` | **新命令**，入参 `limit: number \| null`（缺省 20，钳制 1..=200），返回 `ScanHistory[]`（按时间倒序）。 |
| `Project`（返回结构） | 新增字段 `file_count: number`、`last_scan_at: string \| null`。 |
| `ProjectDetail` | `file_count` / `last_scan_at` 改为直接读库（扫描时落库），语义与 `Project` 一致。 |

**`Project` / `ProjectDetail` 最终字段**（前端 `src/types/project.ts` 建议同步）：
```
{ id, name, path, language, framework, created_at, updated_at, file_count, last_scan_at }
```

### 9.3 语义澄清

- **`last_scan_at`** = 项目最近一次被扫描的时间（本项目级，非工作区级）。扫描某工作区时，其中识别的每个项目都会更新 `last_scan_at`。
- **`updated_at`** = 该项目元数据最近一次写入/更新时间（与 `last_scan_at` 同次扫描时一致）。
- 前端「最近更新」排序：建议优先用 `last_scan_at`（语义更准）；当前前端按 `updated_at` 亦可（两者数值一致）。后端 `get_projects("updated_at")` 即按 `last_scan_at DESC, updated_at DESC, id DESC` 排序，可直接调用。

### 9.4 测试结果

```text
cargo build  ✅ 编译通过（无 warning）
cargo test   ✅ 25 passed; 0 failed
  - core::database::migrations  （新增列结构 / 迁移幂等）
  - core::database::crud::tests （file_count+last_scan_at 填充 / 默认最近扫描排序 / name 排序 /
                                  非法 sort 回退 / 扫描历史读取与 limit 钳制 / 既有 upsert/详情/批量）
  - core::parser / core::scanner（Sprint 2 回归，未破坏）
```

### 9.5 边界遵守（SPRINT3-01Backend）

- 只做后端支撑；未改动任何 Vue 前端代码。
- Read Only（仅写 `~/.ydevsphere/database.sqlite`）。
- 跨平台，无 macOS-only 逻辑。
- 未做：Git / AI / 项目记忆。

---

## 十、Sprint 4 · 后端项目记忆（任务 SPRINT4-01Backend）

> 交付文档 · 任务编号：`SPRINT4-01Backend`

### 10.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `core/models/memory.rs` | ✅ 完成 | 新增 `ProjectMemory` / `ProjectRef` |
| `core/memory/` | ✅ 完成 | 生成 / 读取 / 更新 `.ydevsphere/project.json`；`packageManager` 检测；`stack` 合并 |
| 写入安全 | ✅ 完成 | 仅写 `.ydevsphere/project.json`（原子写）；未授权拒绝 |
| `commands/memory.rs` | ✅ 完成 | 新增 3 个 command，`authorized` 授权标志贯穿 |
| Rust 单测 | ✅ 通过 | `cargo build` 无 warning；`cargo test` 35 passed, 0 failed |

### 10.2 project.json 格式（对齐 PRD）

```json
{
  "name": "YDevSphere",
  "stack": ["Vue3", "TypeScript", "Rust"],
  "packageManager": "pnpm"
}
```

- `stack`：`language` + `framework` 合并去重（language 优先）。
- `packageManager`：由 lockfile 检测（`pnpm-lock.yaml`→`pnpm` / `package-lock.json`→`npm` / `yarn.lock`→`yarn` / `bun.lockb`、`bun.lock`→`bun`）；无 lockfile 时**省略**该字段。

### 10.3 新增 Command 签名（供前端对接）

> 均沿用 `Result<..., String>` 错误格式，`core/` 无 `use tauri`。参数名按 Tauri 2 规则：`project_id` → `projectId`。

| Command | 入参 | 返回 |
|---|---|---|
| `ensure_project_memory` | `projectId: number`, `packageManager: string \| null`, `authorized: boolean` | `ProjectMemory` |
| `get_project_memory` | `projectId: number` | `ProjectMemory \| null`（不存在返回 null） |
| `update_project_memory` | `projectId: number`, `packageManager: string \| null`, `stack: string[] \| null`, `authorized: boolean` | `ProjectMemory` |

**`ProjectMemory` 返回结构**：`{ name: string, stack: string[], packageManager?: string }`（无 packageManager 时字段省略）。

### 10.4 安全红线落地（RESTRICTIONS.md 第 3 节）

- 默认 Read Only：`core/memory` 的读取函数只读，写函数必须 `authorized == true`。
- **仅**允许写 `<project>/.ydevsphere/project.json` 及其 `.tmp` 临时文件；创建 `.ydevsphere/` 目录（若不存在）。
- 写入采用「临时文件 + rename」原子写，避免中断产生半成品。
- 绝不触碰其他源码 / 配置 / 文件；不删除 / 覆盖 / 重写用户已有文件（幂等刷新）。
- `authorized` 标志由前端在用户点击「启用项目记忆」后传入 `true`；否则 `ensure`/`update` 返回 `Unauthorized` 错误且不写任何文件。

### 10.5 测试结果

```text
cargo build  ✅ 编译通过（无 warning）
cargo test   ✅ 35 passed; 0 failed
  - core::memory::tests        （未授权拒绝 / 生成 / 无 lockfile 省略 / lockfile 检测与优先级 /
                                 读取 None/存在 / 幂等 / update 刷新 / stack 去重 / 无效目录）
  - 既有 scanner / parser / database（回归，未破坏）
```

### 10.6 边界遵守（SPRINT4-01Backend）

- 只做后端；未改动任何 Vue 前端代码。
- 写操作仅限用户显式授权后的 `.ydevsphere/project.json`，默认 Read Only。
- 跨平台，无 macOS-only 逻辑。
- 未做：Git / AI / 前端「启用记忆」交互（归前端 agent）。

---

## 十一、Sprint 4 · 前端项目记忆（任务 SPRINT4-01Frontend）

> 交付文档 · 任务编号：`SPRINT4-01Frontend`

### 11.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `src/api/memory.ts` | ✅ | 新增 `ensureProjectMemory` / `getProjectMemory` / `updateProjectMemory`，对齐后端 10.3 节签名 |
| `src/types/project.ts` | ✅ | 新增 `ProjectMemory`（`{ name, stack, packageManager }`） |
| `src/stores/memory.ts` | ✅ | 记忆状态 / 启用中 loading / 错误 + `fetchMemory / enable / skip / update / hasMemory` |
| 首次扫描弹窗 | ✅ | 首次扫描成功且有项目时弹出「是否启用项目记忆？」[启用]/[跳过] |
| 详情页记忆区块 | ✅ | 已启用显示 stack + packageManager 并可编辑；未启用提供「启用」按钮 |
| Dashboard 卡片标识 | ✅ | 已启用记忆的项目卡片右上角显示 ✓ 圆标 |
| 前端构建 | ✅ | `pnpm build`（vue-tsc + vite）通过，0 类型错误；`pnpm dev` 正常 |

### 11.2 新增文件

| 文件 | 用途 |
|---|---|
| `src/api/memory.ts` | 项目记忆 3 个 command 的 invoke 封装 |
| `src/stores/memory.ts` | 项目记忆 Pinia store |
| `src/components/EnableMemoryDialog.vue` | 首次扫描后的「启用项目记忆」询问弹窗 |

### 11.3 与后端联调签名（已对齐 10.3 节）

| 前端 API | 后端 Command | 入参 | 返回 |
|---|---|---|---|
| `ensureProjectMemory(id, packageManager, authorized)` | `ensure_project_memory` | `{ projectId, packageManager, authorized }` | `Promise<ProjectMemory>` |
| `getProjectMemory(id)` | `get_project_memory` | `{ projectId }` | `Promise<ProjectMemory \| null>` |
| `updateProjectMemory(id, packageManager, stack, authorized)` | `update_project_memory` | `{ projectId, packageManager, stack, authorized }` | `Promise<ProjectMemory>` |

### 11.4 安全红线落地（关键）

- **写操作必须 `authorized: true`**：`enable()` / `update()` 仅在用户显式点击「启用 / 保存」后调用并置 `authorized = true`。
- **「跳过」不触发任何写入**：`skip()` 仅本地重置状态，不调用任何接口，不写文件。
- **读取用 `get_project_memory`**（只读，不传 authorized）。
- 仅写 `.ydevsphere/project.json`，前端绝不直接写文件；写入动作全部经 `invoke()` 走后端。

### 11.5 交互说明

- **首次扫描弹窗**：`scanner.memoryPromptTriggered` 在首次扫描成功且识别到项目时置 true，Dashboard 监听后弹出 `EnableMemoryDialog`。
- **启用**：对扫描出的每个项目依次调用 `ensure_project_memory`（authorized=true），成功后 toast 提示并刷新列表（卡片出现 ✓ 标）。
- **跳过**：关闭弹窗，不调用任何写接口。
- **详情页记忆区块**：
  - 未启用：显示「未启用」状态 + 「启用项目记忆」按钮。
  - 已启用：显示技术栈徽标 + 包管理器，提供「编辑」进入编辑态（可改 stack / packageManager）→「保存」调 `update_project_memory`。

### 11.6 待办 / 说明

- [ ] **真实数据联调**：需 `pnpm tauri dev` 选择真实工作区扫描后，人工验证「启用 → 生成 `.ydevsphere/project.json` → 详情展示 → 编辑」全流程（本 agent 已完成 `pnpm build` / `pnpm dev` / `cargo tauri dev` 静态验证）。
- [ ] **记忆标识范围**：Dashboard 卡片 ✓ 标基于本次会话内读取/启用过的项目（`memoryStore.hasMemory`），跨重启后需进入详情页读取才刷新；如需持久化可后续在 `get_projects` 返回 `has_memory` 字段（后端配合）。
- [ ] **编辑校验**：stack 为逗号分隔输入，空串时置 null（保留既有值）。

### 11.7 边界遵守（SPRINT4-01Frontend）

- 仅做前端；未改动任何 Rust / 后端代码。
- 写入动作一律经 `invoke()` 调后端，前端绝不直接写文件。
- 默认 Read Only，仅用户点击「启用」后才触发 `.ydevsphere/project.json` 写入（安全红线）。
- UI 保持简洁专业，与整体风格一致。

---

## 十二、Sprint 5 · 后端 Git 分析（任务 SPRINT5-01Backend）

> 交付文档 · 任务编号：`SPRINT5-01Backend`

### 12.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `git2` 依赖 | ✅ 完成 | `Cargo.toml` 新增 `git2 = "0.20"`（系统 libgit2，跨平台） |
| `core/models/git.rs` | ✅ 完成 | 新增 `GitInfo` / `CommitInfo` / `GitStatus` |
| `core/git/` | ✅ 完成 | `analyze_git(path)` 只读分析：branch / last_commit / status / last_update / is_git_repo |
| 容错 | ✅ 完成 | 非 git 仓库 / `.git` 损坏 / 权限不足 / 无 commit 均优雅降级为 `false`/`None`，不 panic |
| 只读 | ✅ 完成 | 仅用 `git2` 只读 API，无任何写操作 |
| `commands/git.rs` | ✅ 完成 | 新增 `get_project_git_info`（沿用「project_id → 取路径」约定） |
| Rust 单测 | ✅ 通过 | `cargo build` 无 warning；`cargo test` 41 passed, 0 failed |

### 12.2 GitInfo 结构（供前端对接）

```rust
struct GitInfo {
    is_git_repo: bool,
    branch: Option<String>,          // 当前分支名（HEAD detached 为 None）
    last_commit: Option<CommitInfo>, // hash(8位)/message(首行)/author/time(RFC3339)
    status: Option<GitStatus>,       // Clean | Dirty { changed_files }
    last_update: Option<String>,     // 最近 commit 时间（RFC3339）
}
```

### 12.3 新增 Command 签名（供前端对接）

> 沿用 `Result<Option<GitInfo>, String>` 错误格式；参数名按 Tauri 2 规则：`project_id` → `projectId`。`core/` 无 `use tauri`。

| Command | 入参 | 返回 |
|---|---|---|
| `get_project_git_info` | `projectId: number` | `GitInfo \| null` |

**语义**：
- 项目不存在 → `Err("项目不存在: {id}")`。
- 项目存在但非 git 仓库 / `.git` 损坏 / 权限不足 → `Ok(null)`（前端据此隐藏 git 区块）。
- 仓库有效 → `Ok(GitInfo)`（`is_git_repo = true`）。

### 12.4 只读与容错说明

- 仅使用 `git2` 只读 API：`Repository::open` / `head` / `peel_to_commit` / `statuses`。
- **绝不**调用任何写操作（不 commit / checkout / push / pull / reset）。
- 仓库无 HEAD（unborn）→ `branch` / `last_commit` / `status` 为 `None`。
- 跨平台，无 macOS-only 逻辑（依赖系统 libgit2，三平台可用）。

### 12.5 测试结果

```text
cargo build  ✅ 编译通过（无 warning）
cargo test   ✅ 41 passed; 0 failed
  - core::git::analyzer::tests  （非 git→false / 分支+commit 读取 / clean 判定 / dirty 计数 /
                                  缺失目录→false / 只读冒烟）——均用临时 `git init` 仓库测试
  - 既有 memory / scanner / parser / database（回归，未破坏）
```

### 12.6 边界遵守（SPRINT5-01Backend）

- 只做 Git 分析（P1-1）；未做启动项目（P1-2）、AI 分析（P2）。
- 只做后端；未改动任何 Vue 前端代码。
- 严格只读，绝不修改任何 git 状态。
- 跨平台，无 macOS-only 逻辑。

---

## 十三、Sprint 5 · 后端编辑器检测/打开（任务 SPRINT5-02Backend）

> 交付文档 · 任务编号：`SPRINT5-02Backend`

### 13.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `core/models/editor.rs` | ✅ 完成 | 新增 `AvailableEditor` |
| `core/editor/detect.rs` | ✅ 完成 | 编辑器白名单定义 + 检测 / 解析（PATH + 各平台安装路径） |
| `core/editor/open.rs` | ✅ 完成 | 白名单执行 `open_in_editor` |
| `core/editor/settings.rs` | ✅ 完成 | 默认编辑器偏好持久化 `~/.ydevsphere/settings.json` |
| `commands/editor.rs` | ✅ 完成 | 5 个 command；文件管理器打开用 `tauri-plugin-opener` |
| Rust 单测 | ✅ 通过 | `cargo build` 无 warning；`cargo test` 49 passed, 0 failed |

### 13.2 支持编辑器（白名单）

| id | name | CLI 检测 | 平台安装路径（节选） |
|---|---|---|---|
| `vscode` | Visual Studio Code | `code` / `code-insiders` | macOS `/Applications/...app/Contents/Resources/app/bin/code`；Win `Program Files/Microsoft VS Code/bin/code.cmd`；Linux `/usr/bin/code` 等 |
| `cursor` | Cursor | `cursor` | macOS `/Applications/Cursor.app/...` |
| `vscodium` | VSCodium | `codium` | macOS `/Applications/VSCodium.app/...` |
| `webstorm` | WebStorm | `webstorm` | macOS `/Applications/WebStorm.app/...` |
| `intellij` | IntelliJ IDEA | `idea` | macOS `/Applications/IntelliJ IDEA.app/...` |
| `goland` | GoLand | `goland` | macOS `/Applications/GoLand.app/...` |
| `sublime` | Sublime Text | `subl` | macOS `/Applications/Sublime Text.app/...` |
| `atom` | Atom | `atom` | macOS `/Applications/Atom.app/...` |
| `vim` | Vim | `vim` / `gvim` | — |
| `nvim` | Neovim | `nvim` | — |

检测顺序：绝对路径候选 → PATH 中的 CLI（含 Windows `.exe` / `.cmd` 后缀）。

### 13.3 新增 Command 签名（供前端对接）

> 参数名按 Tauri 2 规则：`project_id` → `projectId`；`editor_id` → `editorId`。`core/` 无 `use tauri`。既有 9 个 command 签名不变。

| Command | 入参 | 返回 |
|---|---|---|
| `list_editors` | 无 | `AvailableEditor[]` |
| `open_in_editor` | `projectId: number`, `editorId: string` | `Result<(), string>` |
| `open_in_file_manager` | `projectId: number` | `Result<(), string>` |
| `get_editor_preference` | 无 | `string \| null` |
| `set_editor_preference` | `editorId: string` | `Result<(), string>` |

**`AvailableEditor` 结构**：`{ id: string, name: string }`。

### 13.4 白名单执行 / 安全

- `open_in_editor` 仅执行白名单内、已解析出的编辑器可执行路径；未知 `editor_id` 直接拒绝（`UnknownEditor`），**不执行任何进程**。
- `set_editor_preference` 对非白名单 `editor_id` 拒绝写入。
- 文件管理器打开走 `tauri-plugin-opener::open_path`（系统默认文件管理器），需要 `opener:default` 权限（已具备）。
- 编辑器不可用 → 返回明确错误（`NotFound`），前端据此降级提示。

### 13.5 偏好持久化

- 写 `~/.ydevsphere/settings.json`（应用自身配置目录，非用户项目文件，不受只读红线约束）。
- 支持 `YDEVSPHERE_SETTINGS_PATH` 环境变量覆盖（测试隔离）。

### 13.6 测试结果

```text
cargo build  ✅ 编译通过（无 warning）
cargo test   ✅ 49 passed; 0 failed（连续多轮稳定）
  - core::editor::detect::tests  （白名单完整性 / 拒绝未知 id / 临时 PATH 检测 vscode / id 唯一）
  - core::editor::settings::tests（读取缺失→None / 写入读回 / 非法 editor 拒绝）——env 用锁串行化
  - 既有 git / memory / scanner / parser / database（回归，未破坏）
```

### 13.7 边界遵守（SPRINT5-02Backend）

- 只做编辑器检测 / 打开 / 偏好（Q1+Q2）；未做启动项目（P1-2，另发）、AI 分析（P2）。
- 只做后端；未改动任何 Vue 前端代码。
- 严格白名单执行，非法 `editor_id` 不执行任何进程。
- 跨平台，无 macOS-only 逻辑。

---

## 十四、Sprint 5-02 · 前端编辑器集成（任务 SPRINT5-02Frontend）

> 交付文档 · 任务编号：`SPRINT5-02Frontend`

### 14.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `src/api/editor.ts` | ✅ | 新增 `listEditors` / `openInEditor` / `openInFileManager` / `getEditorPreference` / `setEditorPreference`，对齐后端 13 章签名 |
| `src/types/editor.ts` | ✅ | 新增 `AvailableEditor`（`{ id, name }`） |
| `src/stores/editor.ts` | ✅ | 编辑器列表 / 默认偏好 / 打开动作 + 自动降级到文件管理器 |
| Dashboard 卡片 | ✅ | 双击用默认编辑器打开；「打开」按钮（默认编辑器）+ 下拉菜单（文件管理器 / 其他编辑器） |
| ProjectDetail | ✅ | 「在编辑器打开」「在文件管理器打开」按钮 |
| Settings | ✅ | 「默认启动编辑器」下拉选择，加载 `listEditors`、读 `getEditorPreference`、改 `setEditorPreference` 持久化 |
| 降级处理 | ✅ | 编辑器不可用 → toast 提示 + 自动降级文件管理器；无可用编辑器 → 仅提供文件管理器 |
| 前端构建 | ✅ | `pnpm build`（vue-tsc + vite）通过，0 类型错误；`pnpm dev` 正常 |

### 14.2 新增文件

| 文件 | 用途 |
|---|---|
| `src/api/editor.ts` | 编辑器 5 command 的 invoke 封装 |
| `src/types/editor.ts` | `AvailableEditor` 类型 |
| `src/stores/editor.ts` | 编辑器 Pinia store（列表 / 偏好 / 打开 / 降级） |
| `src/components/OpenActions.vue` | 卡片「打开」按钮 + 下拉菜单 |

### 14.3 与后端联调签名（已对齐 13.2 节）

| 前端 API | 后端 Command | 返回 |
|---|---|---|
| `listEditors()` | `list_editors` | `Promise<AvailableEditor[]>` |
| `openInEditor(projectId, editorId)` | `open_in_editor` | `Promise<void>` |
| `openInFileManager(projectId)` | `open_in_file_manager` | `Promise<void>` |
| `getEditorPreference()` | `get_editor_preference` | `Promise<string \| null>` |
| `setEditorPreference(editorId)` | `set_editor_preference` | `Promise<void>` |

参数名按 Tauri 2 规则：`project_id` → `projectId`，`editor_id` → `editorId`。

### 14.4 交互与降级策略

- **卡片双击**：`ProjectCard` `@dblclick` → `editorStore.openEditor(id, null)`（用默认编辑器；无可用编辑器自动降级文件管理器）。
- **打开按钮**：卡片右上 `OpenActions`，「打开」用默认编辑器（`resolveDefaultEditorId`：优先偏好，其次列表第一个）。
- **下拉菜单**：「在文件管理器中打开」+ 遍历 `editors` 渲染「用 X 打开」；无编辑器时显示「未检测到可用编辑器」。
- **降级**：`openEditor` 捕获失败 → toast「打开编辑器失败…已降级到文件管理器」→ 自动调 `openFileManager`。
- **设置页**：`editorStore.init()` 加载列表 + 偏好；`<select>` 默认值取 `defaultEditorId ?? editors[0]`；change → `setDefault(id)`（后端白名单校验后持久化到 `~/.ydevsphere/settings.json`，跨重启生效）。

### 14.5 待办 / 说明

- [ ] **真实数据联调**：需 `pnpm tauri dev` 有可用编辑器时人工验证：双击卡片打开 / 下拉选其他编辑器 / 设置页切换默认编辑器 / 编辑器不可用时降级提示。
- [ ] **偏好持久化已实现**：`setEditorPreference` 后端写 `~/.ydevsphere/settings.json`，跨重启生效（本 Sprint 聚焦编辑器偏好；全局「工作区路径持久化」为遗留项，未在本 Sprint 处理）。

### 14.6 边界遵守（SPRINT5-02Frontend）

- 仅做前端；未改动任何 Rust / 后端代码。
- 一律走 `invoke()`；前端不直接访问文件系统 / 执行命令。
- UI 保持简洁专业，与整体风格一致。

---

## 十三、Sprint 5 · 前端 Git 分析（任务 SPRINT5-01Frontend）

> 交付文档 · 任务编号：`SPRINT5-01Frontend`

### 13.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `src/api/git.ts` | ✅ | 新增 `getProjectGitInfo`，对齐后端 `get_project_git_info` |
| `src/types/git.ts` | ✅ | 新增 `GitInfo` / `CommitInfo` / `GitStatus`（联合类型） |
| `src/stores/git.ts` | ✅ | git 信息状态 / loading / 错误 + 分支缓存 |
| 详情页 Git 区块 | ✅ | 分支 / 最近提交（hash+message+author+time）/ 工作区状态 / 最近更新 |
| 非 git 仓库空态 | ✅ | 显示「非 Git 仓库」占位 |
| 加载/失败态 | ✅ | loading 提示 + 错误提示条 |
| Dashboard 卡片分支 | ✅（可选） | 已获取过 git 信息的项目卡片显示分支（零额外开销） |
| 前端构建 | ✅ | `pnpm build`（vue-tsc + vite）通过，0 类型错误；`pnpm dev` 正常 |

### 13.2 新增文件

| 文件 | 用途 |
|---|---|
| `src/api/git.ts` | `getProjectGitInfo` invoke 封装 |
| `src/types/git.ts` | Git 类型（对齐 `core/models/git.rs`） |
| `src/stores/git.ts` | Git 分析 Pinia store + `branchOf` 分支缓存 |

### 13.3 与后端联调签名（已对齐 12.3 节）

| 前端 API | 后端 Command | 入参 | 返回 |
|---|---|---|---|
| `getProjectGitInfo(projectId)` | `get_project_git_info` | `{ projectId }` | `Promise<GitInfo \| null>` |

**`GitInfo` 前端结构**：
```ts
interface GitInfo {
  is_git_repo: boolean;
  branch: string | null;                       // HEAD detached 为 null
  last_commit: { hash; message; author; time } | null;
  status: { Clean: true } | { Dirty: { changed_files } } | null;
  last_update: string | null;
}
```

**语义**：项目不存在 → 抛 `ApiError`；非 git 仓库 → `null`（前端显示「非 Git 仓库」）；git 仓库 → `GitInfo`。

### 13.4 详情页 Git 区块渲染

- **当前分支**：有分支显示 `⎇ branch` 徽标；HEAD detached 显示「HEAD detached」。
- **工作区状态**：`Clean` 绿色 / `Dirty（N 处变更）` 琥珀色（`formatGitStatus` / `isGitDirty` 处理 Rust 枚举联合）。
- **最近提交**：hash（8 位 code）+ message + author · time。
- **最近更新**：`last_update` 格式化；无则显示 —。
- **非 git 仓库**：`gitStore.info === null` 时显示虚线空态「非 Git 仓库」。
- **加载/失败**：loading 显示「正在读取 Git 信息...」；错误显示红色提示条。

### 13.5 Dashboard 卡片分支（可选增强）

- 为避免 N 次磁盘/git 批量拉取（性能），Dashboard **不主动**为每个卡片拉 git。
- 进入详情页 `fetchGit` 时写入 `branchCache`；Dashboard 卡片经 `gitStore.branchOf(p.id)` 显示已缓存分支，零额外开销。

### 13.6 待办 / 说明

- [ ] **真实数据联调**：需 `pnpm tauri dev` 选择真实 git 仓库项目，人工验证详情页 Git 区块（分支/提交/状态）与非 git 仓库空态。
- [ ] **分支缓存范围**：卡片分支标识基于本次会话内已访问过详情页的项目；跨重启后需重新进入详情页才刷新。

### 13.7 边界遵守（SPRINT5-01Frontend）

- 仅做前端；未改动任何 Rust / 后端代码。
- 一律走 `invoke()`；前端不直接访问文件系统 / 命令 / git。
- 只读展示，不触发任何 git 写操作（commit / checkout / push / pull）。
- UI 保持简洁专业，与整体风格一致。

---

## 十五、Sprint 5-03 · 后端工作区路径持久化（任务 SPRINT5-03Backend）

> 交付文档 · 任务编号：`SPRINT5-03Backend`

### 15.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `AppSettings` 扩展 | ✅ 完成 | 新增 `workspace_path`（`skip_serializing_if` 省略，向后兼容） |
| 读改写重构 | ✅ 完成 | 抽取 `read_settings` / `write_settings`，编辑器与工作区偏好**互不覆盖** |
| 工作区偏好读写 | ✅ 完成 | `get_workspace_preference` / `set_workspace_preference` |
| 空路径处理 | ✅ 完成 | 空串 / 空白 → 清除工作区偏好（`None`），行为明确 |
| IPC command | ✅ 完成 | 新增 2 个 command，追加不破坏既有 14 个 |
| Rust 单测 | ✅ 通过 | `cargo build` 无 warning；`cargo test` 53 passed, 0 failed |

### 15.2 变更说明

- **`AppSettings` 结构**（`~/.ydevsphere/settings.json`）：
  ```json
  { "default_editor": "vscode", "workspace_path": "/Users/me/Projects" }
  ```
  两字段均 `Option`，序列化时省略 `null`，**向后兼容**旧设置文件。

- **读写策略**：所有 `set_*` 采用「读改写」（read-modify-save），写入时保留对方字段——
  `set_workspace_preference` 不会覆盖 `default_editor`，反之亦然（修复了原 `set_editor_preference` 用 `default()` 重建会覆盖新增字段的问题）。

- **空路径语义**：`set_workspace_preference("")` 或空白 → 清除（写 `null`）；非空 → 保存（`trim` 后）。

- **损坏文件降级**：`settings.json` 解析失败 → 当作无设置，不报错、可正常重写。

### 15.3 新增 Command 签名（供前端对接）

> 参数名按 Tauri 2 规则：`path` → `path`。`core/` 无 `use tauri`。既有 14 个 command 签名不变。

| Command | 入参 | 返回 |
|---|---|---|
| `get_workspace_preference` | 无 | `string \| null` |
| `set_workspace_preference` | `path: string` | `Result<(), string>` |

**前端用法**（配合自动恢复）：
- 用户选完工作区（`select_workspace` 返回路径）后调 `set_workspace_preference(path)` 保存。
- 应用启动时调 `get_workspace_preference()`，非空则自动扫描该工作区并直达 Dashboard，无需重复选择。

### 15.4 边界遵守（SPRINT5-03Backend）

- 只做后端；未改动任何 Vue 前端代码。
- 改动仅限应用自身配置（`~/.ydevsphere/settings.json`），不违反 Read Only 红线（红线只约束用户项目目录）。
- 跨平台，无 macOS-only 逻辑。

---

## 十六、Sprint 5-03 · 前端工作区恢复（任务 SPRINT5-03Frontend）

> 交付文档 · 任务编号：`SPRINT5-03Frontend`

### 16.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `src/api/project.ts` | ✅ | 新增 `getWorkspacePreference` / `setWorkspacePreference`，对齐后端 15 章签名 |
| `src/stores/settings.ts` | ✅ | 启动恢复 `restore()` / 选择后保存（`setWorkspacePreference`）/ 失效降级 `invalidateWorkspace`，含 `restoring` loading 状态 |
| `src/App.vue` | ✅ | 启动自动恢复：非空路径→直达 `/dashboard`；空→保持 Welcome；`restoring` 遮罩防闪烁 |
| Welcome / Settings | ✅ | 选择成功后自动持久化（经 store 调 `setWorkspacePreference`） |
| Dashboard | ✅ | 扫描失败且含「不是有效目录」→ 降级回 Welcome 并 toast 提示 |
| `.gitignore` | ✅ | 补充 `.ydevsphere/`（仓库内本地应用数据） |
| 前端构建 | ✅ | `pnpm build`（vue-tsc + vite）通过，0 类型错误；`pnpm dev` 正常 |

### 16.2 与后端联调签名（已对齐 15.2 节）

| 前端 API | 后端 Command | 入参 | 返回 |
|---|---|---|---|
| `getWorkspacePreference()` | `get_workspace_preference` | 无 | `Promise<string \| null>` |
| `setWorkspacePreference(path)` | `set_workspace_preference` | `{ path }` | `Promise<void>` |

**语义**：`setWorkspacePreference("")` 或空白 → 清除偏好（写 null）；非空 → 保存（trim 后）。

### 16.3 启动自动恢复逻辑

- `App.vue onMounted` → `settings.restore()`（调 `getWorkspacePreference`）。
- 返回非空路径 → `workspacePath` 写入 store + `router.replace({ name: "dashboard" })`（跳过 Welcome）。
- 返回空 → 保持 Welcome（首次使用引导）。
- `restoring` 遮罩层在恢复完成前覆盖，避免闪烁 / 误跳转。

### 16.4 选择后保存与失效降级

- **选择后保存**：`selectWorkspacePath()` 选择成功 → 写 `workspacePath` + 调 `setWorkspacePreference(path)` 持久化（Welcome / Settings 复用此逻辑）。
- **失效降级**：前端不直接访问文件系统（边界约束），故路径失效在后端扫描时暴露（`scan_workspace` 返回 `NotADirectory`「不是有效目录」）。Dashboard `handleScan` 检测到该错误 → `settings.invalidateWorkspace()` + toast「工作区目录已失效，请重新选择。」+ `router.replace({ name: "welcome" })`。

### 16.5 `.gitignore` 核实

| 路径 | 是否覆盖 |
|---|---|
| `dist/`（前端构建产物） | ✅ 已有 `dist` |
| `src-tauri/target/`（Rust 构建产物） | ✅ 已有 `src-tauri/target`（+ `src-tauri/gen`） |
| `.ydevsphere/`（仓库内本地应用数据） | ✅ 本次新增（注：`~/.ydevsphere` 在用户主目录，不在仓库内，此项仅防仓库内同名目录被误提交） |

### 16.6 待办 / 说明

- [ ] **重启恢复人工验证**：需 `cargo tauri dev` 重启场景验证「首次启动→Welcome；选择后保存→再次启动自动恢复直达 Dashboard」；以及恢复路径失效时降级回 Welcome。
- [ ] **失效检测时机**：路径失效检测绑定到扫描失败（因前端不可直接访问文件系统）。若后续需要启动时即校验路径存在性，需后端补充只读 `path_exists` command（未在本 Sprint 实现）。

### 16.7 边界遵守（SPRINT5-03Frontend）

- 仅做前端；未改动任何 Rust / 后端代码。
- 一律走 `invoke()`；前端不直接访问文件系统。
- UI 保持简洁专业，与整体风格一致。

---

## 十七、Sprint 5-04 · 后端系统工作区（任务 SPRINT5-04Backend）

> 交付文档 · 任务编号：`SPRINT5-04Backend`

### 17.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `core/models/workspace.rs` | ✅ 完成 | 新增 `SystemWorkspace` / `SystemWorkspaceKind` |
| `core/workspace/` | ✅ 完成 | `documents_dir()` / `desktop_dir()` / `get_system_workspaces()`（复用 HOME + 英文目录名，不本地化） |
| 扫描复用 | ✅ 完成 | 仅解析路径；实际扫描由前端调 `scan_projects(path)`，不重复实现 |
| `commands/workspace.rs` | ✅ 完成 | 新增 `get_system_workspaces` |
| Rust 单测 | ✅ 通过 | `cargo build` 无 warning；`cargo test` 60 passed, 0 failed |

### 17.2 新增 Command 签名（供前端对接）

> `core/` 无 `use tauri`。仅新增 1 个 command，既有 16 个 command 签名不变。

| Command | 入参 | 返回 |
|---|---|---|
| `get_system_workspaces` | 无 | `SystemWorkspace[]` |

**`SystemWorkspace` 结构**（serde 对齐前端）：
```ts
interface SystemWorkspace {
  kind: "documents" | "desktop";   // 小写
  label: "Documents" | "Desktop";  // 英文，不本地化
  path: string | null;             // 目录不存在为 null
  exists: boolean;                 // 前端据此禁用/隐藏快捷入口
}
```

### 17.3 路径解析

- `documents_dir()` → `~/Documents`；`desktop_dir()` → `~/Desktop`（基于 `dirs::home_dir()` 拼英文目录名，**不做本地化**，符合产品决策）。
- 目录不存在 / 非目录 → 返回 `None`（`exists=false`，`path=null`）。
- 跨平台：macOS 主场景；Windows / Linux 同样解析（不存在则 `exists=false`），无 macOS-only 硬编码。

### 17.4 前端使用约定

- Welcome 页调 `getSystemWorkspaces()` 获取两个入口，`exists` 决定是否展示/禁用。
- 用户点击某入口 → 前端直接调 `scan_projects(该入口.path)` 做实际扫描（**不新造扫描逻辑**）。
- 保留手动 `select_workspace`（隔离能力）。

### 17.5 测试结果

```text
cargo build  ✅ 编译通过（无 warning）
cargo test   ✅ 60 passed; 0 failed（连续多轮稳定）
  - core::workspace::tests（Documents/Desktop 缺失→None / 存在时解析 / 同名文件不算目录 /
                            get_system_workspaces 序列化与 exists 标志 / 不识别本地化目录名）
  - 既有 editor / git / memory / scanner / parser / database（回归，未破坏）
```

### 17.6 边界遵守（SPRINT5-04Backend）

- 只做后端；未改动任何 Vue 前端代码。
- 只读扫描：仅解析路径，不修改任何用户文件。
- 跨平台，无 macOS-only 逻辑。

---

## 十八、Sprint 5-04 · 前端系统工作区（任务 SPRINT5-04Frontend）

> 交付文档 · 任务编号：`SPRINT5-04Frontend`

### 18.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `src/types/workspace.ts` | ✅ | 新增 `SystemWorkspace` / `SystemWorkspaceKind`（对齐 `core/models/workspace.rs`） |
| `src/api/project.ts` | ✅ | 新增 `getSystemWorkspaces()`，对齐后端 `get_system_workspaces` |
| Welcome 页 | ✅ | 顶部两个快捷入口卡片 + 下方「其他选项」可折叠区 |
| 导入联动 | ✅ | `scan_projects` → `setWorkspacePreference` → 跳 Dashboard |
| 目录不存在 | ✅ | 入口禁用 + 点击提示「目录不存在」 |
| 手动选择工作区 | ✅ | 保留（「其他选项」内） |
| 前端构建 | ✅ | `pnpm build`（vue-tsc + vite）通过，0 类型错误；`pnpm dev` 正常 |

### 18.2 与后端联调签名（已对齐 17.2 节）

| 前端 API | 后端 Command | 返回 |
|---|---|---|
| `getSystemWorkspaces()` | `get_system_workspaces` | `Promise<SystemWorkspace[]>` |

**`SystemWorkspace`**：`{ kind: "documents"\|"desktop", label: "Documents"\|"Desktop", path: string\|null, exists: boolean }`

**导入流程**：前端拿到入口后调 `scanProjects(path)`（复用现有 `stores/scanner.ts`），成功后 `setWorkspacePreference(path)` 持久化 + 跳 `/dashboard`。不新造扫描逻辑。

### 18.3 Welcome 页 UI 结构

- **顶部两个快捷入口卡片**：
  - 「一键导入 Documents」（`exists` 为 false → 禁用 + 显示「目录不存在」）
  - 「一键导入 Desktop」（同上）
- **「其他选项」折叠区**（默认收起）：
  - 「选择工作区」（保留手动选择，调 `selectWorkspacePath`）
  - 「只导入 Documents」
  - 「只导入 Desktop」
- 简洁专业风格，与 logo / 整体一致；英文目录名（Documents / Desktop），不本地化。

### 18.4 导入联动细节

- 点击入口 → `scanner.scan(path)`（复用状态机 idle/scanning/done/error）。
- 成功 → `settings.setWorkspace(path)` + `setWorkspacePreference(path)` 持久化 → `router.push("/dashboard")`。
- 失败 → toast「导入失败」；目录不存在 → 按钮禁用 + 点击 toast「目录不存在，无法导入」。
- 扫描中 → 相关按钮禁用，防止重复提交。

### 18.5 待办 / 说明

- [ ] **真实数据联调**：需 `pnpm tauri dev` 在真实 ~/Documents、~/Desktop 存在时人工验证：快捷入口导入、其他选项展开/收起、目录不存在时禁用/提示、手动选择保留。

### 18.6 边界遵守（SPRINT5-04Frontend）

- 仅做前端；未改动任何 Rust / 后端代码。
- 一律走 `invoke()`；前端不直接访问文件系统。
- 扫描复用 `scan_projects`，不新造扫描逻辑。
- UI 保持简洁专业，与整体风格一致。

---

## 十九、Sprint 5-05 · 后端工作区筛选（任务 SPRINT5-05Backend）

> 交付文档 · 任务编号：`SPRINT5-05Backend`

### 19.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| Migration 002 | ✅ 完成 | `projects` 表新增 `workspace TEXT` 列（幂等 `add_column_if_missing`） |
| 兼容旧库 | ✅ 完成 | 已存在项目 `workspace` 为 `NULL`（归「全部」） |
| `upsert_projects` | ✅ 完成 | 扫描时写入 `workspace`（来自 `DetectedProject.workspace`） |
| `Project` / `ProjectDetail` | ✅ 完成 | 新增 `workspace: Option<String>` |
| `get_projects` 筛选 | ✅ 完成 | 新增可选 `workspace_filter`：`all` / `documents` / `desktop` |
| `commands` | ✅ 完成 | `get_projects` 加参数；`scan_projects` 填充 workspace |
| Rust 单测 | ✅ 通过 | `cargo build` 无 warning；`cargo test` 63 passed, 0 failed |

### 19.2 Command 签名变化（供前端对接，向后兼容）

> 既有 17 个 command 中，`get_projects` 追加**可选**参数；其余签名不变。`core/` 无 `use tauri`。

| Command | 入参变化 | 返回 |
|---|---|---|
| `get_projects` | 追加可选 `workspace_filter: string \| null`（`"all"`/`"documents"`/`"desktop"`；不传/`null`/非法值回退 `all`） | `Project[]`（`Project` 新增 `workspace: string \| null`） |

参数名按 Tauri 2 规则：`workspace_filter` → `workspaceFilter`。

**`Project` / `ProjectDetail` 最终字段**（前端 `src/types/project.ts` 建议同步）：
```ts
{ id, name, path, language, framework, created_at, updated_at, file_count, last_scan_at, workspace }
```

### 19.3 筛选规则

- **`all`**（默认）：不过滤（含 NULL workspace 的旧项目 / 手动目录）。
- **`documents`**：`workspace == ~/Documents` 或以 `~/Documents/` 开头（`dirs::home_dir()` 拼接，不硬编码用户名）。
- **`desktop`**：`workspace == ~/Desktop` 或以 `~/Desktop/` 开头。

### 19.4 数据流

- `scan_projects(workspace_path)` → scanner 识别项目 → 为每个 `DetectedProject.workspace = workspace_path` → `upsert_projects` 写入 `projects.workspace`。
- 前端 Dashboard 筛选标签（全部/Documents/Desktop）→ 调 `getProjects(sortBy, workspaceFilter)`。

### 19.5 测试结果

```text
cargo build  ✅ 编译通过（无 warning）
cargo test   ✅ 63 passed; 0 failed（连续多轮稳定）
  - core::database::migrations（新列结构 / 幂等）
  - core::database::crud::tests（upsert 写 workspace / documents/desktop/all 过滤 /
                                 NULL workspace 归 all / 非法 filter 回退 all / 既有排序回归）
  - 既有 editor / git / memory / scanner / parser / workspace（回归，未破坏）
```

### 19.6 边界遵守（SPRINT5-05Backend）

- 只做后端；未改动任何 Vue 前端代码。
- 需数据库 schema 变更（加列），但**只读扫描**，不修改任何用户文件。
- 跨平台，无 macOS-only 逻辑（用 `dirs::home_dir()` 拼路径）。
- 自定义分类功能留 v0.2（本 Sprint 不做）。

---

## 二十、Sprint 5-05 · 前端工作区筛选（任务 SPRINT5-05Frontend）

> 交付文档 · 任务编号：`SPRINT5-05Frontend`

### 20.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `src/types/project.ts` | ✅ | `Project` / `ProjectDetail` 新增 `workspace` 字段；同步补齐 `file_count` / `last_scan_at` |
| `src/api/project.ts` | ✅ | `getProjects(sortBy?, workspaceFilter?)` 新签名，对齐后端 19 章 |
| `src/stores/project.ts` | ✅ | `fetchProjects(sortBy?, workspaceFilter?)` 透传筛选参数 |
| Dashboard 筛选标签栏 | ✅ | 全部 / Documents / Desktop，横向滚动 + 高亮当前项 |
| 搜索/排序联动 | ✅ | 筛选作用于列表（服务端），搜索/排序在其上叠加（客户端） |
| 前端构建 | ✅ | `pnpm build`（vue-tsc + vite）通过，0 类型错误；`pnpm dev` 正常 |

### 20.2 与后端联调签名（已对齐 19.2 节）

| 前端 API | 后端 Command | 入参 | 返回 |
|---|---|---|---|
| `getProjects(sortBy?, workspaceFilter?)` | `get_projects` | `{ sortBy, workspaceFilter }` | `Promise<Project[]>` |

- `sortBy`：`"name"` / `"updated_at"`（默认）；前端 `SortMode` 映射 `updated`→`updated_at`。
- `workspaceFilter`：`"all"`（默认）/ `"documents"` / `"desktop"`；不传回退 `all`（向后兼容）。
- Tauri 参数名映射：`sort_by`→`sortBy`，`workspace_filter`→`workspaceFilter`。
- `Project` / `ProjectDetail` 新增 `workspace: string | null` 字段（前端 types 已同步）。

### 20.3 筛选标签栏（template）

- 位置：「项目列表」标题下方、「工作区：xxx」附近。
- 结构：`<div class="mt-3 flex gap-2 overflow-x-auto pb-1">` + 三个标签按钮（全部 / Documents / Desktop）。
- **横向滚动**：`overflow-x-auto` 使标签多时出现横向滚动条而非换行；`pb-1` 为滚动条留空间。
- 当前选中项高亮：`border-blue-600 bg-blue-600 text-white`；未选中白底灰字。
- 切换 → `toggleFilter(filter)` → `projectStore.fetchProjects(mapSort(sortMode), filter)` 重新拉取。

### 20.4 与搜索 / 排序联动

- **工作区筛选**：服务端（`get_projects` 的 `workspace_filter`）。
- **搜索**：客户端，在已筛选的 `projectStore.projects` 上按名称/路径过滤。
- **排序**：切换排序标签仍走客户端 `sortProjects`；筛选切换时把当前 `sortMode` 映射为 `sort_by` 一并传给后端，保持一致。
- 扫描完成后刷新（`handleScan`）保留当前排序与筛选。

### 20.5 待办 / 说明

- [ ] **真实数据联调**：需 `pnpm tauri dev` 有 Documents / Desktop 项目后人工验证：三个标签切换筛选、横向滚动、筛选与搜索/排序叠加。
- [ ] **自定义分类**：v0.2 再做（本 Sprint 仅系统工作区筛选）。

### 20.6 边界遵守（SPRINT5-05Frontend）

- 仅做前端；未改动任何 Rust / 后端代码。
- 一律走 `invoke()`；前端不直接访问文件系统。
- UI 保持简洁专业，与整体风格一致。

---

## 二十一、Sprint 6 · v0.2 前端重构（任务 V02-FRONT）

> 交付文档 · 任务编号：`V02-FRONT`

### 21.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| 左侧 Sidebar 导航 | ✅ 完成 | 新建 `AppSidebar.vue`（logo + 总览/项目/最近 + 工作区计数 + 设置），替换原顶部 AppNav |
| 布局 | ✅ 完成 | 新建 `layouts/AppLayout.vue`（Sidebar + 内容区），路由改造 |
| Overview 总览页 | ✅ 完成 | 统计卡（真实项目数/仓库数/干净数）+ 活动图（mock）+ 技术栈分布（真实统计）+ 最近项目 |
| Projects 表格页 | ✅ 完成 | 工作区筛选下拉 + 搜索 + 排序下拉 + 扫描状态条 + 项目表格（双击开编辑器） |
| Recent 最近页 | ✅ 完成 | 最近打开项目表格（复用 recentProjects + localStorage 时间戳） |
| Settings 设置页 | ✅ 完成 | 左侧分区导航（通用/工作区/编辑器/隐私/数据库/关于）+ 各分区内容 |
| 数据适配层 | ✅ 完成 | `src/types/view.ts` + `src/lib/view.ts`（Project → ProjectView） |
| Git 状态徽标 | ✅ 完成 | `GitStatusBadge.vue`（Clean/Dirty/Detached/—），映射 `gitStore.infoOf` 缓存 |
| 技术栈徽标改造 | ✅ 完成 | `TechnologyBadge.vue` 对齐 Figma 低饱和配色 |
| 欢迎页 | ✅ 不动 | Welcome.vue 仅改跳转目标 `/dashboard` → `/overview` |
| 前端构建 | ✅ 通过 | `pnpm build`（vue-tsc + vite）0 类型错误 |

### 21.2 新路由结构

| 路由 | 页面 | 说明 |
|---|---|---|
| `/` | Welcome | 不动（现有 Vue 版） |
| `/overview` | Overview 总览 | Figma OverviewPage |
| `/projects` | Projects 表格 | Figma ProjectsPage |
| `/recent` | Recent 最近 | Figma RecentPage |
| `/settings` | Settings 设置 | Figma SettingsPage |
| `/project/:id` | ProjectDetail | 保留（Figma 未设计，纳入 Sidebar 布局） |
| `/dashboard` | → 重定向 `/projects` | 旧路由兼容 |

### 21.3 数据适配层（ProjectView）

Figma 的 `Project`（`technologies[]` / `git{type}` / `updatedAt` 字符串）与后端 `Project`
（`language`/`framework` 单字段 / `git` 需单独拉取）结构不同，统一经 `src/lib/view.ts` 适配：

```ts
interface ProjectView {
  id, name, path,
  technologies: string[],            // [language, framework].filter(Boolean)
  updatedAt: string | null,          // formatDateTime(updated_at)
  lastOpenedAt: string | null,       // localStorage 记录
  gitType: "clean"|"dirty"|"detached"|"none", // gitStore.infoOf 缓存
  gitChanges?: number,
  healthScore?: number,              // v0.2 scanner 迭代后接入（可选）
  raw: Project,                      // 原始引用
}
```

**Git 按需拉取**：表格/总览不批量拉 git（避免 N 次调用），仅展示已缓存信息，未获取显示「—」。
进入详情页 `fetchGit` 后写入 `gitStore.infoCache`。

### 21.4 保留不动的基础设施

- `api/` 层签名不变；`stores/` 内部逻辑不变（仅 `git.ts` 新增 `infoOf` 缓存读取）。
- `Welcome.vue` 完全不动（仅跳转目标改 `/overview`）。
- `App.vue` 启动恢复逻辑保留（跳转目标改 `/overview`）。
- 项目记忆、Git 分析、扫描、工作区筛选、最近打开、打开编辑器/文件管理器、一键导入、启动恢复——全部保留。

### 21.5 mock 图表待接接口清单

| 组件/功能 | 当前状态 | 待接接口 |
|---|---|---|
| Overview 活动图（commits/week） | mock 数据（`ActivityChart.vue`） | 后端 `get_stats`（暂无此接口） |
| Overview 统计「本周活动」 | 未展示（仅项目数/仓库数/干净数） | `get_stats` |
| 技术栈分布 | 真实（language/framework 统计，非 mock） | 无 |

### 21.6 待办 / 说明

- [ ] **真实数据联调**：需 `pnpm tauri dev` 选择真实工作区后人工验证：Sidebar 导航切换、Projects 表格展示/筛选/搜索/排序/扫描、Recent 最近打开、Settings 分区、Overview 统计与图表。
- [ ] **get_stats 接口**：Overview 活动图/本周活动为 mock，待后端提供统计接口后替换。
- [ ] **healthScore**：ProjectView 已预留 `healthScore?`，待 v0.2 scanner 迭代后接入。

### 21.7 边界遵守（V02-FRONT）

- 技术栈保持 Vue 3；Figma React 代码仅作视觉参考，翻译为 Vue，不直接复用。
- 未改动任何 Rust / 后端代码；`api/`、`stores/` 对外签名保留。
- 欢迎页不动；启动恢复逻辑保留。
- 现有功能无回归（扫描/记忆/Git/打开/一键导入/启动恢复）。
- UI 对齐 Figma 布局/配色/间距/字体（`#F7F8FA` 背景、`#2563EB` 主色、`#E5E7EB` 边框）。

---

## 二十二、Sprint 6 · v0.2 Scanner 前端对接（任务 V02-SCAN-FRONT）

> 交付文档 · 任务编号：`V02-SCAN-FRONT`
> 前置：v0.2 Scanner 后端已交付（Phase 1-4），前端重构（V02-FRONT）已完成。

### 22.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `src/types/project.ts` | ✅ | 新增 `ProjectKind` / `DirNode`；`Project` / `ProjectDetail` 各新增 `kind` / `health_score` / `parent_id` 三字段 |
| `src/types/view.ts` | ✅ | `ProjectView` 新增 `healthScore` / `kind` / `parentId`（由 `healthScore?` 转正式必填） |
| `src/api/project.ts` | ✅ | `getProjects` 扩展 `kindFilter?` / `parentIdFilter?` 透传；新增 `getDirChildren(path)` |
| `src/api/ignoreRules.ts` | ✅ | 新增 `getIgnoreRules()` / `setIgnoreRules(dirs)` |
| `src/lib/view.ts` | ✅ | `toProjectView` 正式接入 `health_score` / `kind` / `parent_id` |
| `src/stores/project.ts` | ✅ | `fetchProjects` 透传 `kindFilter` / `parentIdFilter` |
| Projects 页 | ✅ | 健康度分数 + 进度条/颜色分级 + 类型标识（聚合/分类标签）+ 聚合根/分类目录折叠展开（子项目按需加载） |
| 详情页 | ✅ | 展示项目类型 + 健康度；新增「目录结构」懒加载目录树 |
| 设置页 | ✅ | 新增「忽略规则」分区（预设只读说明 + 自定义增删持久化） |
| Overview 页 | ✅ | 技术栈统计按顶层（方案 B），标注待后端提供统计接口 |
| 前端构建 | ✅ | `pnpm build` 0 类型错误；`pnpm dev` 正常 |

### 22.2 与后端联调签名（已对齐）

| 前端 API | 后端 Command | 入参 | 返回 |
|---|---|---|---|
| `getProjects(sortBy?, workspaceFilter?, kindFilter?, parentIdFilter?)` | `get_projects` | `{ sortBy, workspaceFilter, kindFilter, parentIdFilter }` | `Promise<Project[]>` |
| `getDirChildren(path)` | `get_dir_children` | `{ path }` | `Promise<DirNode[]>` |
| `getIgnoreRules()` | `get_ignore_rules` | 无 | `Promise<string[]>` |
| `setIgnoreRules(dirs)` | `set_ignore_rules` | `{ dirs }` | `Promise<void>` |

参数名按 Tauri 2 规则映射：`kind_filter`→`kindFilter`、`parent_id_filter`→`parentIdFilter`。

### 22.3 parent_id_filter 联调确认（关键）

- **已确认**：后端 `get_projects` 的 `parent_id_filter` 匹配的是**父项目 id**（`Option<i64>`，`Some(id)` 时 `parent_id = ?`），而非父项目 path。
- 前端展开聚合根/分类目录时传 `parentIdFilter = 父项目.id`，与后端语义一致，无需修正。
- 后端扫描阶段内部用 `parent_path`（字符串）归属，落库时回填 `parent_id`；前端查询用 `parent_id`，二者在「落库回填」环节已打通。

### 22.4 关键实现说明

- **默认语义变化**：`getProjects()` 不传 `parentIdFilter` 时后端只返回顶层项目（`parent_id IS NULL`）。已核对所有调用点：
  - Projects 页：默认顶层，展开时按需传 `parentIdFilter` 取子项目（符合预期）。
  - Overview 页：技术栈统计仅覆盖顶层（方案 B），已加标注。
  - Recent 页：基于 `recentProjects`（由 `getProjects()` 顶层列表 + localStorage 匹配），子项目若被打开也会记录 id，但顶层列表不含子项目时会漏；属已知限制，待后端统计接口或 recent 接口优化。
- **懒加载目录树**：详情页 `DirTree` + `DirTreeItem`（递归组件），初始 `getDirChildren(项目path)` 展示直接子项；点击 `is_dir && children_count > 0` 的文件夹实时加载下级插入；`has_manifest` 标记真项目根（绿色「项目」标签）。
- **忽略规则**：`getIgnoreRules` 返回目录名列表；设置页预设规则只读展示（node_modules / .git / target / dist / build / vendor / .cache / 隐藏目录），自定义增删经 `setIgnoreRules` 整表替换持久化；提示下次扫描生效。

### 22.5 已知小瑕疵（与后端一致）

- `get_dir_children` 目前只跳过预设忽略目录 + 隐藏目录，**不应用用户自定义忽略规则**——目录树可能显示用户忽略的目录（前端按现状处理，后续可让后端对齐）。
- 忽略规则返回的是目录名（非完整路径），仅匹配扫描时目录名。

### 22.6 待办 / 说明

- [ ] **真实数据联调**：需 `pnpm tauri dev` 选择真实工作区后人工验证：聚合根/分类目录折叠展开、子项目懒加载、详情页目录树展开、设置页忽略规则增删、健康度/类型标识展示。
- [ ] Overview 完整技术栈统计待后端提供 `get_stats`（或支持 `parent_id_filter = i64::MIN` 语义的统计接口）。

### 22.7 边界遵守（V02-SCAN-FRONT）

- 仅做前端（`src/`），未改动任何 Rust / 后端代码。
- 一律走 `invoke()`，前端不直接访问文件系统。
- `api/`、`stores/` 既有签名保留，仅新增可选参数 + 新增 API 函数。
- 现有功能无回归（扫描/工作区筛选/搜索/排序/记忆/Git/打开/最近/设置/启动恢复）。

---

## 二十三、Sprint 6 · v0.2 Scanner 前端修复（任务 V02-BUG-FRONTEND）

> 交付文档 · 任务编号：`V02-BUG-FRONTEND`
> 依据：`docs/v0.2-scanner-bugfix.md`（🟢 前端 agent 负责部分 F1-F6）

### 23.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| F1 工作区改多值集合 | ✅ | `settings store` 新增 `workspaces: string[]`、`addWorkspace`、`removeWorkspace`、`setWorkspaces`；`workspacePath` 改为派生 computed（集合首项，兼容旧引用） |
| F2 一键导入修复 | ✅ | `importAll()`：Documents + Desktop **都加入**集合（不覆盖），依次扫描全部，持久化整个集合 |
| F3 扫描扫全部 | ✅ | `handleScan()`：遍历所有工作区逐个 `scanner.scan`，汇总 scanned/ignored/耗时 |
| F4 工作区列表持久化 | ⚠️ 临时 | 后端无 `get_workspaces`/`set_workspaces` 集合接口；前端采用 `localStorage` 持久化集合，`restore()` 优先读 localStorage，缺失时兼容读后端单值偏好迁移为单元素列表。**已报告总负责人协调后端补集合接口**（见 23.4） |
| F5 menuEl.contains 修复 | ✅ | `ProjectTable` 行菜单 ref 在 `v-for` 内被 Vue 收集为数组导致 `.contains` 不存在；改为 `Map<id, HTMLElement>` 函数 ref 记录每行菜单元素 |
| F6 类型/视图适配 | ✅ | `ProjectView` 等与后端字段一致；设置页/侧边栏展示工作区集合 |
| 前端构建 | ✅ | `pnpm build` 0 类型错误；`pnpm dev` 正常 |

### 23.2 工作区集合模型

- `settings.workspaces: string[]`：多值集合（Documents + Desktop + 手动目录，去重）。
- `addWorkspace(path)` / `removeWorkspace(path)` / `setWorkspaces(list)`：均写 `localStorage`（key `ydevsphere.workspaces`）持久化。
- `workspacePath` 保留为 `computed`（集合首项），兼容 `AppSidebar` / `App.vue` / 既有引用。
- `restore()`：优先读 localStorage 集合 → 无则读后端单值偏好迁移为单元素列表 → 保证与后端一致。

### 23.3 交互说明

- **一键导入**：先 `addWorkspace` 加入 Documents 和 Desktop（不覆盖已有），再逐个 `scanner.scan`，成功即进入 Overview；单个失败不阻断其余。
- **扫描按钮**：遍历 `settings.workspaces` 逐个扫描，汇总提示「已索引 N 个项目，忽略 M 个目录」。
- **设置页工作区**：展示已添加工作区列表，可逐个移除（`removeWorkspace`）。
- **侧边栏**：多工作区时显示「N 个工作区」，单个显示路径末段。

### 23.4 F4 需后端配合（报告总负责人）

- 后端当前仅提供单值 `get_workspace_preference` / `set_workspace_preference`（`AppSettings.workspace_path: Option<String>`），**无法持久化工作区集合**。
- 前端**未擅自改后端**，采用 `localStorage` 作为本次临时持久化（保证「一键导入不覆盖 + 重启恢复」可用）。
- **请总负责人协调后端追加 `get_workspaces` / `set_workspaces` 集合接口**（写入 `AppSettings` 的 `Vec<String>`），前端届时将 `localStorage` 读写替换为 `invoke()` 调用，与后端偏好完全一致。

### 23.5 边界遵守（V02-BUG-FRONTEND）

- 仅做前端（`src/`），未改动任何 Rust / 后端代码。
- 前端不直接访问文件系统 / 命令；工作区集合临时持久化用 `localStorage`（前端本地，非绕过后端写文件），待后端补集合接口后替换。
- 现有功能无回归（扫描/工作区筛选/搜索/排序/记忆/Git/打开/最近/设置/启动恢复）。

---

## 二十四、Sprint 6 · v0.2 工作区集合持久化迁移（任务 V02-WS-FRONTEND）

> 交付文档 · 任务编号：`V02-WS-FRONTEND`
> 前置：后端 V02-WS-BACKEND 已交付 `get_workspaces` / `set_workspaces`。
> 目标：把工作区集合持久化从 localStorage 迁移到后端，后端成为唯一权威源。

### 24.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `src/api/workspaces.ts` | ✅ | 新增 `getWorkspaces()` / `setWorkspaces(dirs)`，对齐后端 `get_workspaces` / `set_workspaces` |
| `src/stores/settings.ts` 迁移 | ✅ | `restore()` 优先调 `getWorkspaces()`（后端权威源）；空则读旧 localStorage 过渡迁移后清 localStorage |
| add/remove/set 持久化 | ✅ | `addWorkspace` / `removeWorkspace` / `setWorkspaces` 更新内存后调 `setWorkspaces()` 持久化到后端 |
| `workspacePath` 派生 | ✅ | 保留 computed（集合首项），兼容旧引用 |
| 过渡期不丢数据 | ✅ | 升级后首次启动：localStorage 有集合而后端无 → 推给后端（`setWorkspaces`）→ 清 localStorage |
| localStorage 降级 | ✅ | 持久化主路径改为后端；localStorage 仅保留为过渡读取兜底（`readLegacyWorkspaces`），迁移后清除，后续可完全移除 |
| 前端构建 | ✅ | `pnpm build` 0 类型错误；`pnpm dev` 正常 |

### 24.2 与后端联调签名（已对齐）

| 前端 API | 后端 Command | 入参 | 返回 |
|---|---|---|---|
| `getWorkspaces()` | `get_workspaces` | 无 | `Promise<string[]>` |
| `setWorkspaces(dirs)` | `set_workspaces` | `{ dirs }` | `Promise<void>` |

后端语义确认：
- `get_workspaces`：集合空但有单值 `workspace_path` 时返回 `[单值]`（旧数据兼容）。
- `set_workspaces`：整表替换（去重 + 去空白），同时镜像 `workspace_path` 为集合首项，保留 `default_editor` / `ignore_dirs`（读改写）。

### 24.3 迁移逻辑（restore）

1. 优先 `getWorkspaces()` 读后端权威源填充 `workspaces`；非空则清旧 localStorage，返回集合首项。
2. 后端为空 → 读旧 localStorage（`ydevsphere.workspaces`）过渡兼容：有集合则 `setWorkspaces(legacy)` 推给后端，成功后清除 localStorage。
3. 前后端都无集合 → 空列表，返回 null（Welcome 首启引导）。

### 24.4 过渡期兼容

- `addWorkspace` / `removeWorkspace` / `setWorkspaces`：内存更新后统一 `persistAll()` → `setWorkspaces(workspaces)` 整表持久化到后端。
- 一键导入（Welcome `importAll`）：`await addWorkspace` 逐个加入并持久化，Documents + Desktop 均落盘到 `settings.json` 的 `workspaces`。
- editor 偏好不受影响：后端 `set_workspaces` 读改写保留 `default_editor`。

### 24.5 边界遵守（V02-WS-FRONTEND）

- 仅做前端（`src/`），未改动任何 Rust / 后端代码。
- 持久化主路径经 `invoke()` 走后端 `set_workspaces`；localStorage 仅过渡读取兜底，迁移后清除。
- 现有功能无回归（扫描/工作区筛选/搜索/排序/记忆/Git/打开/最近/设置/启动恢复/编辑器偏好）。

---

## 二十五、Sprint 6 · i18n 双语适配（任务 V02-I18N）

> 交付文档 · 任务编号：`V02-I18N`
> 目标：前端所有 UI 文案可切换中/英，语言偏好持久化到后端。

### 25.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| vue-i18n 接入 | ✅ | 安装 `vue-i18n@^9`；`src/lib/i18n.ts` 创建实例；`main.ts` 挂载 |
| 语言包 | ✅ | `src/locales/zh-CN.ts` / `en-US.ts` / `index.ts`（messages + supportedLngs + defaultLng） |
| i18n store | ✅ | `src/stores/i18n.ts`：`locale` / `setLocale` / `init`，联动 vue-i18n 实例 |
| 文案抽取 | ✅ | 全部页面（Overview/Projects/Recent/Settings/ProjectDetail/Welcome）+ 组件（Sidebar/ProjectTable/GitStatusBadge/DirTree/DirTreeItem/EnableMemoryDialog/ProjectCard/OpenActions/TechStackList）+ stores/format/toast |
| Documents/Desktop 双语 | ✅ | `workspace.documents`（文稿/Documents）、`workspace.desktop`（桌面/Desktop），路径不变 |
| 动态文案 | ✅ | 参数化：`scan.summary`（scanned/ignored/duration）、`memoryDialog.enabledToast`、`workspace.multiple` 等 |
| 语言切换入口 | ✅ | 设置页「通用」分区新增「语言」下拉（中文/English） |
| 前端构建 | ✅ | `pnpm build` 0 类型错误；`pnpm dev` 正常 |

### 25.2 语言持久化状态（⚠️ 需后端配合）

- **后端尚无 `get_language_preference` / `set_language_preference` 接口**（已确认，0 匹配）。
- 前端已做**降级方案**：
  - `src/api/language.ts` 封装两个 invoke（预期后端签名已按 `get_workspaces` / `set_workspaces` 模式注释）。
  - `i18n store.init()`：优先调后端 `get_language_preference` → 失败（command 不存在）降级读 localStorage（`ydevsphere.language`）→ 再无则默认 `zh-CN`。
  - `setLocale()`：优先 `set_language_preference` → 失败降级 localStorage。
- **待总负责人协调后端追加**：`get_language_preference() -> Result<Option<String>, String>` / `set_language_preference(lng) -> Result<(), String>`，并在 `AppSettings` 加 `language: Option<String>`（参照 `workspaces` 模式）。后端就绪后，前端 i18n store **无需改动**即可走后端持久化（降级逻辑自动失效）。

### 25.3 关键实现说明

- 组件内用 `useI18n()` 的 `t()`；非组件（`format.ts` / `editor store` 等）用 `lib/i18n.ts` 导出的 `t()`（即 `i18n.global.t`）。
- `App.vue` 启动时先 `i18nStore.init()` 再恢复工作区，避免闪烁。
- 用户输入（忽略目录名、项目名、技术栈、包管理器）原样显示，不翻译。
- 后端错误消息（如「不是有效目录」）原样透传显示，不翻译。
- 语言名称「中文」选项硬编码（语言选择器标准做法，各语言下均显示本名）。

### 25.4 边界遵守（V02-I18N）

- 仅做前端（`src/`），未改动任何 Rust / 后端代码。
- 语言持久化后端接口缺失，前端用 localStorage 降级 + 明确注释，未擅自改后端。
- 现有功能无回归（扫描/筛选/搜索/排序/记忆/Git/打开/最近/设置/启动恢复/编辑器偏好）。

---

## 二十六、Sprint 6 · App Shell + Projects 响应式布局（任务 V02-RESPONSIVE）

> 交付文档 · 任务编号：`V02-RESPONSIVE`
> 规格：App Shell + Projects 响应式布局（v0.3 定稿）；Overview/Recent/Settings 仅保证不破版。
> 架构原则：**Layout Mode 只影响 UI，不影响数据模型**（窗口变化不触发重新扫描、不改任何 SQLite 查询字段）。

### 26.1 完成度

| Step | 模块 | 状态 | 说明 |
|---|---|---|---|
| Step 1 | 全局 Layout Mode | ✅ | 新建 `src/stores/layout.ts`（Pinia）：`appMode: large/medium/small`；判定 `>=1072→large`、`>=852→medium`、否则 `small`。`AppLayout.vue` 用**单个 ResizeObserver** 监听 `window.innerWidth` 写 store（未引入第二套列宽测量） |
| Step 2 | Sidebar 三态 | ✅ | `AppSidebar.vue`：large/medium `w-[220px]` 完整导航（icon+文字+工作区）；small `w-[72px]` Icon-only（图标居中 + title tooltip、工作区项右上角项目数角标、隐藏文字）；加 `transition-[width]` |
| Step 3 | ProjectTable 三态 | ✅ | GRID 三态（Header/Row 共享）；列显隐（medium/small 隐藏 Git + 时间）；Tech 单行（flex + truncate，large 3/medium 2/small 1 个 +N）；small More(⋯) 菜单始终可见，含 Open/Open in Editor/reveal/Copy Path/Details，reveal label 按平台 |
| Step 4 | Projects 页适配 | ✅ | 工具栏 small 下 `flex-wrap`，搜索框 `w-full max-w-[340px]`，排序/扫描 `shrink-0`；页头 small 下允许换行；未缩字体 |
| Step 5 | Overview/Recent/Settings 不破版 | ✅ | 均为 `max-w-[1060/1140px]` 居中 + flex 容器，Sidebar 220/72 两种宽度下不溢出、不重叠；未重做其自身响应式 |
| 前端构建 | ✅ | `pnpm build` 0 类型错误；`pnpm dev` 正常 |

### 26.2 各 Mode 表格列（GRID 三态）

| Mode | 判定宽度 | Sidebar | 表格列 | GRID |
|---|---|---|---|---|
| small | <852 | 72px Icon-only | Project / Tech(1 个+N) / Health / More(40px) | `minmax(240px,1fr) minmax(120px,1fr) minmax(100px,1fr) 40px` |
| medium | 852–1071 | 220px 完整 | Project / Tech(2 个+N) / Health / Actions | `minmax(240px,1fr) minmax(120px,1fr) minmax(100px,1fr) 76px` |
| large | ≥1072 | 220px 完整 | 6 列全显示（项目/技术栈/Git/健康度/时间/操作） | `minmax(240px,1fr) minmax(120px,1fr) minmax(110px,1fr) minmax(100px,1fr) minmax(110px,1fr) 76px` |

- 各列有**独立最小可用宽度**，剩余空间允许时参与弹性分配（非简单 1fr 等比例）。
- Header 与 Row 共享同一 `gridTemplateColumns`（保证列对齐）。

### 26.3 Small More 菜单（平台相关 label）

- More(⋯) 按钮在 small 下**始终可见**（非 hover 才显示）。
- 菜单项：Open / Open in Editor（逐个编辑器）/ revealInFileManager / Copy Path / Details。
- reveal label 按平台（`navigator.userAgent` 检测）：macOS「在 Finder 中显示」/ Windows「在资源管理器中显示」/ 其他「在文件管理器中显示」。
- 新增 i18n key：`table.open`、`table.moreTech`、`table.revealInFinder/Explorer/FileManager`（zh-CN / en-US 双语言）。

### 26.4 实现要点

- **Tech 单行**：`flex flex-wrap` → `flex` 单行 + `overflow-hidden`；固定可见数（large 3 / medium 2 / small 1），超出显示 `+N` 灰底徽标；技术名过长 truncate；Row 高度稳定 68px。
- **Layout Mode 只影响 UI**：窗口 resize 仅更新 `appMode`，不触发任何扫描/查询；`ProjectTable` 从 layout store 自动读取，页面无需额外传参。
- 未引入第二套 ResizeObserver 做列宽测量；未缩字体。

### 26.5 边界遵守（V02-RESPONSIVE）

- 仅做前端（`src/`），未改动任何 Rust / 后端代码。
- 不一次性大改 Overview/Recent/Settings 自身响应式（仅保证不破版）。
- 不引入第二套 ResizeObserver 做列宽测量。
- 现有功能无回归（扫描/筛选/搜索/排序/记忆/Git/打开/最近/设置/启动恢复/编辑器偏好）。

---

## 二十七、Sprint 6 · 编辑器动态发现前端对接（任务 SPRINT-EDITOR-FRONTEND）

> 交付文档 · 任务编号：`SPRINT-EDITOR-FRONTEND`
> 前置：后端已交付 `list_editors` 扩展结构 + `rescan_editors`（含 CLI 解析、去重、打开方式分级）。

### 27.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `src/types/editor.ts` | ✅ | `AvailableEditor` 新增 `cli_command?` / `app_path?` / `open_method` / `source` / `category`；新增 `OpenMethod` / `EditorSource` / `EditorCategory` 类型 |
| `src/api/editor.ts` | ✅ | 新增 `rescanEditors()`（`rescan_editors`） |
| `src/stores/editor.ts` | ✅ | 新增 `rescan()`（清缓存重扫 + 刷新列表） |
| Settings 默认编辑器下拉 | ✅ | discovered → 「自动发现」标记；unsupported → 置灰（`:disabled`）+「请手动选目录」标记；新增「重新扫描编辑器」按钮 |
| OpenActions 下拉 | ✅ | unsupported 的编辑器不显示「用 X 打开」项（`openableEditors` 过滤） |
| ProjectTable 行内下拉 | ✅ | 同步过滤 unsupported（两处「用 X 打开」列表 + 默认编辑器名优先选 openable） |
| 前端构建 | ✅ | `pnpm build` 0 类型错误；`pnpm dev` 正常 |

### 27.2 与后端签名对齐（已确认）

| 前端 API | 后端 Command | 入参 | 返回 |
|---|---|---|---|
| `listEditors()` | `list_editors` | 无 | `Promise<AvailableEditor[]>` |
| `rescanEditors()` | `rescan_editors` | 无 | `Promise<AvailableEditor[]>` |

`AvailableEditor` 新增字段（枚举均 snake_case 序列化）：
- `cli_command?: string | null`（CLI 命令 / app 内绝对路径）
- `app_path?: string | null`（.app 包路径）
- `open_method: "cli" | "open_a" | "unsupported"`（打开方式）
- `source: "whitelist" | "discovered"`（来源）
- `category: "vscode_fork" | "native" | "ai_chat"`（分类）

### 27.3 实现要点

- **discovered 标记**：Settings 下拉 `<option>` 文本追加「（自动发现）」；unsupported 追加「（请手动选目录）」并 `:disabled`。
- **unsupported 不显示打开项**：`openableEditors = editors.filter(e => e.open_method !== "unsupported")`，用于 OpenActions 与 ProjectTable 行内下拉；默认编辑器名 / 解析默认 id 优先选 openable。
- **重扫按钮**：Settings 编辑器分区新增「重新扫描编辑器」→ `editorStore.rescan()`（`loading` 期间禁用）。
- 新增 i18n key：`editor.autoDiscovered` / `editor.selectDirManually` / `editor.rescan`（zh-CN / en-US）。

### 27.4 边界遵守（SPRINT-EDITOR-FRONTEND）

- 仅做前端（`src/`），未改动任何 Rust / 后端代码。
- 现有编辑器打开链路不变（`openEditor` / `openFileManager` 降级逻辑未动）。
- 现有功能无回归（扫描/筛选/搜索/排序/记忆/Git/打开/最近/设置/启动恢复/编辑器偏好）。

---

## 二十八、v0.3 · 后端编辑器发现误判治理（任务 V02-EDITOR-FIX / V03）

### 28.1 完成度

| 模块 | 状态 | 说明 |
|---|---|---|
| `core/models/editor.rs` | ✅ | `EditorCategory` 新增 `AiEditor` 变体（serde 序列化为 `ai_editor`） |
| `core/editor/discover.rs` | ✅ | 三层识别口径（L1 product.json 指纹 / L2 代码类型自动 / L3 排除） |
| `core/editor/detect.rs` | ✅ | `list_available_editors` 过滤 unsupported（防御） |
| `core/editor/settings.rs` | ✅ | `AppSettings` 新增 `custom_editors` + `get_custom_editors` / `set_custom_editors` / `is_custom_editor` |
| `commands/editor.rs` | ✅ | `scan_and_cache` 读缓存过滤 unsupported（清旧缓存污染）；新增 `list_app_candidates` / `confirm_custom_editor` |

### 28.2 识别口径（用户拍板）

- **L1 可靠自动**：product.json 指纹（VS Code Fork）→ `Cli`，自动进列表。保持不变。
- **L2 代码类型自动**：无 product.json，但 Info.plist `CFBundleTypeExtensions` 声明了代码文件类型（`CODE_FILE_EXTENSIONS` 清单任一生效，主流语言 + 配置文件扩展名）→ `OpenA`，分类为 `AiEditor`（如 ChatGPT/Codex、Claude），自动进列表。
- **L3 排除**：仅 `public.folder` 或仅 bundleId、无代码文件类型 → 一律不进列表（不产生 Unsupported，不进自动列表、不进手动候选）。IINA / 浏览器 / 办公软件等被排除。

### 28.3 新增 Command 签名（供前端对接）

| 前端 API | 后端 Command | 入参（Rust → camelCase） | 返回 | 语义 |
|---|---|---|---|---|
| `listAppCandidates()` | `list_app_candidates` | 无 | `Promise<AvailableEditor[]>` | 手动候选列表 = 自动检测（L1+L2）+ 已确认 custom_editors，去重合并；仅含可打开（cli/open_a） |
| `confirmCustomEditor(editorId)` | `confirm_custom_editor` | `editor_id` → `editorId` | `Promise<void>` | 将指定编辑器写入 `custom_editors`（去重）；未知 id 返回「未知编辑器」 |

### 28.4 `EditorCategory::AiEditor` 序列化值

- `AiEditor` → `"ai_editor"`（snake_case，serde 自动）。

### 28.5 `custom_editors` 结构

`AppSettings.custom_editors: Vec<AvailableEditor>`（用户手动确认导入的编辑器权威源），持久化到 `~/.ydevsphere/settings.json`。读写采用「读改写」保留其他字段。

### 28.6 测试结果

```text
cargo build  ✅ 编译通过（无 warning）
cargo test   ✅ 127 passed; 0 failed（基线 120 → 新增 7 个）
  - discover：L2 代码类型 open-a/AiEditor、L3 仅 public.folder 排除、L3 仅 bundleId 排除、代码类型判定（大小写不敏感）
  - settings：custom_editors 默认空 / roundtrip+is_custom / 不覆盖其他设置
  - commands：filter_usable 缓存清洗（移除 unsupported / 全 unsupported 为空）
```

### 28.7 架构约束

- `core/` 无 `use tauri`。
- `commands/` 只做参数解析 + 转发，不放业务逻辑。
- 只写 `~/.ydevsphere/` 应用数据；跨平台，无 macOS-only 业务逻辑。
- 编辑器打开仍经过已知编辑器校验，未知 id 拒绝执行。

---

> **后端已完成，待前端 agent 对接**：新增 `listAppCandidates()` / `confirmCustomEditor(editorId)` 两个 command；`AvailableEditor.category` 新增 `ai_editor` 取值；旧 `list_editors` 返回结构不变但已过滤 unsupported。前端需同步 TS 类型 `EditorCategory` 增加 `"ai_editor"`，并据 `list_app_candidates` 实现 Welcome「选择常用编辑器」引导与 Settings「手动导入自定义编辑器」。
