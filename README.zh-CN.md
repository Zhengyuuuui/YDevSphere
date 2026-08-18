<div align="center">

<img src="./assets/logo.png" width="120" />

# YDevSphere

### 本地优先的 AI 开发者工作区智能层

理解你的项目。
组织你的开发世界。

[下载安装](https://github.com/Zhengyuuuui/YDevSphere/releases/latest)
·
[开发文档](./docs)
·
[English](./README.md)

<br />

![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue)
![Tauri](https://img.shields.io/badge/Tauri-2-orange)
![Rust](https://img.shields.io/badge/Rust-stable-black)
![License](https://img.shields.io/badge/license-MIT-green)

</div>

# 什么是 YDevSphere？

现代开发者拥有越来越多的项目：

- 本地仓库不断增长
- 技术栈越来越复杂
- 项目上下文逐渐丢失

**YDevSphere 在开发者与项目之间建立了一层本地智能层。**

它扫描、索引并理解你的项目资产，
将分散的代码仓库转换为个人开发知识系统。

```
本地项目
↓
项目结构
↓
技术识别
↓
Git 上下文
↓
项目记忆
↓
AI 上下文
```

# 为什么需要 YDevSphere？

YDevSphere 不是：

- ❌ IDE
- ❌ 代码编辑器
- ❌ Git 客户端
- ❌ 云端项目管理工具

它是：

- ✅ 开发者工作区智能系统
- ✅ 本地项目知识层
- ✅ 面向未来 AI 开发的基础设施

# 核心功能

## 🧠 工作区智能

自动发现并理解你的项目。

分析：

- 项目结构
- 技术栈
- 编程语言
- 包管理器
- 开发环境

## 🔍 智能项目扫描

多层项目识别引擎。

支持：

- 项目边界识别
- 聚合根判断
- 技术栈分析
- 懒加载目录
- 项目健康评分

## 🛠 编辑器智能发现

理解你的开发工具环境。

支持：

- VS Code 系编辑器
- AI 编辑器
- 手动导入应用
- 默认编辑器偏好

## 🔒 本地优先

你的项目永远留在你的设备上。

数据存储：

```
~/.ydevsphere/
```

不上传。
不依赖云端。

## 🌎 双语支持

支持：

- 中文
- English

# 下载

当前版本：

## v0.3.0

macOS Apple Silicon：

⬇️ `YDevSphere_0.3.0_aarch64.dmg`

下载：

https://github.com/Zhengyuuuui/YDevSphere/releases/latest

# 工作流程

```
开发者工作区
↓
扫描引擎
↓
项目元数据
↓
技术智能层
↓
AI 理解层
```

# 技术栈

| 层 | 技术 |
| --- | --- |
| 桌面运行时 | Tauri 2 |
| 前端 | Vue 3 · TypeScript · Vite |
| 样式 | TailwindCSS |
| 状态管理 | Pinia |
| 后端 | Rust |
| 数据库 | SQLite |
| 通信 | Tauri IPC |

# 架构

```
前端 (Vue 3)
  |
Tauri invoke()
  |
Rust Core
  |
Scanner  ·  Parser  ·  Database  ·  Editor Detection  ·  Git Analyzer  ·  Project Memory
```

架构原则：

- 纯 Rust core，不依赖 Tauri
- 前端仅通过 IPC 通信
- 前端不直接访问文件系统
- 默认只读行为

# 项目结构

```text
src/                          # Vue 3 前端
  ├── pages/                  # Welcome / Overview / Projects / Recent / Settings / ProjectDetail
  ├── components/             # Sidebar、ProjectTable、DirTree、OpenActions 等
  ├── stores/                 # Pinia stores（editor/git/scanner/settings 等）
  ├── api/                    # Tauri invoke() 封装
  └── types/                  # 与 Rust core/models 对齐的 TS 类型
src-tauri/
  ├── src/commands/           # 薄壳层：参数解析 + 转发
  └── src/core/               # 纯 Rust 业务核心（禁止 use tauri）
      ├── scanner/            # 扫描边界识别
      ├── parser/             # 技术栈解析
      ├── database/           # SQLite 连接 / CRUD / 迁移
      ├── editor/             # 编辑器发现 / 检测 / 打开
      ├── git/                # Git 只读分析
      └── memory/             # 项目记忆
docs/                         # 开发顺序、方案、审计、交接等文档
```

# 开发

环境：

- Node >= 20
- pnpm
- Rust stable

安装：

```bash
pnpm install
```

启动：

```bash
pnpm tauri dev
```

构建：

```bash
pnpm build
cd src-tauri
cargo build
```

测试：

```bash
cd src-tauri
cargo test
```

# 路线规划

- ✅ v0.1 基础架构
- ✅ v0.2 工作区系统
- ✅ v0.3 编辑器智能与引导体验

未来：

- 文件监听
- 工作区分析
- AI 项目理解
- MCP 集成

# License

MIT License

© 2026 Zhengyuuuui

当前版本采用 MIT 开源协议。
未来版本可能加入商业功能。
