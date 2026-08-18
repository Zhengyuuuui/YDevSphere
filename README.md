# YDevSphere

> A local-first AI developer workspace intelligence application.

YDevSphere 运行在本地桌面端，扫描、索引并理解开发者的项目资产，为开发者提供工作区智能分析能力。

它不是 IDE、代码编辑器或云端项目管理工具，而是位于以下链路中的 **Developer Intelligence Layer**：

```text
本地代码 → 项目结构 → 技术栈 → Git 历史 → 项目记忆 → AI 上下文
```

## 核心特性

- **本地优先（local-first）**：所有数据默认存储在本地（`~/.ydevsphere/`），不上传任何远程服务器，隐私可控。
- **智能项目扫描**：四类项目边界识别（真项目 / 聚合根 / 分类目录 / 普通目录），健康度评分，按需懒加载目录树。
- **多工作区模型**：Documents / Desktop / 手动目录并存，一键导入与切换。
- **Git 只读分析**：分支、最近提交、工作区状态、最近更新，全程只读不修改任何 Git 状态。
- **项目记忆**：`.ydevsphere/project.json` 记录技术栈与包管理器元数据。
- **编辑器智能发现**：VS Code 系（Fork）自动识别、手动导入任意应用、默认编辑器偏好持久化，支持 AI 编辑器（ChatGPT/Claude/Codex 等）分类。
- **i18n**：中 / English 双语界面。

## 技术栈

| 层 | 技术 |
|---|---|
| 前端 | Vue 3 · TypeScript · Vite · TailwindCSS · Pinia · Vue Router |
| 桌面运行时 | Tauri 2 |
| 后端 | Rust |
| 数据库 | SQLite（`rusqlite` bundled） |
| IPC | 前端统一通过 Tauri `invoke()` 调用后端 |

## 环境要求

- Node >= 20
- pnpm
- Rust（stable）

## 快速开始

安装依赖：

```bash
pnpm install
```

仅启动前端 dev server（无原生窗口）：

```bash
pnpm dev
```

启动完整 Tauri 桌面应用：

```bash
pnpm tauri dev
```

生产构建：

```bash
pnpm build               # 前端类型检查 + 生产构建
cd src-tauri && cargo build   # Rust 构建
cd src-tauri && cargo test    # 后端单元测试
```

> 推荐 IDE：VS Code + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## 项目结构

```text
src/                          # Vue 3 前端
  ├── pages/                  # Welcome / Overview / Projects / Recent / Settings / ProjectDetail
  ├── components/             # Sidebar、ProjectTable、DirTree、OpenActions 等
  ├── stores/                 # Pinia stores（editor/git/scanner/settings 等）
  ├── api/                    # Tauri invoke() 封装
  └── types/                  # 与 Rust core/models 对齐的 TS 类型
src-tauri/
  ├── src/commands/           # 薄壳层：参数解析 + 转发，不放业务逻辑
  └── src/core/               # 纯 Rust 业务核心（禁止 use tauri）
      ├── scanner/            # 扫描边界识别
      ├── parser/             # 技术栈解析
      ├── database/           # SQLite 连接 / CRUD / 迁移
      ├── editor/             # 编辑器发现 / 检测 / 打开
      ├── git/                # Git 只读分析
      └── memory/             # 项目记忆
docs/                         # 开发顺序、方案、审计、交接等文档
```

## 架构约束

- `src-tauri/src/core/` 禁止依赖 Tauri，保持纯 Rust，便于未来 CLI / MCP 复用。
- `commands/` 只做参数解析与转发，不放置业务逻辑。
- 前端不直接访问文件系统 / 执行 shell / 操作 SQLite，统一通过 `invoke()`。
- 默认只读；持久化仅写入 `~/.ydevsphere/` 下的应用数据。

## 开发文档

- [`docs/DEVELOPMENT_ORDER.md`](docs/DEVELOPMENT_ORDER.md)：开发顺序、Sprint 交付记录与全局总览。
- [`docs/V03_HANDOFF.md`](docs/V03_HANDOFF.md)：v0.3 交接文档（接手入口）。
- `docs/` 下其他方案 / 审计 / Backlog 文档。

## 路线规划

- [x] v0.1：基础工程、Scanner、数据库、Dashboard
- [x] v0.2：前端重构、响应式布局、Scanner 迭代、多工作区、编辑器动态发现
- [x] v0.3：编辑器发现误判治理、手动导入、Welcome 引导、重置
- [ ] 文件监听、Overview 统计接口
- [ ] AI 项目分析 / 记忆 / MCP

## License

[MIT](LICENSE) © 2026 Zhengyuuuui

> 说明：当前版本以 MIT 开源。后续将引入加密与授权边界，调整为商业授权模型。
