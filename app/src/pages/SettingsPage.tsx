import { PageHeader } from '@/components/layout/PageHeader'
import { SettingNumberRow } from '@/components/settings/SettingNumberRow'
import { SettingSelectRow } from '@/components/settings/SettingSelectRow'
import { SettingSwitchRow } from '@/components/settings/SettingSwitchRow'
import { SettingsSection } from '@/components/settings/SettingsSection'
import { groupSortLabels } from '@/lib/profile-sorting'
import { ThemeToggle } from '@/components/settings/ThemeToggle'
import { useKagerouStore } from '@/store/kagerou-store'
import type { GroupSortMode, Language, SubscriptionUpdateInterval, Theme, TunInterface } from '@/types/kagerou'

const subscriptionIntervalOptions = [
  { value: '5', label: '5 minutes' },
  { value: '10', label: '10 minutes' },
  { value: '15', label: '15 minutes' },
  { value: '30', label: '30 minutes' },
  { value: '60', label: '60 minutes' },
  { value: 'custom', label: 'Custom' },
] as const

const groupSortOptions = [
  { value: 'ping', label: groupSortLabels.ping },
  { value: 'name', label: groupSortLabels.name },
  { value: 'protocol', label: groupSortLabels.protocol },
] as const

export function SettingsPage() {
  const settings = useKagerouStore((state) => state.settings)
  const updateSettings = useKagerouStore((state) => state.updateSettings)

  return (
    <div className="min-h-screen min-w-0 bg-canvas px-6 py-8 lg:px-[68px] lg:py-[52px]">
      <div className="w-full max-w-[920px] 2xl:mx-auto">
        <PageHeader actions={<p className="mb-0.5 text-[12px] leading-4 text-muted-copy">Changes apply instantly</p>} title="Settings" />
        <div className="mt-12 w-[680px] max-w-full">
          <SettingsSection title="Appearance">
            <div className="flex min-h-14 items-center justify-between gap-8 border-b border-white/[0.055]"><span className="text-[14px] leading-5 text-body">Theme</span><ThemeToggle onChange={(theme: Theme) => updateSettings({ theme })} value={settings.theme} /></div>
            <SettingSelectRow id="language" label="Language" onChange={(language) => updateSettings({ language: language as Language })} options={['English', '中文', '日本語']} value={settings.language} />
          </SettingsSection>
          <SettingsSection title="Startup"><SettingSwitchRow checked={settings.startup} label="Start automatically" onChange={(startup) => updateSettings({ startup })} /></SettingsSection>
          <SettingsSection title="Subscriptions">
            <SettingSwitchRow checked={settings.autoUpdateSubscriptions} description="Keep URL subscription groups fresh automatically." label="Auto-update subscriptions" onChange={(autoUpdateSubscriptions) => updateSettings({ autoUpdateSubscriptions })} />
            {settings.autoUpdateSubscriptions ? (
              <>
                <SettingSelectRow id="subscription-update-interval" label="Update interval" onChange={(subscriptionUpdateInterval) => updateSettings({ subscriptionUpdateInterval: subscriptionUpdateInterval as SubscriptionUpdateInterval })} options={subscriptionIntervalOptions} value={settings.subscriptionUpdateInterval} />
                {settings.subscriptionUpdateInterval === 'custom' ? <SettingNumberRow description="Choose how often Kagerou checks remote subscriptions." id="custom-subscription-update-minutes" label="Custom interval (minutes)" onChange={(customSubscriptionUpdateMinutes) => updateSettings({ customSubscriptionUpdateMinutes })} value={settings.customSubscriptionUpdateMinutes} /> : null}
              </>
            ) : null}
          </SettingsSection>
          <SettingsSection title="Groups"><SettingSelectRow id="group-sort" label="Sort VPNs by" onChange={(groupSort) => updateSettings({ groupSort: groupSort as GroupSortMode })} options={groupSortOptions} value={settings.groupSort} /></SettingsSection>
          <SettingsSection title="Network"><SettingSelectRow id="tun-interface" label="TUN interface" onChange={(tunInterface) => updateSettings({ tunInterface: tunInterface as TunInterface })} options={['utun / tun0', 'utun', 'tun0']} value={settings.tunInterface} /></SettingsSection>
        </div>
      </div>
    </div>
  )
}
