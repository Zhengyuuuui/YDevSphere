import { AreaChart, Area, ResponsiveContainer, Tooltip } from 'recharts'
import { ArrowRight } from 'lucide-react'
import type { Project } from '../types'

const ACTIVITY = [
  { day: 'Mon', commits: 4 },
  { day: 'Tue', commits: 7 },
  { day: 'Wed', commits: 11 },
  { day: 'Thu', commits: 6 },
  { day: 'Fri', commits: 14 },
  { day: 'Sat', commits: 3 },
  { day: 'Sun', commits: 8 },
]

const TECH_STACK = [
  { name: 'JavaScript', count: 38, color: '#F7DF1E' },
  { name: 'TypeScript', count: 27, color: '#3178C6' },
  { name: 'Rust', count: 14, color: '#CE422B' },
  { name: 'Python', count: 9, color: '#3776AB' },
]

const RECENT: { name: string; tech: string; timeAgo: string }[] = [
  { name: 'YDevSphere', tech: 'Rust · Vue · Tauri', timeAgo: '12m' },
  { name: 'NewAPI', tech: 'Go · SQLite · Docker', timeAgo: '1h' },
  { name: 'Yunex Tools', tech: 'Vue · TypeScript', timeAgo: '3h' },
  { name: 'LifeKline', tech: 'React · PostgreSQL', timeAgo: '1d' },
]

function StatCard({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="flex flex-col gap-1 min-w-0">
      <span className="text-[28px] font-semibold text-[#17191C] leading-none tracking-tight tabular-nums">
        {value}
      </span>
      <span className="text-[12px] text-[#9CA3AF]">{label}</span>
    </div>
  )
}

function getHour() {
  const h = new Date().getHours()
  if (h < 12) return 'morning'
  if (h < 17) return 'afternoon'
  return 'evening'
}

interface OverviewPageProps {
  projects: Project[]
  onNavigate: (page: 'projects') => void
}

export default function OverviewPage({ onNavigate }: OverviewPageProps) {
  const maxCommits = Math.max(...ACTIVITY.map((d) => d.commits))
  const totalActivity = ACTIVITY.reduce((s, d) => s + d.commits, 0)

  return (
    <div className="min-h-full bg-[#F7F8FA]">
      <div className="max-w-[1060px] mx-auto px-8 py-8">
        {/* Greeting */}
        <div className="mb-7">
          <h1 className="text-[22px] font-semibold text-[#17191C] tracking-tight leading-tight">
            Good {getHour()}, 小喻
          </h1>
          <p className="text-[13px] text-[#9CA3AF] mt-1">
            Here&apos;s what&apos;s happening across your workspace.
          </p>
        </div>

        {/* Stat row */}
        <div className="flex items-center gap-8 mb-7 pb-7 border-b border-[#EAEDF0]">
          <StatCard value={127} label="Projects" />
          <div className="w-px h-8 bg-[#EAEDF0]" />
          <StatCard value={42} label="Repositories" />
          <div className="w-px h-8 bg-[#EAEDF0]" />
          <StatCard value={18} label="Active" />
        </div>

        {/* Two-column grid */}
        <div className="grid grid-cols-[1fr_260px] gap-5 mb-5">
          {/* Activity chart */}
          <div className="bg-white border border-[#EAEDF0] rounded-[10px] p-5">
            <div className="flex items-center justify-between mb-4">
              <span className="text-[13px] font-semibold text-[#17191C]">Workspace Activity</span>
              <span className="text-[12px] text-[#9CA3AF]">{totalActivity} commits this week</span>
            </div>

            <div className="h-[120px]">
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={ACTIVITY} margin={{ top: 4, right: 0, left: 0, bottom: 0 }}>
                  <defs>
                    <linearGradient id="actGrad" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="0%" stopColor="#2563EB" stopOpacity={0.15} />
                      <stop offset="100%" stopColor="#2563EB" stopOpacity={0} />
                    </linearGradient>
                  </defs>
                  <Tooltip
                    content={({ active, payload, label }) => {
                      if (!active || !payload?.length) return null
                      return (
                        <div className="bg-[#17191C] text-white text-[11px] px-2 py-1 rounded-[5px]">
                          {label}: {payload[0].value} commits
                        </div>
                      )
                    }}
                    cursor={false}
                  />
                  <Area
                    type="monotone"
                    dataKey="commits"
                    stroke="#2563EB"
                    strokeWidth={1.5}
                    fill="url(#actGrad)"
                    dot={false}
                    activeDot={{ r: 3, fill: '#2563EB', strokeWidth: 0 }}
                  />
                </AreaChart>
              </ResponsiveContainer>
            </div>

            <div className="flex justify-between mt-3">
              {ACTIVITY.map((d) => (
                <span key={d.day} className="text-[11px] text-[#B0B7C3]">
                  {d.day}
                </span>
              ))}
            </div>
          </div>

          {/* Tech stack */}
          <div className="bg-white border border-[#EAEDF0] rounded-[10px] p-5">
            <span className="text-[13px] font-semibold text-[#17191C] block mb-4">
              Technology Stack
            </span>

            <div className="space-y-3">
              {TECH_STACK.map((t) => (
                <div key={t.name} className="flex items-center justify-between gap-3">
                  <div className="flex items-center gap-2 min-w-0">
                    <span
                      className="w-2 h-2 rounded-full flex-shrink-0"
                      style={{ background: t.color }}
                    />
                    <span className="text-[13px] text-[#374151] truncate">{t.name}</span>
                  </div>
                  <div className="flex items-center gap-2 flex-shrink-0">
                    <div className="w-[60px] h-[4px] bg-[#F3F4F6] rounded-full overflow-hidden">
                      <div
                        className="h-full rounded-full"
                        style={{
                          width: `${(t.count / maxCommits) * 100}%`,
                          background: t.color,
                          opacity: 0.7,
                        }}
                      />
                    </div>
                    <span className="text-[12px] text-[#9CA3AF] w-5 text-right tabular-nums">
                      {t.count}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Recent Projects */}
        <div className="bg-white border border-[#EAEDF0] rounded-[10px]">
          <div className="flex items-center justify-between px-5 py-4 border-b border-[#F3F4F6]">
            <span className="text-[13px] font-semibold text-[#17191C]">Recent Projects</span>
            <button
              onClick={() => onNavigate('projects')}
              className="flex items-center gap-1 text-[12px] text-[#6B7280] hover:text-[#2563EB] transition-colors"
            >
              View all
              <ArrowRight size={12} />
            </button>
          </div>

          <div className="divide-y divide-[#F3F4F6]">
            {RECENT.map((r) => (
              <div
                key={r.name}
                className="flex items-center justify-between px-5 py-3 hover:bg-[#FAFAFA] transition-colors cursor-pointer"
              >
                <div className="flex flex-col gap-0.5">
                  <span className="text-[13px] font-medium text-[#17191C]">{r.name}</span>
                  <span className="text-[11px] text-[#9CA3AF]">{r.tech}</span>
                </div>
                <span className="text-[12px] text-[#B0B7C3] tabular-nums flex-shrink-0">
                  {r.timeAgo}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
