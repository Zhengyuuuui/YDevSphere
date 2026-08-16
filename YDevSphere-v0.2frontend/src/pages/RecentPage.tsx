import type { Project } from '../types'
import ProjectTable from '../components/ProjectTable'

export default function RecentPage({ projects }: { projects: Project[] }) {
  const recent = projects.filter((p) => p.lastOpenedAt).slice(0, 8)

  return (
    <div className="min-h-full bg-[#F7F8FA]">
      <div className="max-w-[1140px] mx-auto px-8 py-7">
        <div className="mb-5">
          <h1 className="text-[22px] font-semibold text-[#17191C] tracking-tight leading-tight">
            Recent
          </h1>
          <p className="text-[13px] text-[#9CA3AF] mt-1">Projects you opened recently</p>
        </div>

        {recent.length > 0 && (
          <div className="mb-2 px-1">
            <span className="text-[10px] font-semibold uppercase tracking-[0.09em] text-[#B0B7C3]">
              Recently Opened
            </span>
          </div>
        )}

        <div className="bg-white border border-[#E5E7EB] rounded-[8px]">
          <ProjectTable projects={recent} config={{ showLastOpened: true }} />
        </div>
      </div>
    </div>
  )
}
