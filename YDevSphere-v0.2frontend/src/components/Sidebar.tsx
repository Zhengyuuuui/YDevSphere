import { LayoutDashboard, FolderOpen, Clock, Settings, Folder } from 'lucide-react'
import type { Page } from '../types'

interface SidebarProps {
  activePage: Page
  onNavigate: (page: Page) => void
  workspace: string
}

function NavItem({
  icon: Icon,
  label,
  active,
  onClick,
}: {
  icon: React.ComponentType<{ size: number; className?: string }>
  label: string
  active: boolean
  onClick: () => void
}) {
  return (
    <button
      onClick={onClick}
      className={`w-full flex items-center gap-2.5 px-3 py-[7px] rounded-[6px] text-[13px] text-left transition-colors duration-75 ${
        active
          ? 'bg-[#EEF2FF] text-[#2563EB] font-medium'
          : 'text-[#6B7280] hover:bg-[#F3F4F6] hover:text-[#374151]'
      }`}
    >
      <Icon
        size={14}
        className={active ? 'text-[#2563EB]' : 'text-[#9CA3AF]'}
      />
      {label}
    </button>
  )
}

function SectionLabel({ label }: { label: string }) {
  return (
    <div className="px-3 pb-1 pt-3">
      <span className="text-[10px] font-semibold uppercase tracking-[0.1em] text-[#C4C9D0]">
        {label}
      </span>
    </div>
  )
}

export default function Sidebar({ activePage, onNavigate, workspace }: SidebarProps) {
  return (
    <div className="w-[220px] flex-shrink-0 h-full bg-white border-r border-[#E5E7EB] flex flex-col select-none">
      {/* Logo */}
      <div className="px-4 pt-[18px] pb-3 flex items-center gap-2.5">
        <div className="w-5 h-5 bg-[#17191C] rounded-[5px] flex items-center justify-center flex-shrink-0">
          <span className="text-white text-[10px] font-bold leading-none tracking-tight">Y</span>
        </div>
        <span className="text-[13px] font-semibold text-[#17191C] tracking-tight">YDevSphere</span>
      </div>

      <div className="mx-3 h-px bg-[#F0F1F3]" />

      {/* Navigation */}
      <nav className="flex-1 px-2 pt-1 overflow-y-auto">
        <SectionLabel label="Main" />

        <div className="space-y-0.5">
          <NavItem
            icon={LayoutDashboard}
            label="Overview"
            active={activePage === 'overview'}
            onClick={() => onNavigate('overview')}
          />
          <NavItem
            icon={FolderOpen}
            label="Projects"
            active={activePage === 'projects'}
            onClick={() => onNavigate('projects')}
          />
          <NavItem
            icon={Clock}
            label="Recent"
            active={activePage === 'recent'}
            onClick={() => onNavigate('recent')}
          />
        </div>

        <SectionLabel label="Workspace" />

        <button
          onClick={() => onNavigate('projects')}
          className="w-full flex items-center gap-2.5 px-3 py-[7px] rounded-[6px] text-[13px] text-left transition-colors duration-75 text-[#6B7280] hover:bg-[#F3F4F6] hover:text-[#374151]"
        >
          <Folder size={14} className="text-[#9CA3AF] flex-shrink-0" />
          <span className="flex-1 truncate">{workspace}</span>
          <span className="text-[11px] text-[#C4C9D0] flex-shrink-0">127</span>
        </button>
      </nav>

      {/* Settings */}
      <div className="px-2 pb-3 pt-2 border-t border-[#F0F1F3]">
        <NavItem
          icon={Settings}
          label="Settings"
          active={activePage === 'settings'}
          onClick={() => onNavigate('settings')}
        />
      </div>
    </div>
  )
}
