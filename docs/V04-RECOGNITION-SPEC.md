# YDevSphere · V0.4 识别引擎规格（V04-RECOGNITION-SPEC）

> 维护人：总负责人 · 状态：规格定稿（待按 PR1-4 实现）
> 依据：`V04-RECOGNITION-AUDIT` 诊断报告 + v0.4 方向讨论
> 定位：把 YDevSphere 识别能力从 v0.3「单项目单技术」升级为 v0.4「**项目结构 + 多技术栈 + 子项目聚合**」的**第一代识别引擎**。
>
> **核心原则**：这不是"多加几条识别规则"，而是**建立识别引擎**。渐进式重构，不推倒重写。

---

## 0. 目标与原则

### 0.1 能力升级目标

```text
V0.3                     V0.4                     V0.5+
单目录                  → 项目节点               → Monorepo
language + framework    → 多个技术               → Evidence / Confidence
项目卡片                → 子项目                 → AI 项目理解
                        → 聚合
                        → 更准确的项目理解
```

### 0.2 迭代原则

- **渐进式重构**：不一次推倒整个 Scanner。
- **分 PR 交付**：PR1 → PR2 → PR3 → PR4。
- **先定 Spec**：本文档即规格，实现严格对齐。
- **第一验收案例**：藏蓝闪送（前端 Vue/uni-app + 后端 Express/SQLite）。

### 0.3 本阶段明确不做

- ❌ Evidence / Confidence
- ❌ AI 判断
- ❌ 无限递归
- ❌ 一次性追求识别 100+ 框架

**先解决「一个项目可以拥有多个技术」。**

---

## 1. Technology 数据结构（Phase 1 · PR1 核心）

### 1.1 模型

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
    name: String,
    category: TechnologyCategory,
    ecosystem: Option<String>,   // 如 "javascript" / "python" / "rust"
}
```

### 1.2 `ProjectMeta` 升级

```rust
// 替代原单值模型：
// struct ProjectMeta { language: Option<String>, framework: Option<String> }
struct ProjectMeta {
    technologies: Vec<Technology>,
}
```

### 1.3 实例

「藏蓝闪送-前端」：

```json
{
  "technologies": [
    { "name": "Vue", "category": "framework", "ecosystem": "javascript" },
    { "name": "TypeScript", "category": "language", "ecosystem": "javascript" },
    { "name": "uni-app", "category": "platform", "ecosystem": "javascript" },
    { "name": "Vite", "category": "build_tool", "ecosystem": "javascript" },
    { "name": "Pinia", "category": "library", "ecosystem": "javascript" }
  ]
}
```

「藏蓝闪送-后端」：

```json
{
  "technologies": [
    { "name": "Node.js", "category": "runtime" },
    { "name": "Express", "category": "framework" },
    { "name": "SQLite", "category": "database" },
    { "name": "JWT", "category": "library" }
  ]
}
```

---

## 2. ProjectNode / Aggregate 模型（Phase 5 · PR3）

### 2.1 项目节点

```text
ProjectNode
├── technologies: Vec<Technology>
├── children: Vec<ProjectNode>   // 子项目（聚合根/分类目录下）
```

### 2.2 聚合规则

聚合根（Aggregate）不是"无技术栈的空父目录"，而是**聚合所有子项目技术栈**：

```text
Parent.technologies = Union(Child1.technologies, Child2.technologies, ...)   // 去重
```

实例：

```text
Child 1 (前端): Node, Vue, TypeScript
Child 2 (后端): Node, Express, SQLite
─────────────────────────────────────
Parent (藏蓝闪送):
  Node, Vue, TypeScript, Express, SQLite
```

### 2.3 期望结果（第一验收标准）

```text
藏蓝闪送
├── kind: aggregate
├── children: 2
└── technologies:
    Vue, TypeScript, uni-app, Vite, Node.js, Express, SQLite
```

子项目：

```text
藏蓝闪送-前端:  Vue, TypeScript, uni-app, Vite, Pinia
藏蓝闪送-后端:  Node.js, Express, SQLite, JWT
```

---

## 3. Detection Registry（Phase 2 · PR2）

### 3.1 原则：不要 `if vue... if express... if sqlite...`

用注册表：

```text
Manifest
   ↓
Detector Registry
   ↓
Technology[]
```

### 3.2 目录结构

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

### 3.3 检测来源（以 package.json 为例）

```text
dependencies
devDependencies
scripts
engines
packageManager
```

分别检测：

```text
Runtime        (engines)
Language       (TypeScript 等)
Framework      (Vue/React/Express...)
Library        (Pinia/JWT...)
Build Tool     (Vite/Webpack...)
Database       (SQLite/Postgres...)
Package Manager(pnpm/npm/yarn/bun)
Platform       (uni-app/Tauri/Electron)
```

### 3.4 识别规则优先级（分层，不一次全做）

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

### 3.5 强证据 vs 辅助证据（重要原则）

```text
强证据：manifest dependency（如 "sqlite3" → SQLite）
辅助证据：README / filename / source import（可信度低，延后）
```

不**只靠名字匹配**。

---

## 4. 子项目递归与聚合规则（Phase 3-4 · PR2/PR3）

### 4.1 有限递归（不做无限递归）

**第一层**：项目根目录 `/`。

**第二层**：若根目录无 Manifest，寻找候选目录：

```text
frontend, backend, client, server, web, api, app, apps, packages, services
```

**第三层**：仅当检测到 workspace / monorepo / package workspace 信号才继续深入：

```text
pnpm-workspace.yaml
或 package.json 的 "workspaces": [...]
```

> 兼顾准确性 + 扫描性能。

### 4.2 Scanner 挂载

- `project_meta_if_manifest`：目录本身含清单 → 解析 `technologies`。
- 聚合根 / 分类目录：聚合子项目技术栈，**不再硬编码 `None`**（PR3 修复）。

---

## 5. SQLite 数据库升级（Phase 6 · PR1）

### 5.1 保留旧字段 + 新增新字段（不破坏现有库）

```text
projects

id
name
path
kind
language      -- 保留（旧数据兼容）
framework     -- 保留（旧数据兼容）
technologies_json   -- 新增（新模型）
...
```

### 5.2 兼容策略

```text
新数据：technologies_json = [ ... ]（前端优先读取）
旧数据：language/framework（前端 fallback）
```

> 不需要一次性破坏现有数据库；旧数据可正常运行。

---

## 6. 前端展示升级（Phase 7 · PR4）

### 6.1 ProjectCard

```text
藏蓝闪送
Vue · uni-app · Express · SQLite   +2
```

（多技术栈 badge 流，溢出显示 `+N`。）

### 6.2 ProjectDetail

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

（支持"前端栈 / 后端栈"分区展示。）

### 6.3 旧数据兼容

```text
优先读 technologies_json；为空则 fallback language/framework。
```

---

## 7. Evidence / Confidence（Phase 8 · 延后 v0.4 后半或 v0.5）

### 7.1 未来形态

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

### 7.2 意义

未来能回答"为什么你认为这是 Express？"，对 AI 分析尤其重要。

### 7.3 本阶段不做

先解决"一个项目可以拥有多个技术"，Evidence/Confidence 后置。

---

## 8. 分 PR 交付

> 不要一个 PR 做完。

| PR | 内容 | 阶段 |
|---|---|---|
| **PR1** | Recognition Model：`Technology` / `ProjectMeta` / `ProjectNode` / SQLite migration（technologies_json） | Phase 1/6 |
| **PR2** | Technology Detection Engine：Detection Registry + package.json/Cargo.toml/go.mod/pyproject.toml + P0 识别规则扩充 | Phase 2 |
| **PR3** | Aggregate Recognition：藏蓝闪送前后端子项目 + 技术栈聚合 + 有限递归 | Phase 3-5 |
| **PR4** | UI + Regression：ProjectCard 多 badge + ProjectDetail 分区 + 测试矩阵 | Phase 7 |

---

## 9. 测试矩阵（fixtures/）

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
├── monorepo/
├── unknown-folder/
└── empty-folder/
```

> 防止"修了 Express，把 Vue 识别坏了"。

---

## 10. 最终架构

```text
Workspace Scanner
        │
        ↓
Project Boundary Detector
        │
        ↓
Manifest Discovery
        │
        ↓
Detection Registry
        │
        ↓
Technology[]
        │
        ↓
ProjectNode
        ├── technologies[]
        └── children[] → technologies[] → children[]
        │
        ↓
Aggregation
        │
        ↓
SQLite
        │
        ↓
Vue UI
```

> 这条链路即 YDevSphere 的 **Project Intelligence Engine**。
