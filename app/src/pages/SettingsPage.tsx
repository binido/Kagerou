import { PageHeader } from '@/components/layout/PageHeader'
import { SettingSelectRow } from '@/components/settings/SettingSelectRow'
import { SettingSwitchRow } from '@/components/settings/SettingSwitchRow'
import { SettingsSection } from '@/components/settings/SettingsSection'
import { ThemeToggle } from '@/components/settings/ThemeToggle'
import { useKagerouStore } from '@/store/kagerou-store'
import type { Language, Theme, TunInterface } from '@/types/kagerou'

export function SettingsPage() {
  const settings = useKagerouStore((state) => state.settings)
  const updateSettings = useKagerouStore((state) => state.updateSettings)

  return (
    <div className="min-h-screen min-w-0 bg-canvas px-6 py-8 lg:px-[68px] lg:py-[52px]">
      <div className="w-full max-w-[920px]">
        <PageHeader actions={<p className="mb-0.5 text-[12px] leading-4 text-muted-copy">Changes apply instantly</p>} title="Settings" />
        <div className="mt-12 w-[680px] max-w-full">
          <SettingsSection title="Appearance">
            <div className="flex min-h-14 items-center justify-between gap-8 border-b border-white/[0.055]"><span className="text-[14px] leading-5 text-body">Theme</span><ThemeToggle onChange={(theme: Theme) => updateSettings({ theme })} value={settings.theme} /></div>
            <SettingSelectRow id="language" label="Language" onChange={(language) => updateSettings({ language: language as Language })} options={['English', '中文', '日本語']} value={settings.language} />
          </SettingsSection>
          <SettingsSection title="Startup"><SettingSwitchRow checked={settings.startup} label="Start automatically" onChange={(startup) => updateSettings({ startup })} /></SettingsSection>
          <SettingsSection title="Network"><SettingSelectRow id="tun-interface" label="TUN interface" onChange={(tunInterface) => updateSettings({ tunInterface: tunInterface as TunInterface })} options={['utun / tun0', 'utun', 'tun0']} value={settings.tunInterface} /></SettingsSection>
        </div>
      </div>
    </div>
  )
}
