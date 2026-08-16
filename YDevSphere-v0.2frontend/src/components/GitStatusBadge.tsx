import type { GitStatus } from '../types'

export default function GitStatusBadge({ status }: { status: GitStatus }) {
  if (status.type === 'clean') {
    return (
      <span className="flex items-center gap-1.5 text-[12px] text-[#16A34A]">
        <span className="w-[6px] h-[6px] rounded-full bg-[#16A34A] flex-shrink-0" />
        Clean
      </span>
    )
  }

  if (status.type === 'dirty') {
    return (
      <span className="flex items-center gap-1.5 text-[12px] text-[#D97706]">
        <span className="w-[6px] h-[6px] rounded-full bg-[#D97706] flex-shrink-0" />
        {status.changes} changed
      </span>
    )
  }

  if (status.type === 'detached') {
    return (
      <span className="flex items-center gap-1.5 text-[12px] text-[#DC2626]">
        <span className="w-[6px] h-[6px] rounded-full bg-[#DC2626] flex-shrink-0" />
        Detached
      </span>
    )
  }

  return <span className="text-[12px] text-[#9CA3AF]">— No Git</span>
}
