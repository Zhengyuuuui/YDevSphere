import { useState, useRef, useEffect } from 'react'
import { ExternalLink, MoreHorizontal, Copy, RefreshCw, Terminal, Folder } from 'lucide-react'
import type { Project } from '../types'
import TechBadge from './TechBadge'
import GitStatusBadge from './GitStatusBadge'

const GRID = '1fr 200px 148px 68px 76px'

interface TableConfig {
  showLastOpened?: boolean
}

interface OverflowMenuProps {
  onClose: () => void
}

function OverflowMenu({ onClose }: OverflowMenuProps) {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose()
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [onClose])

  const items: Array<{ icon: typeof Copy; label: string } | null> = [
    { icon: ExternalLink, label: 'Open in Cursor' },
    { icon: ExternalLink, label: 'Open in VS Code' },
    { icon: Folder, label: 'Open in File Manager' },
    { icon: Terminal, label: 'Open in Terminal' },
    null,
    { icon: Copy, label: 'Copy Path' },
    { icon: RefreshCw, label: 'Rescan' },
  ]

  return (
    <div
      ref={ref}
      className="absolute right-0 top-full mt-1 w-[196px] bg-white border border-[#E5E7EB] rounded-[8px] z-50 py-1"
      style={{ boxShadow: '0 4px 16px rgba(0,0,0,0.08), 0 1px 4px rgba(0,0,0,0.04)' }}
    >
      {items.map((item, i) =>
        item === null ? (
          <div key={`sep-${i}`} className="my-1 border-t border-[#F3F4F6]" />
        ) : (
          <button
            key={item.label}
            className="w-full flex items-center gap-2.5 px-3 py-1.5 text-[13px] text-[#374151] hover:bg-[#F9FAFB] text-left transition-colors duration-75"
            onClick={onClose}
          >
            <item.icon size={13} className="text-[#9CA3AF] flex-shrink-0" />
            {item.label}
          </button>
        )
      )}
    </div>
  )
}

function ProjectIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none" className="flex-shrink-0 text-[#C4C9D0]">
      <path
        d="M2 5C2 4.17 2.67 3.5 3.5 3.5H6L7.5 5H12.5C13.33 5 14 5.67 14 6.5V11.5C14 12.33 13.33 13 12.5 13H3.5C2.67 13 2 12.33 2 11.5V5Z"
        stroke="currentColor"
        strokeWidth="1.2"
        strokeLinejoin="round"
      />
    </svg>
  )
}

export default function ProjectTable({
  projects,
  config = {},
}: {
  projects: Project[]
  config?: TableConfig
}) {
  const [hoveredId, setHoveredId] = useState<string | null>(null)
  const [openMenuId, setOpenMenuId] = useState<string | null>(null)
  const [selectedId, setSelectedId] = useState<string | null>(null)

  const timeLabel = config.showLastOpened ? 'LAST OPENED' : 'UPDATED'

  if (projects.length === 0) return null

  return (
    <div className="w-full">
      {/* Column headers */}
      <div
        className="grid items-center px-4 py-2.5 border-b border-[#E5E7EB]"
        style={{ gridTemplateColumns: GRID }}
      >
        {['PROJECT', 'TECHNOLOGY', 'GIT', timeLabel, ''].map((col) => (
          <span
            key={col}
            className={`text-[10px] font-semibold uppercase tracking-[0.09em] text-[#B0B7C3] ${col === timeLabel ? 'text-right' : ''}`}
          >
            {col}
          </span>
        ))}
      </div>

      {/* Rows */}
      {projects.map((project, index) => {
        const isHovered = hoveredId === project.id
        const isSelected = selectedId === project.id
        const menuOpen = openMenuId === project.id
        const showActions = isHovered || menuOpen

        const rowBg = isSelected
          ? '#F0F4FF'
          : isHovered || menuOpen
          ? '#FAFAFA'
          : '#FFFFFF'

        const isFirst = index === 0
        const isLast = index === projects.length - 1

        return (
          <div
            key={project.id}
            className={`relative grid items-center px-4 border-b cursor-pointer transition-colors duration-75 ${
              isLast ? 'border-transparent' : 'border-[#EAECEF]'
            }`}
            style={{
              gridTemplateColumns: GRID,
              minHeight: '68px',
              backgroundColor: rowBg,
              borderRadius: isFirst && isLast ? '7px' : isFirst ? '7px 7px 0 0' : isLast ? '0 0 7px 7px' : undefined,
            }}
            onMouseEnter={() => setHoveredId(project.id)}
            onMouseLeave={() => setHoveredId(null)}
            onClick={() => setSelectedId(isSelected ? null : project.id)}
          >
            {/* Project name + path */}
            <div className="flex items-center gap-3 min-w-0 pr-4">
              <ProjectIcon />
              <div className="min-w-0">
                <div
                  className={`text-[14px] font-medium truncate leading-tight ${
                    isSelected ? 'text-[#1D4ED8]' : 'text-[#17191C]'
                  }`}
                >
                  {project.name}
                </div>
                <div className="text-[12px] text-[#9CA3AF] truncate mt-[3px]">{project.path}</div>
              </div>
            </div>

            {/* Technologies */}
            <div className="flex flex-wrap gap-1 pr-4">
              {project.technologies.map((tech) => (
                <TechBadge key={tech.name} tech={tech} />
              ))}
            </div>

            {/* Git status */}
            <div className="pr-4">
              <GitStatusBadge status={project.git} />
            </div>

            {/* Time */}
            <div className="text-right">
              <span className="text-[12px] text-[#9CA3AF]">
                {config.showLastOpened ? project.lastOpenedAt : project.updatedAt}
              </span>
            </div>

            {/* Row actions */}
            <div className="flex items-center justify-end gap-1 relative">
              {showActions && (
                <>
                  <button
                    className="p-1.5 rounded-[5px] hover:bg-[#EAECEF] text-[#9CA3AF] hover:text-[#6B7280] transition-colors"
                    title="Open project"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <ExternalLink size={13} />
                  </button>
                  <div className="relative">
                    <button
                      className={`p-1.5 rounded-[5px] hover:bg-[#EAECEF] text-[#9CA3AF] hover:text-[#6B7280] transition-colors ${menuOpen ? 'bg-[#EAECEF] text-[#6B7280]' : ''}`}
                      title="More actions"
                      onClick={(e) => {
                        e.stopPropagation()
                        setOpenMenuId(menuOpen ? null : project.id)
                      }}
                    >
                      <MoreHorizontal size={13} />
                    </button>
                    {menuOpen && (
                      <OverflowMenu onClose={() => setOpenMenuId(null)} />
                    )}
                  </div>
                </>
              )}
            </div>
          </div>
        )
      })}
    </div>
  )
}
