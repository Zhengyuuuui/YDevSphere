YDevSphere 最大风险不是技术，而是**第一版做成一个复杂 IDE**。

我们的第一阶段目标应该非常明确：

> **证明 YDevSphere 能够理解用户电脑里的开发项目。**

所以 MVP 不做 AI Agent、不做云同步、不做代码编辑。

先完成：

**扫描 → 索引 → 展示 → 生成项目记忆**

---

# YDevSphere MVP v0.1 功能规划

## 总览

```
YDevSphere v0.1

P0:
基础项目管理核心

P1:
开发者体验增强

P2:
AI智能能力
```

---

# P0（必须完成）

目标：

> 一个可以安装使用的本地开发项目管理工具。

---

# P0-1 项目目录扫描

优先级：

★★★★★

功能：

用户选择目录：

例如：

```
~/Projects
~/Code
~/Workspace
```

YDevSphere 扫描：

```
Projects

├ YunexTools
├ NewAPI
├ MindOS
├ LifeKline
└ TestProject
```

识别：

* 文件夹
* 项目入口文件
* 技术栈

支持：

第一版：

```
package.json
pnpm-lock.yaml

go.mod

Cargo.toml

requirements.txt

pom.xml

composer.json
```

输出：

```
Project
|
├ name
├ path
├ language
└ framework
```

---

# P0-2 项目列表 Dashboard

核心页面。

类似：

VS Code Welcome：

```
--------------------------------

YDevSphere


My Projects


┌─────────────┐

Yunex Tools

Vue3
TypeScript


└─────────────┘



┌─────────────┐

NewAPI

Go
SQLite


└─────────────┘


--------------------------------
```

功能：

* 项目卡片
* 搜索
* 排序
* 最近打开

---

# P0-3 SQLite 本地数据库

建立：

```
.ydevsphere/database.sqlite
```

表：

## projects

```sql
id

name

path

language

framework

created_at

updated_at
```

## scan_history

```sql
id

project_id

scan_time

status
```

---

# P0-4 .ydevsphere 项目配置

首次扫描：

询问：

```
是否启用项目记忆？

[启用]

[跳过]
```

启用：

创建：

```
project

├ .ydevsphere

    ├ project.json

    └ metadata.json

```

例如：

project.json:

```json
{
"name":"YDevSphere",
"stack":[
"Vue3",
"Rust",
"SQLite"
],
"packageManager":"pnpm"
}
```

---

# P0-5 项目详情页

点击项目：

进入：

```
Project Detail
```

展示：

```
YDevSphere


Path:

~/Projects/YDevSphere


Technology:


Vue3

TypeScript

Tauri


Last Scan:

2 minutes ago

```

---

# P0-6 基础设置

包括：

```
Settings


Scan Directory

Language

Theme

Database Location

Privacy
```

---

# P0完成标准

达到：

用户：

下载安装

↓

选择代码目录

↓

自动出现项目列表

↓

点击查看项目信息

这就是第一个可用版本。

---

# P1（增强体验）

目标：

让开发者觉得：

「这个工具懂我的项目」

---

# P1-1 Git分析

读取：

```
.git
```

显示：

```
Git:


Branch:
main


Last Commit:
xxxxx


Last Update:
2 hours ago


Status:
Clean

```

Rust:

使用：

```
git2-rs
```

---

# P1-2 项目启动管理

例如：

检测：

package.json

发现：

```json
{
"scripts":{
"dev":"vite"
}
}
```

显示：

按钮：

```
▶ Run
```

执行：

```
pnpm dev
```

---

# P1-3 技术栈识别增强

识别：

Frontend:

```
Vue
React
Angular
Svelte
```

Backend:

```
Go
Node
Python
Java
Rust
```

Database:

```
Postgres
SQLite
MySQL
MongoDB
```

Docker:

```
docker-compose.yml
Dockerfile
```

---

# P1-4 文件监听

使用 Rust：

```
notify crate
```

监听：

```
项目变化
```

自动更新：

SQLite索引。

---

# P1-5 最近项目

类似：

VS Code：

```
Recent Projects
```

记录：

* 打开时间
* 使用次数

---

# P1完成标准

用户感觉：

```
这不是文件管理器

它知道我的项目是什么
```

---

# P2（AI能力）

目标：

形成差异化。

---

# P2-1 AI项目分析

输入：

```
project.json

README

package.json

Git信息
```

输出：

例如：

```
这个项目是一个Vue3后台管理系统。

架构：

Frontend:
Vue3 + Vite

Backend:
Express

Database:
SQLite


建议：

1.
升级TypeScript配置

2.
优化组件结构

```

---

# P2-2 AI项目记忆

生成：

```
.ydevsphere/memory.md
```

内容：

```
项目目标：

这是一个AI API中转服务。


技术决策：

使用SQLite原因：

...


历史：

2026-08:
重构认证模块

```

---

# P2-3 AI Chat With Project

类似：

Cursor：

但是不是编辑代码。

而是：

```
Ask about project


Q:
这个项目怎么部署？


AI:

根据项目结构回答。
```

---

# P2-4 MCP Server

未来：

让：

Claude Code

Cursor

OpenCode

调用：

```
YDevSphere Context
```

例如：

AI:

```
读取我的所有项目关系
```

---

# 暂不开发功能（明确限制）

## ❌ 代码编辑器

原因：

竞争 Cursor。

---

## ❌ 云同步

原因：

增加账号、服务器、隐私复杂度。

---

## ❌ 团队协作

原因：

不是核心价值。

---

## ❌ 在线IDE

完全偏离。

---

# MVP开发顺序

我建议：

## Sprint 1

基础工程

```
Tauri2初始化

Vue3 UI

Rust Command

SQLite连接
```

---

## Sprint 2

Scanner

完成：

```
选择目录

扫描项目

写入SQLite
```

---

## Sprint 3

Dashboard

完成：

```
项目列表

详情页

搜索
```

---

## Sprint 4

项目记忆

完成：

```
.ydevsphere

project.json
```

---

## Sprint 5

Git + AI

---

# 最终 MVP v0.1

功能列表：

| 功能           | 版本 |
| ------------ | -- |
| Tauri桌面程序    | P0 |
| Vue界面        | P0 |
| 目录扫描         | P0 |
| 项目识别         | P0 |
| SQLite索引     | P0 |
| 项目列表         | P0 |
| 项目详情         | P0 |
| project.json | P0 |
| Git状态        | P1 |
| 启动项目         | P1 |
| 文件监听         | P1 |
| AI分析         | P2 |
| AI记忆         | P2 |
| MCP          | P2 |

---