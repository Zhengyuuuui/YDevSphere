DEVELOPMENT_SPEC.md
# YDevSphere Development Specification


Version:
0.1.0


Status:
Draft




---


# 1. Development Goal




## Objective


Build the first usable version of YDevSphere.


The MVP should allow developers to:


1. Select a local workspace directory
2. Scan projects inside the directory
3. Detect project technology stack
4. Store project metadata locally
5. Display projects in desktop UI




---


# 2. MVP Scope




## Included




### Project Scanner


- Directory selection
- Recursive scanning
- Project detection
- Metadata extraction




### Project Database


- SQLite initialization
- Project CRUD
- Scan history




### Desktop UI


- Welcome page
- Dashboard
- Project detail




### Local Memory


Generate:


.ydevsphere/project.json






---


## Excluded




Not implemented:




- AI analysis
- Cloud sync
- User account
- Team collaboration
- Code editing
- Online service






---


# 3. Development Environment




## Frontend




Node:


>=20



ydevsphere/

src/

src-tauri/

docs/

database/





---


# 5. Frontend Specification




## Pages




## Welcome Page




Path:


/




Purpose:


First launch experience.






Components:




- Logo
- Introduction
- Select Folder Button






Action:




Click:


"Choose Workspace"






Invoke Rust:



select_workspace()







---


# Dashboard Page




Path:


/dashboard






Purpose:


Display projects.






Features:




- Project list
- Search
- Sort
- Refresh scan






Data source:




SQLite through Rust API.






---


# Project Detail Page




Path:




/project/:id






Display:




- Project name
- Path
- Stack
- Last scan time






---


# 6. Backend Specification




# Scanner Module




Location:





src-tauri/src/scanner







Responsibilities:




Scan directories.




Input:





workspace_path





Output:





Vec<Project>







---


# Project Detection Rules




A folder is considered a project when:




## Node




Contains:





package.json







## Rust




Contains:





Cargo.toml







## Go




Contains:





go.mod







## Python




Contains:





requirements.txt

pyproject.toml







---


# Ignore Rules




Scanner MUST ignore:





node_modules

.git

target

dist

build

vendor

.cache







---


# 7. Database Specification






Location:





~/.ydevsphere/database.sqlite







---


# Tables




## projects




```sql


CREATE TABLE projects (


id INTEGER PRIMARY KEY,


name TEXT,


path TEXT UNIQUE,


language TEXT,


framework TEXT,


created_at DATETIME,


updated_at DATETIME


);




scan_history


CREATE TABLE scan_history (


id INTEGER PRIMARY KEY,


workspace TEXT,


scan_time DATETIME,


status TEXT


);




8. Tauri Commands

Frontend available commands:

select_workspace

Purpose:

Open native folder picker.

Input:

None

Output:

String path


scan_projects

Input:

workspace_path



Output:

Project[]


get_projects

Input:

None

Output:

Project[]


get_project_detail

Input:

project_id



Output:

ProjectDetail


9. Error Handling

All errors must return:



{


"success":false,


"message":"error description"


}



Examples:

Permission denied
Directory unavailable
Database failure
10. Security Requirements

Scanner:

Read only.

No source modification.

No command execution.

No network request.

Allowed:

Create:

.ydevsphere/



Only after user confirmation.

11. Performance Requirements

Small project:

< 1000 files

Response:

instant

Large project:

100k files

Requirement:

UI remains responsive.

Scanner:

must run asynchronously.

12. Development Order
Phase 1

Project bootstrap

完成:

Tauri initialization
Vue setup
Rust modules
Phase 2

Backend foundation

完成:

SQLite
Scanner
Project parser
Phase 3

Frontend

完成:

Dashboard
Project cards
Detail page
Phase 4

Integration

完成:

IPC communication
Complete workflow
13. Acceptance Criteria

Version 0.1 is complete when:

User can:

Install YDevSphere

Open application

Choose:

~/Projects

Automatically detect:

Project A


Vue3


TypeScript




Project B


Go


SQLite



Close and reopen application

Projects remain available