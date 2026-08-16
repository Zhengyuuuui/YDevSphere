# YDevSphere 前端结构说明（v0.2 前端重构后）

> 版本：v0.2 Frontend · 维护人：总负责人
> 用途：记录 v0.2 前端重构（Figma 设计 → Vue 3）后的代码结构、页面 UI、组件与数据模型依据。

---

## 一、前端目录树（带注释）

```
src/
├── main.ts                        # 应用入口：挂载 Pinia + Router + 样式
├── App.vue                        # 根组件：启动自动恢复工作区、全局 Toast 容器、路由出口
├── styles.css                     # Tailwind 指令
├── vite-env.d.ts                  # Vite 环境类型声明
│
├── router/
│   └── index.ts                   # 路由定义（懒加载）
│       ├── /                → Welcome        （欢迎页 / 首次引导，不动）
│       ├── AppLayout（Sidebar 布局）
│       │   ├── /overview   → Overview        （总览）
│       │   ├── /projects   → Projects        （项目表格）
│       │   ├── /recent     → Recent          （最近）
│       │   ├── /settings   → Settings        （设置）
│       │   └── /project/:id → ProjectDetail  （项目详情，保留）
│       └── /dashboard      → 重定向 /projects（旧路由兼容）
│
├── layouts/
│   └── AppLayout.vue              # Sidebar 布局（左侧导航 + 内容区）
│
├── pages/                         # 页面组件（视图层）
│   ├── Welcome.vue                # 欢迎页（不动）：logo、一键导入、其他选项折叠
│   ├── Overview.vue               # 总览：统计卡、活动图（mock）、技术栈分布（真实）、最近项目
│   ├── Projects.vue               # 项目表格：工作区筛选下拉、搜索、排序、扫描状态条、表格
│   ├── Recent.vue                 # 最近：最近打开项目表格
│   ├── Settings.vue               # 设置：分区导航（通用/工作区/编辑器/隐私/数据库/关于）
│   └── ProjectDetail.vue          # 项目详情（保留）：基本信息、项目记忆、Git 信息
│
├── components/                    # 可复用组件
│   ├── AppSidebar.vue             # 左侧导航（logo + 总览/项目/最近 + 工作区 + 设置）
│   ├── ProjectTable.vue           # 项目表格（名称/技术栈/Git/时间/操作下拉）
│   ├── TechnologyBadge.vue        # 技术栈徽标（Figma 低饱和配色）
│   ├── GitStatusBadge.vue         # Git 状态徽标（Clean/Dirty/Detached/—）
│   ├── StatCard.vue               # 总览统计卡
│   ├── ActivityChart.vue          # 活动图（mock，待接 get_stats）
│   ├── TechStackList.vue          # 技术栈分布列表（真实数据统计）
│   ├── ToggleSwitch.vue           # 设置页开关（视觉组件）
│   ├── OpenActions.vue            # 卡片「打开」按钮 + 下拉菜单（编辑器/文件管理器）
│   ├── ProjectCard.vue            # 项目卡片（保留，供旧交互复用）
│   ├── EnableMemoryDialog.vue     # 首次扫描后「启用项目记忆」询问弹窗
│   └── ToastContainer.vue         # 全局 toast 提示容器
│
├── stores/                        # Pinia 状态
│   ├── settings.ts                # 工作区路径（选择/恢复/持久化）
│   ├── project.ts                 # 项目列表/详情/最近打开
│   ├── scanner.ts                 # 扫描状态机（idle/scanning/done/error）
│   ├── memory.ts                  # 项目记忆状态（启用/更新）
│   ├── git.ts                     # Git 信息 + 分支缓存 + infoCache（供列表徽标）
│   └── editor.ts                  # 编辑器检测/默认编辑器/打开动作
│
├── api/                           # 后端 command 调用封装（一律走 invoke()）
│   ├── project.ts                 # selectWorkspace / scanProjects / getProjects / getProjectDetail / getSystemWorkspaces / 工作区偏好
│   ├── memory.ts                  # ensure/get/update 项目记忆
│   ├── git.ts                     # getProjectGitInfo
│   ├── editor.ts                  # listEditors / openInEditor / openInFileManager / 编辑器偏好
│   └── index.ts                   # 统一导出
│
├── types/                         # 前端 TS 类型（对齐 Rust core/models）
│   ├── project.ts                 # Project / ProjectDetail / ScanResult / ScanHistory / ProjectMemory
│   ├── git.ts                     # GitInfo / CommitInfo / GitStatus
│   ├── editor.ts                  # AvailableEditor
│   ├── workspace.ts               # SystemWorkspace / SystemWorkspaceKind
│   ├── view.ts                    # ProjectView / GitStatusView / TechnologyView（视图模型）
│   └── index.ts                   # 统一导出
│
└── lib/                           # 工具函数
    ├── constants.ts               # 展示常量（数据库位置说明等）
    ├── format.ts                  # 时间/耗时/Git 状态格式化
    ├── sort.ts                    # 项目排序
    ├── recent.ts                  # 最近打开项目（localStorage）
    ├── view.ts                    # 数据适配层（Project → ProjectView）
    └── toast.ts                   # 轻量 toast
```

---

## 二、数据适配层（Figma mock → 后端真实）

Figma 的 `Project` 结构与后端不同，前端统一经 `src/lib/view.ts` 适配：

```ts
// 后端真实 Project（language/framework 单字段）
// → 前端视图模型 ProjectView（technologies[] / gitType / updatedAt）

interface ProjectView {
  id: number;
  name: string;
  path: string;
  technologies: string[];      // [language, framework].filter(Boolean)
  updatedAt: string | null;    // formatDateTime(updated_at)
  lastOpenedAt: string | null; // 来自 localStorage 记录
  gitType: "clean" | "dirty" | "detached" | "none"; // 来自 git store infoCache
  gitChanges?: number;         // dirty 时变更数
  healthScore?: number;        // v0.2 scanner 迭代后接入（当前可选）
  raw: Project;                // 原始后端引用（跳详情/打开用）
}
```

关键适配函数（`src/lib/view.ts`）：
- `toTechnologies(p)`：`[language, framework].filter(Boolean)`
- `toGitStatusView(info)`：Rust 枚举 `GitStatus` → 视图 `clean/dirty/detached/none`
- `gitChangeCount(info)`：dirty 变更数
- `toProjectView(project, gitInfo?, lastOpenedAt?)`：单项目适配
- `toProjectViews(projects, gitOf, lastOpenedOf?)`：批量适配

**Git 列按需拉取策略**：表格/总览**不**批量拉 git（避免 N 次调用），仅展示 `gitStore.infoOf(id)` 已缓存的信息；未获取显示「—」。进入详情页 `fetchGit` 后写入 `infoCache`。

---

## 三、页面级 UI 结构

### 1. Welcome 页（/）—— 不动
原 Welcome.vue 保持不变，仅将导入成功后跳转目标从 `/dashboard` 改为 `/overview`。

### 2. Overview 总览（/overview）
```
┌──────────────────────────────────────────┐
│ [Sidebar]  总览                           │
│  早上好 / 下午好 / 晚上好                    │
│  [127 项目] | [42 Git仓库] | [18 干净]    │  ← StatCard（真实数据）
│  ┌─────────────┐  ┌──────────────┐       │
│  │ 工作区活动    │  │ 技术栈分布     │       │
│  │ (mock 折线)  │  │ (真实统计)    │       │
│  └─────────────┘  └──────────────┘       │
│  ┌─ 最近项目 ──────────────────── [查看全部]│
│  │ 项目名 · 技术栈           时间          │
│  └──────────────────────────────────────┘│
└──────────────────────────────────────────┘
```

### 3. Projects 表格（/projects）
```
┌──────────────────────────────────────────┐
│ [Sidebar]  项目            [扫描]         │
│  全部 ▾ · 127 个项目                        │
│  [扫描状态条：扫描中/完成/失败]              │
│  [搜索框]                    [排序 ▾]      │
│  ┌ 表格 ──────────────────────────────┐  │
│  │ 项目 | 技术栈 | Git | 时间 | 操作   │  │
│  │ 行：名称+路径 / 徽标 / Clean / ...  │  │
│  └────────────────────────────────────┘  │
│  （双击行 → 默认编辑器打开）               │
└──────────────────────────────────────────┘
```

### 4. Recent 最近（/recent）
最近打开项目表格（`showLastOpened=true`，时间列显示「最近打开」）。

### 5. Settings 设置（/settings）
左侧分区导航（通用/工作区/编辑器/隐私/数据库/关于）+ 右侧内容区。

### 6. ProjectDetail（/project/:id）—— 保留
纳入 Sidebar 布局，移除原顶部 AppNav，其余保留（基本信息/项目记忆/Git 信息/打开按钮）。

---

## 四、Figma → Vue 组件映射

| Figma 组件 | Vue 实现 |
|---|---|
| Sidebar | `components/AppSidebar.vue` |
| ProjectTable | `components/ProjectTable.vue` |
| TechBadge | `components/TechnologyBadge.vue`（改造，低饱和配色） |
| GitStatusBadge | `components/GitStatusBadge.vue`（新建） |
| OverviewPage 统计卡 | `components/StatCard.vue` |
| 活动图 / 技术栈分布 | `components/ActivityChart.vue`（mock）/ `TechStackList.vue`（真实） |
| SettingsPage 开关 | `components/ToggleSwitch.vue` |

---

## 五、待接后端接口清单（mock 占位）

| 组件/功能 | 当前状态 | 待接接口 |
|---|---|---|
| Overview 活动图（commits/week） | mock 数据 | 后端 `get_stats`（暂无此接口） |
| Overview 统计「本周活动」 | 未展示（仅项目数/仓库数/干净数） | `get_stats` |
| 技术栈分布 | **真实**（language/framework 统计） | 无 |

---

## 六、设计约束（PRD UI 原则）

- 遵循 Apple Human Interface Guidelines。
- 简洁、高信息密度、专业开发者工具。
- **禁止**：游戏化 UI、大面积渐变、玻璃效果、复杂动画。
- 视觉基调：白底 / 灰边框（`#E5E7EB`）/ 主色蓝（`#2563EB`）/ 低饱和技术栈徽标。
- 页面背景 `#F7F8FA`，卡片白底圆角 `rounded-[8px]`/`rounded-[10px]`。
