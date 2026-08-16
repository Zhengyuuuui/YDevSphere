import { useState } from 'react'
import Sidebar from './components/Sidebar'
import OverviewPage from './pages/OverviewPage'
import ProjectsPage from './pages/ProjectsPage'
import RecentPage from './pages/RecentPage'
import SettingsPage from './pages/SettingsPage'
import { PROJECTS, WORKSPACES } from './data'
import type { Page, Workspace } from './types'

export default function App() {
  const [activePage, setActivePage] = useState<Page>('overview')
  const [activeWorkspace, setActiveWorkspace] = useState<Workspace>(WORKSPACES[0])

  return (
    <div className="flex h-screen font-sans overflow-hidden bg-[#F7F8FA]">
      <Sidebar
        activePage={activePage}
        onNavigate={setActivePage}
        workspace={activeWorkspace.name}
      />
      <main className="flex-1 overflow-y-auto">
        {activePage === 'overview' && (
          <OverviewPage
            projects={PROJECTS}
            onNavigate={(page) => setActivePage(page)}
          />
        )}
        {activePage === 'projects' && (
          <ProjectsPage
            projects={PROJECTS}
            workspace={activeWorkspace}
            workspaces={WORKSPACES}
            onWorkspaceChange={setActiveWorkspace}
          />
        )}
        {activePage === 'recent' && <RecentPage projects={PROJECTS} />}
        {activePage === 'settings' && <SettingsPage />}
      </main>
    </div>
  )
}
