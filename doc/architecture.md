**保证 YDevSphere 在长期开发中不会架构腐化。**

---

# ARCHITECTURE.md

```md
# YDevSphere Architecture Document

Version:
0.1.0


Project:
YDevSphere


Description:

YDevSphere is a local-first AI developer workspace intelligence application.

It scans, indexes and understands local developer projects.

---

# 1. Architecture Overview


YDevSphere uses a desktop hybrid architecture.


```

```
                    YDevSphere Desktop


                          |

                     Tauri 2 Runtime


                          |


        ------------------------------------


        |                                  |


 Vue3 Frontend                       Rust Backend


        |                                  |


 UI / State / UX                    System Intelligence


                                           |


                 --------------------------------------


                 |             |            |


              Scanner       Git       Project Parser


                                           |


                                           |


                                      SQLite


                                           |


                                   Local Project Memory


                                           |


                                      AI Layer

```

---

# 2. Technology Stack

## Desktop Framework

Tauri 2

Responsibilities:

* Desktop window
* Application lifecycle
* Native integration
* Permissions
* Packaging

---

## Frontend

Framework:

Vue 3

Language:

TypeScript

Build:

Vite

UI:

TailwindCSS

State:

Pinia

Routing:

Vue Router

Responsibilities:

* User interface
* Project dashboard
* Settings
* Visualization

Frontend MUST NOT:

* Access filesystem directly
* Execute shell commands
* Manage database

---

## Backend Core

Language:

Rust

Responsibilities:

* Filesystem scanning
* Git analysis
* Project parsing
* Database operations
* Native system interaction

---

## Database

SQLite

Purpose:

Local project index database.

NOT:

Cloud database replacement.

---

# 3. Directory Structure

```

ydevsphere/


├── src/

│

│   ├── main.ts

│   ├── App.vue

│   │

│   ├── pages/

│   │

│   ├── components/

│   │

│   ├── stores/

│   │

│   ├── api/

│   │

│   └── types/


│


├── src-tauri/


│

│   ├── src/


│   │

│   │── main.rs

│   │

│   │── commands/

│   │

│   │── scanner/

│   │

│   │── parser/

│   │

│   │── git/

│   │

│   │── database/

│   │

│   │── models/


│


├── database/


│

│── migrations/


│


├── docs/


│

│── PRD.md

│── ARCHITECTURE.md

│── RESTRICTIONS.md


│


└── README.md


```

---

# 4. Rust Backend Modules

## commands

Tauri IPC入口。

Example:

```
commands/

project.rs

scanner.rs

settings.rs

```

Only commands expose functions to frontend.

---

## scanner

Filesystem scanning.

Responsibilities:

* directory traversal
* project detection
* file indexing

Example:

Input:

```
/Users/user/Projects
```

Output:

```
Project[]

```

---

## parser

Project technology detection.

Supported:

Frontend:

```
Vue
React
Angular
Svelte
```

Backend:

```
Node
Go
Python
Rust
Java
```

Detection sources:

```
package.json

go.mod

Cargo.toml

requirements.txt

```

---

## git

Git information.

Using:

```
git2-rs
```

Provides:

```
branch

commit

status

last_update

```

---

## database

SQLite layer.

Recommended:

```
sqlx

```

Responsibilities:

* migrations
* CRUD
* transactions

---

# 5. Frontend Architecture

## Pages

```

pages/


Dashboard.vue


ProjectDetail.vue


Settings.vue


Welcome.vue


```

---

## Components

```

components/


ProjectCard.vue


ProjectList.vue


ScanButton.vue


TechnologyBadge.vue


```

---

## Stores

Pinia:

```

stores/


project.ts


settings.ts


scanner.ts


```

---

# 6. Frontend-Rust Communication

Communication:

Tauri IPC

Flow:

```

Vue


 |

invoke()


 |

Rust Command


 |

Service Layer


 |

SQLite / OS


```

Example:

Frontend:

```ts
invoke(
"scan_projects",
{
 path:"~/Projects"
}
)

```

Rust:

```rust
#[tauri::command]

fn scan_projects(path:String){

}

```

---

# 7. Database Design

Database location:

```

~/.ydevsphere/database.sqlite

```

---

## projects table

```

id

name

path

language

framework

created_at

updated_at

```

---

## files table

```

id

project_id

path

hash

size

modified_time

```

---

## git_info

```

id

project_id

branch

commit

status

updated_at

```

---

## ai_analysis

```

id

project_id

summary

architecture

suggestions

updated_at

```

---

# 8. Local Project Memory

Each project can contain:

```

project/


.ydevsphere/


project.json

memory.md

analysis.json


```

---

project.json:

```json

{

"name":"example",

"stack":[

"Vue3",

"TypeScript"

],

"packageManager":"pnpm"

}

```

---

# 9. Security Architecture

Principle:

Local First.

Rules:

Frontend:

NO filesystem permission.

Rust:

controlled access.

User:

must select directory.

---

Write operation:

Requires:

explicit user permission.

Allowed:

```
.ydevsphere/

```

Forbidden:

```
modify source code

delete files

rewrite config

```

---

# 10. AI Architecture

AI is a separate layer.

Architecture:

```

Project Context


        |


Context Builder


        |


AI Provider


        |


Analysis Result


```

Supported providers:

Future:

* OpenAI
* Claude
* Gemini
* Local LLM

---

# 11. Performance Requirements

Scanner:

Must run asynchronously.

UI MUST NOT freeze.

Large projects:

100k+ files supported.

---

Database:

Use:

batch insert.

Avoid:

one file one transaction.

---

# 12. Cross Platform

Targets:

Priority:

1. macOS

2. Windows

3. Linux

Forbidden:

macOS-only business logic.

---

# 13. Packaging

macOS:

```
.dmg

.app

```

Windows:

```
.exe

.msi

```

Linux:

```
.AppImage

.deb

```

---

# 14. Development Principles

## Principle 1

Local First

User data belongs to user.

---

## Principle 2

Modular Rust Core

Each capability isolated.

---

## Principle 3

Frontend only handles presentation.

---

## Principle 4

AI is enhancement, not foundation.

---

# 15. Future Extension

Possible modules:

```

cloud-sync

mcp-server

agent

team

marketplace


```

Architecture must allow extension.

```

---

```

---

这个 ARCHITECTURE.md 定下来之后，下一步建议不要马上写业务代码。

正确顺序：

## Step 1

初始化 Tauri 工程：

```bash
pnpm create tauri-app
```

选择：

```
Vue
TypeScript
Rust
```

---

## Step 2

建立 Rust workspace：

```
src-tauri/src

├ commands
├ scanner
├ parser
├ database
├ models
```

---

## Step 3

先完成最小闭环：

```
选择目录

↓

Rust扫描

↓

SQLite保存

↓

Vue展示项目列表

```

这个闭环完成，YDevSphere 就有第一个真正可运行版本。

---

另外一个建议：

因为这个项目未来可能会拆分 CLI 和 MCP，我建议 Rust 部分从第一天就不要写成普通 Tauri command 堆积，而采用：

```
src-tauri

core/

scanner/

parser/

database/

commands/

```

让未来：

```
YDevSphere Desktop

YDevSphere CLI

YDevSphere MCP Server
```

共享同一个 core。这个会极大降低后期重构成本。
