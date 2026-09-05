import { useTranslation } from 'react-i18next'

import { LanguageSwitcher } from '@/components/settings/LanguageSwitcher'
import { PageContainer } from '@/components/layout/PageContainer'
import { SettingsFooter } from '@/components/settings/SettingsFooter'
import { PageHeader } from '@/components/layout/PageHeader'
import { SettingNumberRow } from '@/components/settings/SettingNumberRow'
import { SettingSelectRow } from '@/components/settings/SettingSelectRow'
import { SettingSwitchRow } from '@/components/settings/SettingSwitchRow'
import { SettingsSection } from '@/components/settings/SettingsSection'
import { ThemePicker } from '@/components/settings/ThemePicker'
import { useKagerouStore } from '@/store/kagerou-store'
import type { GroupSortMode, SubscriptionUpdateInterval, TunInterface } from '@/types/kagerou'

export function SettingsPage() {
  const { t } = useTranslation('settings')
  const settings = useKagerouStore((state) => state.settings)
  const setTheme = useKagerouStore((state) => state.setTheme)
  const updateSettings = useKagerouStore((state) => state.updateSettings)
  const subscriptionIntervalOptions = [
    { value: '5', label: t('options.interval5') },
    { value: '10', label: t('options.interval10') },
    { value: '15', label: t('options.interval15') },
    { value: '30', label: t('options.interval30') },
    { value: '60', label: t('options.interval60') },
    { value: 'custom', label: t('options.custom') },
  ] as const
  const groupSortOptions = [
    { value: 'ping', label: t('options.sortPing') },
    { value: 'name', label: t('options.sortName') },
    { value: 'protocol', label: t('options.sortProtocol') },
  ] as const

  return (
    <PageContainer className="flex flex-col" contentClassName="flex flex-1 flex-col">
      <div className="flex w-full max-w-[680px] flex-1 flex-col">
        <PageHeader actions={<p className="mb-0.5 text-[12px] leading-4 text-muted-copy">{t('page.instant')}</p>} eyebrow={t('page.eyebrow')} title={t('page.title')} />
        <div className="mt-12 flex flex-1 flex-col">
          <SettingsSection title={t('sections.connection')}>
            <SettingSwitchRow checked={settings.tunMode} description={t('descriptions.tunMode')} label={t('fields.tunMode')} onChange={(tunMode) => updateSettings({ tunMode })} />
            <SettingSwitchRow checked={settings.systemProxy} description={t('descriptions.systemProxyUnavailable')} disabled label={t('fields.systemProxy')} onChange={(systemProxy) => updateSettings({ systemProxy })} />
          </SettingsSection>
          <SettingsSection title={t('sections.appearance')}>
            <div className="flex min-h-14 items-center justify-between gap-8 border-b border-hairline max-[639px]:flex-col max-[639px]:items-stretch max-[639px]:gap-3 max-[639px]:py-3"><span className="text-[14px] leading-5 text-body">{t('fields.theme')}</span><ThemePicker onChange={setTheme} value={settings.theme} /></div>
            <LanguageSwitcher />
          </SettingsSection>
          <SettingsSection title={t('sections.startup')}><SettingSwitchRow checked={settings.startup} label={t('fields.startAutomatically')} onChange={(startup) => updateSettings({ startup })} /></SettingsSection>
          <SettingsSection title={t('sections.subscriptions')}>
            <SettingSwitchRow checked={settings.autoUpdateSubscriptions} description={t('descriptions.autoUpdateSubscriptions')} label={t('fields.autoUpdateSubscriptions')} onChange={(autoUpdateSubscriptions) => updateSettings({ autoUpdateSubscriptions })} />
            {settings.autoUpdateSubscriptions ? (
              <>
                <SettingSelectRow id="subscription-update-interval" label={t('fields.updateInterval')} onChange={(subscriptionUpdateInterval) => updateSettings({ subscriptionUpdateInterval: subscriptionUpdateInterval as SubscriptionUpdateInterval })} options={subscriptionIntervalOptions} value={settings.subscriptionUpdateInterval} />
                {settings.subscriptionUpdateInterval === 'custom' ? <SettingNumberRow description={t('descriptions.customInterval')} id="custom-subscription-update-minutes" label={t('fields.customInterval')} onChange={(customSubscriptionUpdateMinutes) => updateSettings({ customSubscriptionUpdateMinutes })} value={settings.customSubscriptionUpdateMinutes} /> : null}
              </>
            ) : null}
          </SettingsSection>
          <SettingsSection title={t('sections.groups')}><SettingSelectRow id="group-sort" label={t('fields.sortVpns')} onChange={(groupSort) => updateSettings({ groupSort: groupSort as GroupSortMode })} options={groupSortOptions} value={settings.groupSort} /></SettingsSection>
          <SettingsSection title={t('sections.network')}><SettingSelectRow id="tun-interface" label={t('fields.tunInterface')} onChange={(tunInterface) => updateSettings({ tunInterface: tunInterface as TunInterface })} options={[{ value: 'utun / tun0', label: t('options.tunBoth') }, { value: 'utun', label: t('options.utun') }, { value: 'tun0', label: t('options.tun0') }]} value={settings.tunInterface} /></SettingsSection>
          <div className="mt-auto">
            <SettingsFooter />
          </div>
        </div>
      </div>
    </PageContainer>
  )
}
