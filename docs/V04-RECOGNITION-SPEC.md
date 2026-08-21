# YDevSphere · V0.4 识别引擎规格（V04-RECOGNITION-SPEC v0.5）

> 维护人：总负责人 · 状态：**v0.5（已修正 6 个 P0 审计问题）** · 待冻结后按 PR1-4 实现
> 依据：`V04-RECOGNITION-AUDIT` 诊断报告 + 两轮 v0.4 方向讨论 + P0/P1 审计
> 定位：把 YDevSphere 识别能力从 v0.3「单项目单技术」升级为 v0.4「**项目结构 + 多技术栈 + 子项目聚合**」的**第一代识别引擎 / Project Intelligence Model**。
>
> **核心原则**：不是"多加识别规则"，而是**建立识别引擎**。渐进式重构，不推倒重写。识别必须**可解释**（内部留 source_kind，但暂不做 Confidence）。

---

## 0. 目标与原则

### 0.1 能力升级目标

```text
V0.3                     V0.4                     V0.5+
单目录                  → 项目节点               → 复杂 Monorepo Intelligence
language + framework    → 多技术（多来源）        → Evidence / Confidence
项目卡片                → 子项目 + 聚合          → AI 项目理解
```

### 0.2 迭代原则

- 渐进式重构，不一次推倒整个 Scanner。
- 分 PR 交付（PR1 → PR2 → PR3 → PR4，含测试矩阵）。
- 第一验收案例：藏蓝闪送（前端 Vue/uni-app + 后端 Express/SQLite）。
- **父级技术栈是 derived，子项目才是 Source of Truth**（见 §2.2）。

### 0.3 本阶段明确不做

- ❌ Evidence / Confidence 数值（但内部保留 `source_kind`，见 §13）
- ❌ AI 判断
- ❌ 无限递归
- ❌ 复杂 Monorepo dependency graph（V0.5）
- ❌ 一次性识别 100+ 框架

**先解决「一个项目可以拥有多个技术 + 子项目聚合」。**

---

## 1. Technology 数据结构（Phase 1 · PR1 核心）

### 1.1 模型（含 canonical id）

```rust
enum TechnologyCategory {
    Language,        // 编程语言：TypeScript / JavaScript / Python / Rust ...
    Runtime,         // 运行时：Node.js / Bun / Deno ...
    Framework,       // 框架：Vue / React / Express / NestJS / Fastify ...
    Library,         // 库：Pinia / JWT / Axios ...
    Database,        // 数据库：SQLite / PostgreSQL / MySQL / MongoDB / Redis ...
    BuildTool,       // 构建工具：Vite / Webpack / Rspack ...
    PackageManager,  // 包管理器：pnpm / npm / yarn / bun ...
    Platform,        // 平台/框架：uni-app / Tauri / Electron ...
}

struct Technology {
    id: String,                  // canonical id（稳定，跨 detector 统一）
    name: String,                // 展示名（人类可读）
    category: TechnologyCategory,
    ecosystem: Option<String>,   // 如 "javascript" / "python" / "rust"
}
```

### 1.2 Canonical id（P0-5，防重复/便于去重聚合）

不同 detector 对同一技术必须产出**同一个 id**：

```text
id = "nodejs"      name = "Node.js"
id = "postgresql"  name = "PostgreSQL"
id = "vue"         name = "Vue"
id = "express"     name = "Express"
```

dedupe / aggregation / filter / search / statistics 全部基于 `id`。

### 1.3 `ProjectMeta` 升级

```rust
// 替代原单值模型：{ language, framework }
struct ProjectMeta {
    technologies: Vec<Technology>,
}
```

### 1.4 实例

「藏蓝闪送-前端」：

```json
{
  "technologies": [
    { "id": "vue", "name": "Vue", "category": "framework", "ecosystem": "javascript" },
    { "id": "typescript", "name": "TypeScript", "category": "language", "ecosystem": "javascript" },
    { "id": "uniapp", "name": "uni-app", "category": "platform", "ecosystem": "javascript" },
    { "id": "vite", "name": "Vite", "category": "build_tool", "ecosystem": "javascript" },
    { "id": "pinia", "name": "Pinia", "category": "library", "ecosystem": "javascript" }
  ]
}
```

「藏蓝闪送-后端」：

```json
{
  "technologies": [
    { "id": "nodejs", "name": "Node.js", "category": "runtime" },
    { "id": "express", "name": "Express", "category": "framework" },
    { "id": "sqlite", "name": "SQLite", "category": "database" },
    { "id": "jwt", "name": "JWT", "category": "library" }
  ]
}
```

---

## 2. ProjectNode / Aggregate 模型（Phase 5 · PR3）

### 2.1 领域模型关系（P0-3，避免模型漂移）

**不引入第四套独立领域模型。**

```text
Detection layer
    ↓
DetectedProject        // 扫描阶段产物（transient）

Domain layer
    ↓
Project                // 数据库持久化领域模型（含 parent_id / kind / technologies）

ProjectNode            // 树形视图 / 扫描结果模型，不用作独立领域模型
```

- `DetectedProject`：检测层输出（transient）。
- `Project`：持久化领域模型，通过 `parent_id` 表达树。
- `ProjectNode`：**运行时树形视图**（scan tree），不落库为独立表；落库后靠 `Project.parent_id` 恢复树。

```text
DetectedProject
  ↓
Project
  ├── parent_id
  ├── kind
  └── technologies
```

### 2.2 聚合原则（P0-2，父级 derived，子级 Source of Truth）

> **子项目的 `technologies` 是 Source of Truth。父项目的 `technologies` 是聚合结果（derived summary）。**

```text
Child Project      → authoritative technologies
Parent Aggregate   → derived technologies summary
```

父级 `technologies` **只是派生缓存**，不作为唯一来源。前端要展示「Express 属于 backend、Vue 属于 frontend」，**必须通过 `parent.children[]` 重新计算**，而不是只靠父级 `technologies[]`。

聚合规则：

```text
Parent.technologies = Union(Child1.technologies, Child2.technologies, ...)   // 按 id 去重
```

### 2.3 期望结果（第一验收标准）

```text
藏蓝闪送
├── kind: aggregate
├── children: 2
└── technologies (derived):
    Vue, TypeScript, uni-app, Vite, Node.js, Express, SQLite

├── 藏蓝闪送-前端 (source of truth):
│   Vue, TypeScript, uni-app, Vite, Pinia
└── 藏蓝闪送-后端 (source of truth):
    Node.js, Express, SQLite, JWT
```

---

## 3. 项目类型 ProjectKind（P0-6，正式定义）

> 命名协调（PR1 回传）：沿用**现有实现** `Real / AggregatedRoot / Category`（serde `real / aggregated_root / category`），其语义与 Spec 原拟的 `Project / Aggregate / Directory` 对应，但已被数据库、前端 TS、既有测试锁定，**不改名**，仅在语义上做明确区分。

```rust
enum ProjectKind {
    Real,            // 自己拥有 manifest 的真项目（对应 Spec 概念 "Project"）
    AggregatedRoot,  // 聚合根，包含多个子项目（对应 Spec 概念 "Aggregate"）
    Category,        // 普通分类目录，不出现在"真正项目"统计（对应 Spec 概念 "Directory"）
}
```

- **Real**：自身含 manifest，是技术栈的 **Source of Truth**。
- **AggregatedRoot**：聚合根，包含多个子项目（children），其技术栈是 **derived summary**。
- **Category**：普通分类目录，默认不进入"项目"统计（除非 UI 特别展示）。

> 明确 `kind` 语义，避免后续 "aggregated_root 到底是什么意思" 模糊。

---

## 4. 有限递归与子项目发现（Phase 3-4 · PR2/PR3）

### 4.1 两个独立阶段（P0-4，关键修正）

**不能从"根目录无 Manifest 才递归"改为「根目录有 Manifest 就不递归」。**

```text
Project Root
   ├── Detect own manifest        （阶段 A：识别根自身技术栈）
   └── Detect children            （阶段 B：独立发现子项目）
        ├── workspace signal?     （pnpm-workspace.yaml / package.json workspaces）
        └── known project dirs?   （frontend/backend/client/server/web/api/app/apps/packages/services）
```

这样不会漏掉：

```text
my-app/
├── package.json          ← 根有 Manifest（阶段 A 识别）
├── frontend/package.json ← 仍是子项目（阶段 B 发现）
└── backend/package.json
```

### 4.2 递归深度

- 第一层：项目根。
- 第二层：若存在 workspace 信号 或 known project dirs，发现直接子项目。
- 第三层：**仅当**检测到 workspace / monorepo / package workspace 信号（`pnpm-workspace.yaml` 或 `package.json.workspaces`）才继续深入。

### 4.3 候选目录误报防护（P1-11）

候选目录（`app/`/`frontend/`/`backend/`...）**进入子项目检测前必须**：

```text
contains manifest
OR
contains workspace signal
```

仅因目录名是 `app` 不能当作子项目。例如 `docs/app/` 不能被误判为子项目。

---

## 5. Detection Registry（Phase 2 · PR2）

### 5.1 原则：不要 `if vue... if express...`

用注册表：

```text
Manifest
   ↓
Detector Registry
   ↓
Technology[]
```

### 5.2 目录结构

```text
core/parser/
├── mod.rs
├── registry.rs        // Detector Registry（注册 + 分发）
├── node.rs
├── rust.rs
├── go.rs
├── python.rs
└── detectors/
    ├── javascript.rs
    ├── python.rs
    ├── rust.rs
    ├── go.rs
    ├── database.rs
    └── infrastructure.rs
```

### 5.3 检测来源（以 package.json 为例）

```text
dependencies
devDependencies
scripts
engines
packageManager
```

分别映射到 category：

```text
Runtime         (engines + Node-specific 依赖 + lockfile/scripts)
Language        (TypeScript 等)
Framework       (Vue/React/Express...)
Library         (Pinia/JWT/Axios...)
Build Tool      (Vite/Webpack...)
Database        (依赖映射表，见 §5.6)
Package Manager (packageManager 字段 + lockfile，见 §5.5)
Platform        (uni-app/Tauri/Electron)
```

### 5.4 识别规则优先级（分层）

**P0（先做）— JavaScript/Node + 数据库**

```text
Node.js, JavaScript, TypeScript, Vue, React, Next.js, Nuxt, Svelte, Angular,
Express, NestJS, Fastify, Vite, Webpack, Rspack,
pnpm, npm, yarn, Bun, uni-app, Tauri, Electron

SQLite, PostgreSQL, MySQL, MongoDB, Redis
```

**P1（后续）**

```text
Python (FastAPI/Flask/Django), Go, Rust, Java (Spring Boot), Docker
```

**P2（延后）**

```text
大量小众框架和库（Registry 不一开始就写几千条）
```

### 5.5 Package Manager 识别优先级（P1-8）

```text
1. package.json "packageManager": "pnpm@..."（最高优先级）
2. lockfile：pnpm-lock.yaml → pnpm；package-lock.json → npm；yarn.lock → yarn；bun.lock/bun.lockb → bun
```

> `package.json` 没写 packageManager + 存在 `pnpm-lock.yaml` → 仍识别为 pnpm。

### 5.6 Runtime 多来源检测（P1-9）

Runtime 不能只靠 `engines.node`：

```text
engines.node
+
Node-specific manifest / dependencies（如 express 等）
+
lockfile / scripts
```

多来源综合判定，V0.4 只定义优先级（不做 Evidence 数值）。

### 5.7 Database 依赖映射表（P1-10，明确"dependency-level"语义）

```text
better-sqlite3 → SQLite
sqlite3        → SQLite
pg             → PostgreSQL
postgres       → PostgreSQL
mysql2         → MySQL
mongoose       → MongoDB
redis          → Redis
```

> **明确定义为 dependency-level detection**（"有依赖暗示该技术栈"），**不是**实际运行环境检测。避免用户质疑"我 package.json 有 redis 包，你凭什么说我部署了 Redis"。

### 5.8 强证据 vs 辅助证据

```text
强证据：manifest dependency（如 "sqlite3" → SQLite）
辅助证据：README / filename / source import（可信度低，延后）
```

不**只靠名字匹配**。

---

## 6. SQLite 数据库升级（Phase 6 · PR1）

### 6.1 保留旧字段 + 新增新字段（不破坏现有库）

```text
projects

id
parent_id             -- 新增（P0-1：父项目 id，NULL=顶层）
name
path
kind                  -- 新增（ProjectKind：real/aggregated_root/category）
language              -- 保留（旧数据兼容）
framework             -- 保留（旧数据兼容）
technologies_json     -- 新增（新模型，含 schema_version）
...
```

### 6.2 父子关系持久化（P0-1）

```text
parent_id = NULL   → 顶层项目
parent_id = 123    → 属于「藏蓝闪送」(id=123)
```

数据库才能真正表达：

```text
Aggregate
 ├── Project
 └── Project
```

> 否则 `children` 只是扫描期间的数据结构，数据库重启后丢失。**`parent_id` 是恢复树的唯一途径。**

### 6.3 technologies_json 版本控制（P1-12）

```json
{
  "schema_version": 1,
  "technologies": [ ... ]
}
```

Technology 结构未来变化（V0.5/V0.6）时，靠 `schema_version` 迁移旧 JSON。

### 6.4 兼容策略

```text
新数据：technologies_json（前端优先读取）
旧数据：language/framework（前端 fallback）
```

---

## 7. 前端展示升级（Phase 7 · PR4）

### 7.1 ProjectCard

```text
藏蓝闪送
Vue · uni-app · Express · SQLite
2 projects
```

**Library 默认不作为首页主技术栈**（P1-7）：

```text
Vue · Express · SQLite · Vite
+ 7 libraries
```

首页主技术栈只展示**架构级技术**：Language / Runtime / Framework / Database / BuildTool / PackageManager / Platform。Library 折叠为 `+N libraries`。

### 7.2 ProjectDetail

```text
Technology Stack

Frontend
Vue
TypeScript
uni-app
Vite
Pinia

Backend
Node.js
Express
SQLite
JWT
```

（前端栈/后端栈分区展示，**基于 `children` 重新计算**，不靠父级 derived technologies。）

### 7.3 旧数据兼容

```text
优先读 technologies_json；为空则 fallback language/framework。
```

---

## 8. 识别可解释性（P1-13，为 V0.5 Evidence 留接口，但暂不做功能）

V0.4 不做 Evidence/Confidence 数值，但**检测器内部保留来源**：

```rust
struct DetectionResult {
    technology: Technology,
    source_kind: SourceKind,   // ManifestDependency / ManifestField / Lockfile / Script / ...
}
```

- **不落库、不展示、不做 confidence**。
- 只为未来 V0.5 Evidence 保留 detector API，避免届时重新设计。

---

## 9. Evidence / Confidence（Phase 8 · 延后 v0.4 后半或 v0.5）

### 9.1 未来形态

```json
{
  "name": "Express",
  "category": "framework",
  "confidence": 0.99,
  "evidence": [
    { "source": "package.json", "reason": "dependency" }
  ]
}
```

### 9.2 意义

未来能回答"为什么你认为这是 Express？"，对 AI 分析尤其重要。

### 9.3 本阶段不做

先解决"一个项目拥有多个技术 + 子项目聚合"，Evidence/Confidence 后置。

---

## 10. V0.4 vs V0.5 边界（P0-5，避免逻辑冲突）

| | V0.4（做） | V0.5（做） |
|---|---|---|
| **Workspace/Monorepo Detection** | ✅ 是否存在 workspace 结构？有哪些直接子项目？ | 复杂 Monorepo Intelligence |
| Dependency graph | ❌ | ✅ workspace dependency graph / package graph / cross-package / turbo-nx graph |
| Evidence/Confidence | ❌（仅内部 source_kind） | ✅ |

> V0.4 只做"识别入口"（workspace 结构 + 直接子项目），复杂依赖图留给 V0.5。

---

## 11. 最终架构

```text
Workspace Scanner
        │
        ↓
Project Boundary Detector
        │
        ↓
Manifest Discovery（阶段 A：根自身 + 阶段 B：子项目）
        │
        ↓
Detection Registry → Detector → Technology[]
        │
        ↓
ProjectNode（transient scan tree）
        ├── technologies[]
        └── children[]
        │
        ↓
Aggregation（父级 derived summary，子级 source of truth）
        │
        ↓
SQLite（projects + parent_id + kind + technologies_json）
        │
        ↓
Vue UI
```

---

## 12. 分 PR 交付（P1-14，PR1 含 fixture 保护）

> 不要一个 PR 做完。每个 PR 有测试保护。

| PR | 内容 | 阶段 |
|---|---|---|
| **PR1** | Recognition Model：`Technology`（含 canonical id）/ `ProjectKind` / `ProjectMeta` / `Project`(parent_id+kind+technologies_json) / SQLite migration + **fixture framework** + serialization tests | Phase 1/6 |
| **PR2** | Technology Detection Engine：Detection Registry + package.json/Cargo.toml/go.mod/pyproject.toml + P0 识别规则（含 canonical id、database 映射、packageManager/runtime 优先级）+ detector 内部 source_kind | Phase 2 |
| **PR3** | Boundary + Aggregate：藏蓝闪送前后端子项目 + 有限递归 + 父级 derived 聚合 + 候选目录误报防护 | Phase 3-5 |
| **PR4** | UI + E2E/regression：ProjectCard 多 badge（架构级 + libraries 折叠）+ ProjectDetail 前后端分区 + 测试矩阵 | Phase 7 |

---

## 13. 测试矩阵（fixtures/）

> 不用只测「藏蓝闪送」；建立小型真实测试矩阵，每次改识别器跑 `cargo test`。

```text
fixtures/
├── vue-project/
├── react-project/
├── express-project/
├── rust-project/
├── go-project/
├── python-project/
├── frontend-backend/     // 即藏蓝闪送结构
├── root-plus-children/   // 根有 package.json + frontend/backend 子项目（P0-4 场景）
├── monorepo/
├── unknown-folder/
└── empty-folder/
```

> 防止"修了 Express，把 Vue 识别坏了"。

---

## 14. 验收案例（藏蓝闪送）

```text
藏蓝闪送

Vue · uni-app · Express · SQLite
2 projects

├── 前端
│   Vue · TypeScript · uni-app · Vite
│
└── 后端
    Node.js · Express · SQLite
```

- 父级 `technologies` 是 **derived**。
- 子项目 `technologies` 是 **Source of Truth**。
- 前端栈/后端栈分区靠 `children` 计算，不靠父级 derived。
