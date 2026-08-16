# PRD.md

```md
# YDevSphere Product Requirements Document

Version: 0.1.0

Product Name:
YDevSphere

Product Type:
Local-first AI Developer Workspace Intelligence Tool


---

# 1. 产品定位

## 一句话定义

YDevSphere 是一个运行在用户本地的 AI 开发者工作空间管理工具，通过扫描、分析和理解开发项目，为开发者建立个人代码资产知识库。


## 核心理念

不是：

- IDE
- 代码编辑器
- Git客户端
- 云端项目管理系统


而是：

Developer Intelligence Layer


连接：

代码

↓

项目结构

↓

技术栈

↓

Git历史

↓

AI上下文


帮助开发者理解和管理自己的软件资产。


---

# 2. 产品目标


## Phase 1 MVP

目标：

建立本地开发项目索引系统。


核心能力：

- 扫描指定开发目录
- 自动识别项目类型
- 生成项目列表
- 生成 project.json
- 保存项目元数据
- 展示项目 Dashboard


---

## Phase 2


增加：

- Git状态分析
- 项目健康度分析
- 启动项目
- Docker状态检测
- 技术栈分析


---

## Phase 3


增加：

- AI项目分析
- 项目长期记忆
- MCP Server
- AI Agent调用


---

## Phase 4


云能力：

- 多设备同步
- 团队共享
- 云端AI分析
- 企业版本


---

# 3. 技术架构


## Desktop Framework

Tauri 2


## Frontend

Vue 3

TypeScript

Vite

TailwindCSS


## Backend Core

Rust


负责：

- 文件扫描
- 系统调用
- Git分析
- 文件监听
- 权限管理


## Database

SQLite


用途：

本地索引数据库


不作为：

云端数据库替代方案。


---

# 4. 系统架构


```

YDevSphere Desktop

```
    |

 Tauri 2


    |
```

---

|               |

Vue3          Rust

```
            |

    ----------------

    |       |       |

Scanner   Git   Parser


            |

          SQLite


            |

        .ydevsphere/
```

```


---

# 5. 数据设计


## SQLite


### projects

保存项目基础信息


字段：

id

name

path

language

framework

created_at

updated_at



---

### files


文件索引


字段：

id

project_id

path

hash

size

modified_time



---

### analysis


AI分析结果


字段：

project_id

summary

architecture

suggestion

updated_at



---

# 6. 项目文件结构


用户项目：

```

MyProject

├ src

├ package.json

├ .git

└ .ydevsphere

```
├ project.json

├ analysis.md

└ memory.json
```

````


---

# 7. project.json设计


示例：

```json

{
"name":"YDevSphere",
"type":"vue",
"framework":"Vue3",
"language":"TypeScript",
"packageManager":"pnpm",
"git":{
 "branch":"main",
 "lastCommit":"xxxx"
}
}

````

---

# 8. 用户流程

首次启动：

1.

欢迎页面

2.

选择开发目录

例如：

```
~/Projects
~/Code
~/Workspace
```

3.

扫描

4.

生成项目列表

5.

用户查看分析结果

---

# 9. UI设计原则

遵循：

Apple Human Interface Guidelines

设计关键词：

* 简洁
* 高信息密度
* 专业开发者工具

禁止：

* 游戏化UI
* 大面积渐变
* 玻璃效果
* 复杂动画

---

# 10. AI设计原则

AI不是核心入口。

AI基于：

项目上下文

输入：

* 项目结构
* README
* package.json
* Git信息

输出：

* 项目介绍
* 架构分析
* 优化建议

---

# 11. 商业方向

免费：

本地项目管理

Pro:

* AI分析
* 云同步
* 多设备

Team:

* 团队知识库
* 企业管理

````

---

# RESTRICTIONS.md

```md
# YDevSphere Development Restrictions


Version:
0.1.0


---

# 1. 数据隐私限制


## 禁止默认上传用户代码


YDevSphere 默认：

所有数据只保存在用户本机。


禁止：

- 自动上传代码
- 自动同步项目文件
- 未授权发送源码


---

# 2. 文件访问限制


禁止：

扫描整个磁盘。


禁止：

默认访问：

/System

/Windows

用户隐私目录



必须：

用户主动选择目录。


---

# 3. 写入限制


默认：

Read Only。


禁止：

自动修改用户代码。


允许：

创建：

````

.ydevsphere/

```


但是：

必须用户授权。


---

# 4. AI限制


禁止：

默认发送完整代码到第三方AI。


必须：

用户明确配置：

- API Key
- Provider
- 数据范围


---

# 5. 性能限制


扫描要求：

不能阻塞UI。


必须：

Rust后台线程执行。


禁止：

Frontend直接扫描文件。


---

# 6. 数据库限制


SQLite:

只保存：

- metadata
- index
- cache


禁止：

保存：

完整代码文件。


---

# 7. 架构限制


Frontend:

只能负责：

UI


Backend:

负责：

系统能力。



禁止：

Vue直接调用系统API。



---

# 8. 跨平台限制


目标：

macOS

Windows

Linux


禁止：

依赖macOS独有API。


---

# 9. 安全原则


任何危险操作：

必须：

用户确认。


包括：

- 写文件
- 执行命令
- 修改项目


---

# 10. 产品原则


YDevSphere 永远优先：

用户数据控制权


Local First

Privacy First

Developer First

```

---

我建议后续项目初始化就按照这个结构：

```
ydevsphere/

├ apps/
│
├ frontend/
│
├ src-tauri/
│
├ docs/
│   ├ PRD.md
│   └ RESTRICTIONS.md
│
├ database/
│
└ README.md
```