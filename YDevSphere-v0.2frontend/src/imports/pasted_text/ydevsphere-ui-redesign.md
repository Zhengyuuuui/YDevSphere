Design a polished desktop application UI for a product called “YDevSphere”.

YDevSphere is a local-first developer workspace intelligence application.

It is NOT:
- an IDE
- a code editor
- a Git client
- a cloud project management SaaS
- an AI chatbot

It is a local intelligence layer that helps developers understand, organize, scan and manage the software projects on their computer.

The product should feel like a serious developer desktop utility that can be used every day.

Primary platforms:
- macOS
- Windows
- Linux

Technology:
- Tauri 2
- Vue 3
- Rust
- SQLite

The current implementation is functional but visually weak. Redesign the visual system while preserving the existing product logic.

==================================================
CORE DESIGN DIRECTION
==================================================

The visual concept is:

“Professional local developer infrastructure tool.”

Keywords:
- minimal
- precise
- calm
- technical
- trustworthy
- efficient
- elegant
- information-dense
- desktop-native

Visual references:
- Linear
- Raycast
- Apple system applications
- GitHub Desktop
- VS Code
- Vercel

Do NOT copy any reference product literally.

The result should feel like its own product.

Avoid:
- excessive gradients
- neon colors
- cyberpunk aesthetics
- glassmorphism
- 3D effects
- futuristic HUD interfaces
- excessive shadows
- excessive rounded cards
- decorative illustrations
- startup landing page aesthetics
- consumer AI aesthetics

Do NOT use AI clichés:
- robot icons
- brain icons
- magic wand
- AI sparkle effects

Do NOT overuse developer clichés:
- terminal graphics
- code brackets
- matrix effects
- circuit board graphics

The design must communicate trust, ownership and local-first privacy through restraint and clarity rather than decorative graphics.

==================================================
IMPORTANT PRODUCT PRINCIPLE
==================================================

YDevSphere should feel like:

“An intelligent command center for my personal software universe.”

But it should NOT look like a futuristic command center.

It should look like a refined professional desktop application.

Prioritize:
1. information hierarchy
2. navigation clarity
3. fast scanning
4. project discovery
5. low visual noise
6. desktop-native interaction

==================================================
DESKTOP WINDOW
==================================================

Create the design as a real desktop application rather than a responsive SaaS website.

Primary design frame:

1440 × 900 px

Also consider behavior at:
- 1280 × 800
- 1728 × 1117

The application should have a native desktop feeling.

On macOS:
- preserve the native macOS menu bar
- preserve the standard traffic-light window controls
- do not create a fake browser-like top navigation bar
- do not create a marketing-style header

The application content starts below the native desktop chrome.

==================================================
APP SHELL
==================================================

Create a persistent left sidebar.

Sidebar width:
approximately 216–224 px.

Sidebar should be visually quiet and compact.

Structure:

YDevSphere
────────────────

PRIMARY

Projects
Recent

WORKSPACE

Desktop
or the currently selected workspace

────────────────

Settings

The sidebar should NOT contain too many navigation items.

Do not add:
- Analytics
- AI
- Favorites
- Collections
- MCP
- Dashboard
- Integrations

These are not part of the MVP navigation.

Use simple monochrome line icons.

Suggested icons:
- Projects: folder/grid/list icon
- Recent: clock icon
- Workspace: folder icon
- Settings: gear icon

Icons should be subtle and small.

Do not use colorful icons in the navigation.

The YDevSphere logo should be compact and professional.

Logo treatment:
- black/dark logo mark
- “YDevSphere” wordmark
- restrained blue accent if appropriate

Do not make the logo oversized.

==================================================
SIDEBAR STATES
==================================================

Create:
1. default sidebar
2. Projects selected
3. Recent selected
4. Workspace selected
5. Settings selected
6. sidebar item hover state

Selected item should use a subtle neutral/blue background.

Avoid strong gradients.

Example selected treatment:

light neutral blue-gray surface
with dark text
and a small blue accent or icon

The selected state should be visible but understated.

==================================================
MAIN CONTENT — PROJECTS
==================================================

The main page is the Projects page.

This is the primary MVP experience.

Use a large but restrained content area.

Recommended content max-width:
approximately 1100–1200 px.

The content should not feel stretched across the entire 1440px window.

Page structure:

Projects

Desktop · 127 projects                         Scan

Search projects...        All    Git    Recent    Sort

ALL PROJECTS

Project table

==================================================
PAGE HEADER
==================================================

Top section:

Projects

Under the title:

Desktop · 127 projects

Use:
- title: strong, approximately 24px
- metadata: 13–14px
- muted gray secondary text

On the right:

Scan button

The Scan button should be compact.

Use a subtle refresh/scan icon.

Button should be:
- white or very light surface
- thin border
- small radius
- subtle hover state

Do NOT make Scan look like a primary marketing CTA.

When scanning is active, design a scanning state:

Scanning…
with a subtle progress indicator or status text.

==================================================
WORKSPACE SELECTOR
==================================================

The current workspace should be represented clearly.

Example:

Desktop · 127 projects

Add a small dropdown affordance:

Desktop ▾

Possible menu:

All Workspaces
Desktop
Documents
Custom Workspace

The workspace selector should NOT look like a large pill dashboard filter.

It should look like a desktop application control.

==================================================
SEARCH + FILTER TOOLBAR
==================================================

Create a compact toolbar below the page header.

Left:

Search projects...

Search field:
- approximately 320–380px wide
- search icon
- subtle gray border
- 8px radius
- 36–40px height

Placeholder:
Search projects...

Search should support:
- project name
- project path

Right side:

All
Git
Recent
Sort

These are controls, not large colorful pills.

Selected filter should have a subtle filled background.

Sort should open a menu:

Recently updated
Name A–Z
Name Z–A
Recently scanned

Do not create excessive filter UI.

==================================================
PROJECT TABLE
==================================================

This is the most important component.

Use a refined developer-oriented table/list.

DO NOT use a traditional enterprise admin dashboard table.

DO NOT use spreadsheet styling.

DO NOT put every project inside a separate card.

Instead create one continuous table with subtle row separation.

Columns:

PROJECT
TECHNOLOGY
GIT
UPDATED

Optional right-side action column.

Example:

PROJECT                  TECHNOLOGY        GIT          UPDATED
──────────────────────────────────────────────────────────────
◇ client                 Node · Vue        ● Clean      12m
  ~/Desktop/client

◇ ydevsphere             Rust · Vue        ● Clean      1h
  ~/Developer/ydevsphere

◇ page-flip              Node              3 changed    2d
  ~/Desktop/page-flip

◇ figma-make-app         React             ● Clean      3d
  ~/Documents/...

==================================================
PROJECT ROW DESIGN
==================================================

Project name is the primary visual element.

Example:

client

Under it:

~/Desktop/client

Project name:
- 14–15px
- medium or semibold
- dark text

Path:
- 12–13px
- muted gray
- truncate long paths

Each row should have comfortable vertical padding.

Approximate row height:
64–72px.

Do not make rows too dense.

Do not make rows too tall.

Use very subtle separators.

Example:

1px #EAECEF divider

Avoid heavy table borders.

==================================================
PROJECT ICON
==================================================

Every project should have a small neutral project icon.

Possible icon:
- simple outlined folder
- simple project/application symbol

Do not use random colorful project logos.

The icon should be:
- 16–18px
- monochrome
- low contrast

The project icon is primarily for visual scanning.

==================================================
TECHNOLOGY COLUMN
==================================================

Technology should use compact badges.

Examples:

Node.js
Vue
React
Rust
TypeScript
Python
Next.js

Use low-saturation technology colors.

Examples:
- Node: very subtle green
- Vue: subtle green
- Rust: subtle warm orange
- React: subtle blue
- TypeScript: subtle blue

Do not make badges saturated.

Do not turn the table into a rainbow.

When multiple technologies exist:

Node.js   Vue   TypeScript

Keep badges compact.

==================================================
GIT COLUMN
==================================================

Git status should be extremely easy to scan.

Clean:

● Clean

Dirty:

● 2 changed

Not a Git repository:

— Not a Git repository

Detached HEAD:

● Detached

Use small status indicators.

Colors should be restrained:
- clean: muted green
- changed: muted orange
- error: muted red
- unavailable: neutral gray

Do not use large status cards.

==================================================
UPDATED COLUMN
==================================================

Show relative time:

12m
1h
2d
3d

On hover or detail view, the full timestamp can be available.

Right align this column.

Use muted text.

==================================================
ROW INTERACTION
==================================================

Rows should be clickable.

Single click:
- opens Project Detail

Double click:
- opens the project in the default editor

Hover:
- subtle background highlight

Selected row:
- very subtle blue-gray background
- thin blue accent if appropriate

Do NOT use a large glowing selection effect.

==================================================
ROW ACTIONS
==================================================

Do NOT put a large “Open in Cursor” button inside every row.

This is important.

Default row:

client                       ● Clean        12m       ›

On hover:

client                       ● Clean        12m     ↗  ⋯

Use a compact overflow menu.

Menu:

Open in Cursor
Open in VS Code
Open in File Manager
Open in Terminal
Copy Path
Rescan

The action UI should only become visible when needed.

This dramatically reduces visual noise.

==================================================
RECENT PROJECTS
==================================================

Do NOT put a large “Recent Projects” card section above the table.

Recent projects already have their own navigation item in the sidebar.

The Projects page should primarily focus on the project index.

This is an important information architecture decision.

==================================================
EMPTY STATES
==================================================

Create three empty states.

1. No workspace

Title:

No workspace selected

Description:

Choose a local development directory to start indexing your projects.

Primary action:

Choose Workspace

Secondary action:

Import Desktop
Import Documents

Keep this very minimal.

No illustrations.

No robot graphics.

No decorative AI graphics.

2. Workspace has no projects

Title:

No projects found

Description:

YDevSphere could not find any recognizable projects in this workspace.

Actions:

Scan Again
Choose Another Workspace

3. Search has no results

Title:

No matching projects

Description:

Try a different project name or path.

Action:

Clear Search

These empty states should use typography and a simple icon only.

==================================================
SCAN STATUS
==================================================

Create a subtle scan status component.

When scanning:

Scanning Desktop…

Indexing 42 of 127 projects

[progress bar]

When completed:

Indexed 127 projects
Completed just now

When partially failed:

Indexed 124 projects
3 projects could not be scanned

View details

Do not use a giant modal for normal scan progress.

Use a compact status area near the toolbar or below the header.

==================================================
RECENT PAGE
==================================================

Create a second screen for the “Recent” sidebar item.

Title:

Recent

Subtitle:

Projects you opened recently

Use the same Project Table component as the Projects page.

Do NOT create a completely different visual language.

Recent table:

PROJECT
TECHNOLOGY
GIT
LAST OPENED

This page should be extremely simple.

==================================================
WORKSPACE SWITCHER
==================================================

Create a dropdown interaction for the workspace selector.

Example:

Desktop ▾

Dropdown:

All Workspaces

Desktop
~/Desktop

Documents
~/Documents

Custom Workspace
~/Developer

────────────

Manage Workspaces

The dropdown should feel like a native desktop menu.

No giant modal.

==================================================
SETTINGS ENTRY
==================================================

Settings is accessible from the sidebar.

Do not design the Settings page in detail yet.

Only create the navigation entry and a placeholder transition.

The visual system should be designed so Settings can later use:

Settings
├── General
├── Workspace
├── Editor
├── Privacy
├── Database
└── About

==================================================
DESIGN SYSTEM
==================================================

Create a small reusable design system in Figma.

Color philosophy:

Primary:
- restrained developer blue

Suggested blue:
#2563EB

Background:
- very light neutral gray
- approximately #F7F8FA

Surface:
- #FFFFFF

Primary text:
- near-black
- approximately #17191C

Secondary text:
- #6B7280

Muted text:
- #9CA3AF

Border:
- #E5E7EB

Hover:
- #F3F4F6

Selected:
- subtle blue-tinted neutral

Do not overuse blue.

Blue should primarily communicate:
- selected state
- primary action
- focus
- important interaction

The UI should still look good in grayscale.

==================================================
TYPOGRAPHY
==================================================

Use a clean system-style sans serif.

Preferred:
- SF Pro Display / SF Pro Text on macOS
- Inter as cross-platform fallback

Typography hierarchy:

Page title:
24px / semibold

Section title:
13–14px / semibold / uppercase or small label when appropriate

Project name:
14–15px / medium or semibold

Body:
13–14px

Metadata:
12–13px

Do not use huge typography.

Do not use marketing typography.

Use typography to create hierarchy instead of cards and shadows.

==================================================
SPACING
==================================================

Use an 8px spacing system.

Common values:
4
8
12
16
20
24
32
40
48

Main content horizontal padding:
32px

Sidebar:
216–224px

Table row:
64–72px

Toolbar height:
36–40px

==================================================
BORDER RADIUS
==================================================

Use restrained radii.

Inputs:
7–8px

Buttons:
7–8px

Dropdown:
8px

Table:
0px or very subtle rounding

Do not put every section inside a rounded card.

The overall application should feel structured rather than “card-based”.

==================================================
SHADOWS
==================================================

Avoid large shadows.

Use:
- no shadow for normal content
- extremely subtle shadow only for floating menus/popovers

The main UI should rely on:
- spacing
- borders
- typography
- surface contrast

==================================================
ICONOGRAPHY
==================================================

Use a consistent line icon system.

Preferred visual language:
- 16–18px
- 1.5px stroke
- monochrome
- simple geometry

Possible icon library style:
Lucide-like.

Do not use emoji.

Do not use colorful illustrations.

==================================================
DARK MODE PREPARATION
==================================================

The first design should be light mode.

However, create color tokens that can later support dark mode.

Do not make the layout dependent on pure white backgrounds.

Use semantic tokens:
- background
- surface
- border
- text
- muted text
- accent
- success
- warning
- danger

==================================================
FIGMA COMPONENTS
==================================================

Create reusable components for:

1. App Shell
2. Sidebar
3. Sidebar Item
4. Workspace Selector
5. Page Header
6. Search Input
7. Filter Control
8. Sort Menu
9. Project Table
10. Project Row
11. Technology Badge
12. Git Status
13. Scan Button
14. Scan Status
15. Overflow Menu
16. Empty State
17. Toast
18. Dropdown

Create component variants for:
- default
- hover
- selected
- disabled
- loading
- empty
- error

==================================================
SCREENS TO GENERATE
==================================================

Generate the following high-fidelity desktop screens:

SCREEN 1
Projects — normal state

Workspace:
Desktop

127 projects

Include approximately 10 realistic projects.

Example names:
- ydevsphere
- client
- page-flip
- vibe-diary-server
- canglan-flash-backend
- canglan-flash-frontend
- campus-wall-backend
- campus-wall-uniapp
- banking-backend
- figma-make-app

Use realistic technologies.

SCREEN 2
Projects — hover interaction

Show one project row hovered.

Reveal:
Open icon
Overflow menu

SCREEN 3
Projects — search results

Search:
client

Show matching projects.

SCREEN 4
Projects — empty search

Show:
No matching projects

SCREEN 5
Projects — scanning

Show:
Scanning Desktop…

Indexed 42 of 127 projects

SCREEN 6
Recent

Use the same table design.

SCREEN 7
Workspace dropdown

Show the workspace selector opened.

SCREEN 8
Project overflow menu

Show:
Open in Cursor
Open in VS Code
Open in File Manager
Open in Terminal
Copy Path
Rescan

==================================================
FINAL VISUAL QUALITY
==================================================

The final design must NOT look like:
- a generic SaaS dashboard
- an admin panel
- a startup landing page
- an AI assistant
- a futuristic developer tool
- a card-heavy project management product

It should look like:

A polished native desktop utility for developers.

Think:
Finder + Linear + GitHub Desktop + Raycast

but with an original visual identity.

The interface should feel:
quiet,
fast,
precise,
trustworthy,
and built for daily use.

Every visible element must have a functional reason to exist.

Prioritize usability over visual decoration.