<div align="center">

<img src="./assets/logo.png" width="120" />

# YDevSphere

### Your local-first AI developer workspace intelligence layer.

Understand your projects.
Organize your development universe.

[Download](https://github.com/Zhengyuuuui/YDevSphere/releases/latest)
·
[Documentation](./docs)
·
[中文](./README.zh-CN.md)

<br />

![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue)
![Tauri](https://img.shields.io/badge/Tauri-2-orange)
![Rust](https://img.shields.io/badge/Rust-stable-black)
![License](https://img.shields.io/badge/license-MIT-green)

</div>

# What is YDevSphere?

Modern developers work across dozens or hundreds of projects.

Repositories are scattered.
Technologies are mixed.
Project context disappears over time.

**YDevSphere builds a local intelligence layer between developers and their projects.**

It scans, indexes, and understands your workspace, turning your local projects into an organized development knowledge system.

```
Local Projects
  ↓
Project Structure
  ↓
Technology Recognition
  ↓
Git Context
  ↓
Project Memory
  ↓
AI Context
```

# Why YDevSphere?

YDevSphere is not:

- ❌ An IDE
- ❌ A code editor
- ❌ A Git client
- ❌ A cloud project manager

It is:

- ✅ A developer workspace intelligence system
- ✅ A local project knowledge layer
- ✅ A foundation for future AI-assisted development

# Features

## 🧠 Workspace Intelligence

Automatically discover and understand your projects.

YDevSphere analyzes:

- Project structures
- Frameworks
- Languages
- Package managers
- Development environments

## 🔍 Smart Project Scanner

A multi-layer project discovery engine.

Features:

- Project boundary detection
- Aggregate root recognition
- Technology stack detection
- Lazy directory loading
- Health scoring

## 🛠 Editor Intelligence

Understand your development environment.

Supports:

- VS Code family editors
- AI coding editors
- Custom imported applications
- Default editor preferences

## 🔒 Local First

Your projects stay on your machine.

All application data is stored locally:

```
~/.ydevsphere/
```

No cloud synchronization.
No remote upload.

## 🌎 Multi-language

Built-in internationalization:

- English
- 中文

# Download

Latest Release:

## v0.3.0

macOS Apple Silicon:

⬇️ `YDevSphere_0.3.0_aarch64.dmg`

Download:

https://github.com/Zhengyuuuui/YDevSphere/releases/latest

# How It Works

```
Developer Workspace
    ↓
Scanner Engine
    ↓
Project Metadata
    ↓
Technology Intelligence
    ↓
AI Understanding Layer
```

# Technology Stack

| Layer | Technology |
| --- | --- |
| Desktop Runtime | Tauri 2 |
| Frontend | Vue 3 · TypeScript · Vite |
| Styling | TailwindCSS |
| State | Pinia |
| Backend | Rust |
| Database | SQLite |
| Communication | Tauri IPC |

# Architecture

```
Frontend (Vue 3)
  |
Tauri invoke()
  |
Rust Core
  |
Scanner  ·  Parser  ·  Database  ·  Editor Detection  ·  Git Analyzer  ·  Project Memory
```

Architecture principles:

- Pure Rust core without Tauri dependency
- Frontend communicates only through IPC
- No direct filesystem access from frontend
- Default read-only behavior

# Project Structure

```text
src/                          # Vue 3 frontend
  ├── pages/                  # Welcome / Overview / Projects / Recent / Settings / ProjectDetail
  ├── components/             # Sidebar, ProjectTable, DirTree, OpenActions, etc.
  ├── stores/                 # Pinia stores (editor/git/scanner/settings, etc.)
  ├── api/                    # Tauri invoke() wrappers
  └── types/                  # TS types aligned with Rust core/models
src-tauri/
  ├── src/commands/           # Thin layer: param parsing + forwarding only
  └── src/core/               # Pure Rust business core (no `use tauri`)
      ├── scanner/            # Scan boundary recognition
      ├── parser/             # Tech stack parsing
      ├── database/           # SQLite connection / CRUD / migration
      ├── editor/             # Editor discovery / detection / opening
      ├── git/                # Git read-only analysis
      └── memory/             # Project memory
docs/                         # Development order, proposals, audits, handoff docs
```

# Development

Requirements:

- Node >= 20
- pnpm
- Rust stable

Install:

```bash
pnpm install
```

Development:

```bash
pnpm tauri dev
```

Production build:

```bash
pnpm build
cd src-tauri
cargo build
```

Tests:

```bash
cd src-tauri
cargo test
```

# Roadmap

- ✅ v0.1 Foundation
- ✅ v0.2 Workspace system
- ✅ v0.3 Editor intelligence & onboarding

Future:

- File watcher
- Advanced workspace analytics
- AI project understanding
- MCP integration

# License

MIT License

© 2026 Zhengyuuuui

YDevSphere is currently open source under MIT.
Future versions may introduce additional commercial features.
