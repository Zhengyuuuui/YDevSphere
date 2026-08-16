import { useState, useRef, useEffect } from 'react'
import { Search, RefreshCw, ChevronDown, Check, X, FolderOpen } from 'lucide-react'
import type { Project, Workspace } from '../types'
import ProjectTable from '../components/ProjectTable'

type FilterId = 'all' | 'git' | 'recent'
type SortId = 'recently-updated' | 'name-asc' | 'name-desc' | 'recently-scanned'
type ScanState = 'idle' | 'scanning' | 'completed'

const SORT_OPTIONS: { id: SortId; label: string }[] = [
  { id: 'recently-updated', label: 'Recently updated' },
  { id: 'name-asc', label: 'Name A–Z' },
  { id: 'name-desc', label: 'Name Z–A' },
  { id: 'recently-scanned', label: 'Recently scanned' },
]

const FILTERS: { id: FilterId; label: string }[] = [
  { id: 'all', label: 'All' },
  { id: 'git', label: 'Git' },
  { id: 'recent', label: 'Recent' },
]

function WorkspaceDropdown({
  workspace,
  workspaces,
  onSelect,
}: {
  workspace: Workspace
  workspaces: Workspace[]
  onSelect: (ws: Workspace) => void
}) {
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false)
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [])

  return (
    <div ref={ref} className="relative">
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1 text-[13px] text-[#6B7280] hover:text-[#374151] transition-colors"
      >
        {workspace.name}
        <ChevronDown size={12} className="text-[#B0B7C3]" />
      </button>

      {open && (
        <div
          className="absolute top-full left-0 mt-2 w-[220px] bg-white border border-[#E5E7EB] rounded-[8px] py-1 z-50"
          style={{ boxShadow: '0 4px 16px rgba(0,0,0,0.08), 0 1px 4px rgba(0,0,0,0.04)' }}
        >
          <div className="px-3 py-1.5">
            <span className="text-[10px] font-semibold uppercase tracking-[0.09em] text-[#B0B7C3]">
              Workspaces
            </span>
          </div>

          {workspaces.map((ws) => (
            <button
              key={ws.id}
              onClick={() => {
                onSelect(ws)
                setOpen(false)
              }}
              className="w-full flex items-center justify-between px-3 py-2 hover:bg-[#F9FAFB] transition-colors text-left"
            >
              <div>
                <div className="text-[13px] text-[#17191C]">{ws.name}</div>
                <div className="text-[11px] text-[#9CA3AF] mt-0.5">{ws.path}</div>
              </div>
              {workspace.id === ws.id && (
                <Check size={13} className="text-[#2563EB] flex-shrink-0 ml-2" />
              )}
            </button>
          ))}

          <div className="my-1 border-t border-[#F3F4F6]" />

          <button className="w-full px-3 py-2 text-[13px] text-[#6B7280] hover:bg-[#F9FAFB] hover:text-[#374151] transition-colors text-left">
            Manage Workspaces
          </button>
        </div>
      )}
    </div>
  )
}

function SortDropdown({ value, onChange }: { value: SortId; onChange: (v: SortId) => void }) {
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false)
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [])

  return (
    <div ref={ref} className="relative">
      <button
        onClick={() => setOpen(!open)}
        className={`flex items-center gap-1.5 h-[32px] px-3 text-[13px] rounded-[6px] transition-colors ${
          open
            ? 'bg-white border border-[#E5E7EB] text-[#17191C] shadow-[0_1px_2px_rgba(0,0,0,0.04)]'
            : 'text-[#6B7280] hover:bg-[#F3F4F6] hover:text-[#374151]'
        }`}
      >
        Sort
        <ChevronDown size={12} className="text-[#B0B7C3]" />
      </button>

      {open && (
        <div
          className="absolute right-0 top-full mt-1 w-[180px] bg-white border border-[#E5E7EB] rounded-[8px] py-1 z-50"
          style={{ boxShadow: '0 4px 16px rgba(0,0,0,0.08), 0 1px 4px rgba(0,0,0,0.04)' }}
        >
          {SORT_OPTIONS.map((opt) => (
            <button
              key={opt.id}
              onClick={() => {
                onChange(opt.id)
                setOpen(false)
              }}
              className="w-full flex items-center justify-between px-3 py-1.5 text-[13px] hover:bg-[#F9FAFB] transition-colors text-left"
            >
              <span className={value === opt.id ? 'text-[#2563EB] font-medium' : 'text-[#374151]'}>
                {opt.label}
              </span>
              {value === opt.id && <Check size={12} className="text-[#2563EB]" />}
            </button>
          ))}
        </div>
      )}
    </div>
  )
}

function ScanStatusBar({
  state,
  progress,
  total,
}: {
  state: ScanState
  progress: number
  total: number
}) {
  if (state === 'idle') return null

  if (state === 'scanning') {
    return (
      <div className="flex items-center gap-3 px-4 py-2.5 bg-white border border-[#E5E7EB] rounded-[8px] text-[13px]">
        <span className="w-[7px] h-[7px] rounded-full bg-[#2563EB] animate-pulse flex-shrink-0" />
        <span className="text-[#17191C] font-medium">Scanning Desktop…</span>
        <span className="text-[#9CA3AF]">
          Indexing {progress} of {total} projects
        </span>
        <div className="flex-1 h-[3px] bg-[#F3F4F6] rounded-full overflow-hidden">
          <div
            className="h-full bg-[#2563EB] rounded-full transition-all duration-300"
            style={{ width: `${(progress / total) * 100}%` }}
          />
        </div>
      </div>
    )
  }

  return (
    <div className="flex items-center gap-3 px-4 py-2.5 bg-[#F0FDF4] border border-[#BBF7D0] rounded-[8px] text-[13px]">
      <span className="w-[7px] h-[7px] rounded-full bg-[#16A34A] flex-shrink-0" />
      <span className="text-[#15803D] font-medium">Indexed {total} projects</span>
      <span className="text-[#9CA3AF]">Completed just now</span>
    </div>
  )
}

function EmptyState({
  type,
  onClear,
}: {
  type: 'no-projects' | 'no-results'
  onClear?: () => void
}) {
  if (type === 'no-results') {
    return (
      <div className="flex flex-col items-center justify-center py-20">
        <p className="text-[15px] font-semibold text-[#17191C] mb-1.5">No matching projects</p>
        <p className="text-[13px] text-[#9CA3AF] mb-5">Try a different project name or path.</p>
        <button
          onClick={onClear}
          className="px-3 py-1.5 text-[13px] text-[#6B7280] border border-[#E5E7EB] rounded-[6px] hover:bg-[#F3F4F6] transition-colors"
        >
          Clear Search
        </button>
      </div>
    )
  }

  return (
    <div className="flex flex-col items-center justify-center py-20">
      <div className="w-10 h-10 rounded-[10px] border-[1.5px] border-dashed border-[#D1D5DB] flex items-center justify-center mb-4">
        <FolderOpen size={18} className="text-[#C4C9D0]" />
      </div>
      <p className="text-[15px] font-semibold text-[#17191C] mb-1.5">No projects found</p>
      <p className="text-[13px] text-[#9CA3AF] text-center max-w-[300px] mb-5">
        YDevSphere could not find any recognizable projects in this workspace.
      </p>
      <div className="flex gap-2">
        <button className="px-4 py-2 bg-[#17191C] text-white text-[13px] font-medium rounded-[7px] hover:bg-[#2D3038] transition-colors">
          Scan Again
        </button>
        <button className="px-4 py-2 text-[13px] text-[#6B7280] border border-[#E5E7EB] rounded-[7px] hover:bg-[#F3F4F6] transition-colors">
          Choose Another Workspace
        </button>
      </div>
    </div>
  )
}

interface ProjectsPageProps {
  projects: Project[]
  workspace: Workspace
  workspaces: Workspace[]
  onWorkspaceChange: (ws: Workspace) => void
}

export default function ProjectsPage({
  projects,
  workspace,
  workspaces,
  onWorkspaceChange,
}: ProjectsPageProps) {
  const [search, setSearch] = useState('')
  const [filter, setFilter] = useState<FilterId>('all')
  const [sort, setSort] = useState<SortId>('recently-updated')
  const [scanState, setScanState] = useState<ScanState>('idle')
  const [scanProgress, setScanProgress] = useState(0)
  const TOTAL = 127

  const handleScan = () => {
    if (scanState === 'scanning') return
    setScanState('scanning')
    setScanProgress(0)
    let progress = 0
    const interval = setInterval(() => {
      progress += Math.floor(Math.random() * 9) + 4
      if (progress >= TOTAL) {
        progress = TOTAL
        clearInterval(interval)
        setScanProgress(progress)
        setTimeout(() => setScanState('completed'), 400)
        setTimeout(() => setScanState('idle'), 4000)
      } else {
        setScanProgress(progress)
      }
    }, 180)
  }

  const filtered = projects
    .filter((p) => {
      if (!search) return true
      const q = search.toLowerCase()
      return p.name.toLowerCase().includes(q) || p.path.toLowerCase().includes(q)
    })
    .filter((p) => {
      if (filter === 'git') return p.git.type !== 'none'
      if (filter === 'recent') return p.updatedAt.endsWith('m') || p.updatedAt === '1h'
      return true
    })
    .sort((a, b) => {
      if (sort === 'name-asc') return a.name.localeCompare(b.name)
      if (sort === 'name-desc') return b.name.localeCompare(a.name)
      return 0
    })

  const sectionLabel = search
    ? `${filtered.length} result${filtered.length !== 1 ? 's' : ''}`
    : 'All Projects'

  return (
    <div className="min-h-full bg-[#F7F8FA]">
      <div className="max-w-[1140px] mx-auto px-8 py-7">
        {/* Page header */}
        <div className="flex items-start justify-between mb-5">
          <div>
            <h1 className="text-[22px] font-semibold text-[#17191C] tracking-tight leading-tight">
              Projects
            </h1>
            <div className="flex items-center gap-1.5 mt-1.5">
              <WorkspaceDropdown
                workspace={workspace}
                workspaces={workspaces}
                onSelect={onWorkspaceChange}
              />
              <span className="text-[#D1D5DB] text-[13px] leading-none">·</span>
              <span className="text-[13px] text-[#9CA3AF]">{TOTAL} projects</span>
            </div>
          </div>

          <button
            onClick={handleScan}
            disabled={scanState === 'scanning'}
            className="flex items-center gap-1.5 px-3 py-[7px] text-[13px] text-[#374151] border border-[#E5E7EB] rounded-[7px] bg-white hover:bg-[#F9FAFB] disabled:opacity-50 disabled:cursor-not-allowed transition-colors mt-1"
          >
            <RefreshCw
              size={13}
              className={`text-[#9CA3AF] ${scanState === 'scanning' ? 'animate-spin' : ''}`}
            />
            {scanState === 'scanning' ? 'Scanning…' : 'Scan'}
          </button>
        </div>

        {/* Scan status */}
        {scanState !== 'idle' && (
          <div className="mb-4">
            <ScanStatusBar state={scanState} progress={scanProgress} total={TOTAL} />
          </div>
        )}

        {/* Toolbar */}
        <div className="flex items-center justify-between mb-5">
          <div className="relative">
            <Search
              size={13}
              className="absolute left-3 top-1/2 -translate-y-1/2 text-[#B0B7C3] pointer-events-none"
            />
            <input
              type="text"
              placeholder="Search projects..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="w-[340px] h-[36px] pl-8 pr-8 text-[13px] border border-[#E5E7EB] rounded-[8px] bg-white placeholder:text-[#B0B7C3] text-[#17191C] focus:outline-none focus:border-[#2563EB] transition-colors"
              style={{ boxShadow: 'none' }}
            />
            {search && (
              <button
                onClick={() => setSearch('')}
                className="absolute right-2.5 top-1/2 -translate-y-1/2 text-[#B0B7C3] hover:text-[#6B7280] transition-colors"
              >
                <X size={13} />
              </button>
            )}
          </div>

          <div className="flex items-center gap-0.5">
            {FILTERS.map(({ id, label }) => (
              <button
                key={id}
                onClick={() => setFilter(id)}
                className={`h-[32px] px-3 text-[13px] rounded-[6px] transition-colors ${
                  filter === id
                    ? 'bg-white border border-[#E5E7EB] text-[#17191C] font-medium shadow-[0_1px_2px_rgba(0,0,0,0.04)]'
                    : 'text-[#6B7280] hover:bg-[#F3F4F6] hover:text-[#374151]'
                }`}
              >
                {label}
              </button>
            ))}
            <div className="w-px h-4 bg-[#E5E7EB] mx-1.5" />
            <SortDropdown value={sort} onChange={setSort} />
          </div>
        </div>

        {/* Section label */}
        {filtered.length > 0 && (
          <div className="mb-2 px-1">
            <span className="text-[10px] font-semibold uppercase tracking-[0.09em] text-[#B0B7C3]">
              {sectionLabel}
            </span>
          </div>
        )}

        {/* Table */}
        <div className="bg-white border border-[#E5E7EB] rounded-[8px]">
          {filtered.length === 0 ? (
            search ? (
              <EmptyState type="no-results" onClear={() => setSearch('')} />
            ) : (
              <EmptyState type="no-projects" />
            )
          ) : (
            <ProjectTable projects={filtered} />
          )}
        </div>
      </div>
    </div>
  )
}
