import type { Technology } from '../types'

const COLORS: Record<string, { bg: string; text: string }> = {
  node: { bg: '#F0FDF4', text: '#15803D' },
  vue: { bg: '#ECFDF5', text: '#059669' },
  react: { bg: '#EFF6FF', text: '#1D4ED8' },
  typescript: { bg: '#EEF2FF', text: '#4338CA' },
  rust: { bg: '#FFF7ED', text: '#9A3412' },
  java: { bg: '#FEF9C3', text: '#854D0E' },
  spring: { bg: '#F0FDF4', text: '#166534' },
  python: { bg: '#FFFBEB', text: '#92400E' },
  nextjs: { bg: '#F3F4F6', text: '#374151' },
  neutral: { bg: '#F3F4F6', text: '#4B5563' },
}

export default function TechBadge({ tech }: { tech: Technology }) {
  const { bg, text } = COLORS[tech.variant] ?? COLORS.neutral
  return (
    <span
      style={{ backgroundColor: bg, color: text }}
      className="inline-flex items-center px-[7px] py-[2px] text-[11px] font-medium rounded-[4px] leading-[18px]"
    >
      {tech.name}
    </span>
  )
}
