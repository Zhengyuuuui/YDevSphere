import { useState } from 'react'

const SECTIONS = ['General', 'Workspace', 'Editor', 'Privacy', 'Database', 'About'] as const
type Section = (typeof SECTIONS)[number]

function Toggle({ enabled, onToggle }: { enabled: boolean; onToggle: () => void }) {
  return (
    <button
      onClick={onToggle}
      className={`w-9 h-5 rounded-full relative transition-colors duration-150 flex-shrink-0 ${
        enabled ? 'bg-[#2563EB]' : 'bg-[#D1D5DB]'
      }`}
    >
      <span
        className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform duration-150 ${
          enabled ? 'translate-x-4' : 'translate-x-0.5'
        }`}
        style={{ boxShadow: '0 1px 3px rgba(0,0,0,0.15)' }}
      />
    </button>
  )
}

function SettingRow({
  label,
  description,
  enabled,
  onToggle,
  isLast,
}: {
  label: string
  description: string
  enabled: boolean
  onToggle: () => void
  isLast?: boolean
}) {
  return (
    <div
      className={`flex items-center justify-between py-3.5 ${!isLast ? 'border-b border-[#F3F4F6]' : ''}`}
    >
      <div>
        <div className="text-[13px] font-medium text-[#17191C]">{label}</div>
        <div className="text-[12px] text-[#9CA3AF] mt-0.5">{description}</div>
      </div>
      <Toggle enabled={enabled} onToggle={onToggle} />
    </div>
  )
}

export default function SettingsPage() {
  const [activeSection, setActiveSection] = useState<Section>('General')
  const [toggles, setToggles] = useState({
    launch: false,
    autoScan: true,
    showPath: true,
  })

  const toggle = (key: keyof typeof toggles) =>
    setToggles((prev) => ({ ...prev, [key]: !prev[key] }))

  return (
    <div className="min-h-full bg-[#F7F8FA]">
      <div className="max-w-[1140px] mx-auto px-8 py-7">
        <div className="mb-6">
          <h1 className="text-[22px] font-semibold text-[#17191C] tracking-tight leading-tight">
            Settings
          </h1>
        </div>

        <div className="flex gap-5">
          {/* Settings nav */}
          <div className="w-[172px] flex-shrink-0">
            <nav className="space-y-0.5">
              {SECTIONS.map((section) => (
                <button
                  key={section}
                  onClick={() => setActiveSection(section)}
                  className={`w-full flex items-center px-3 py-[7px] rounded-[6px] text-[13px] text-left transition-colors ${
                    activeSection === section
                      ? 'bg-[#EEF2FF] text-[#2563EB] font-medium'
                      : 'text-[#6B7280] hover:bg-[#F3F4F6] hover:text-[#374151]'
                  }`}
                >
                  {section}
                </button>
              ))}
            </nav>
          </div>

          {/* Settings content */}
          <div className="flex-1 bg-white border border-[#E5E7EB] rounded-[8px] px-6 py-5">
            {activeSection === 'General' && (
              <>
                <h2 className="text-[14px] font-semibold text-[#17191C] mb-4">General</h2>
                <SettingRow
                  label="Launch at startup"
                  description="Open YDevSphere when you log in"
                  enabled={toggles.launch}
                  onToggle={() => toggle('launch')}
                />
                <SettingRow
                  label="Auto-scan on launch"
                  description="Scan workspaces when the app starts"
                  enabled={toggles.autoScan}
                  onToggle={() => toggle('autoScan')}
                />
                <SettingRow
                  label="Show path in project list"
                  description="Display the file path below each project name"
                  enabled={toggles.showPath}
                  onToggle={() => toggle('showPath')}
                  isLast
                />
              </>
            )}

            {activeSection !== 'General' && (
              <div className="flex flex-col items-center justify-center py-14">
                <p className="text-[14px] font-medium text-[#17191C] mb-1.5">{activeSection}</p>
                <p className="text-[13px] text-[#9CA3AF]">This section is coming soon.</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
