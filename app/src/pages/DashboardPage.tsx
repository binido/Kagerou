import { useTranslation } from 'react-i18next'

import { ConnectionStage } from '@/components/dashboard/ConnectionStage'
import { PageContainer } from '@/components/layout/PageContainer'
import { PageHeader } from '@/components/layout/PageHeader'
import { regionToCountry } from '@/lib/formatters'
import { useKagerouStore } from '@/store/kagerou-store'

export function DashboardPage() {
  const { t, i18n } = useTranslation('dashboard')
  const { t: tp } = useTranslation('profiles')
  const connected = useKagerouStore((state) => state.connected)
  const profiles = useKagerouStore((state) => state.profiles)
  const profileGroups = useKagerouStore((state) => state.profileGroups)
  const activeProfileId = useKagerouStore((state) => state.activeProfileId)
  const trafficSample = useKagerouStore((state) => state.trafficSample)
  const sessionTraffic = useKagerouStore((state) => state.sessionTraffic)
  const toggleConnection = useKagerouStore((state) => state.toggleConnection)
  const activeProfile = profiles.find((profile) => profile.id === activeProfileId) ?? profiles[0]
  const group = activeProfile ? profileGroups.find((g) => g.id === activeProfile.groupId) : undefined
  const groupLabel = group?.kind === 'default' ? tp('group.defaultName') : group?.label
  const profileName = activeProfile
    ? groupLabel
      ? t('connection.profile', { group: groupLabel, name: activeProfile.name })
      : activeProfile.name
    : t('connection.fallbackProfile')
  const ping = activeProfile ? activeProfile.url : { value: 'Not tested', tone: 'muted' as const }
  const location = regionToCountry(activeProfile?.region ?? '', i18n.resolvedLanguage ?? 'en') ?? t('connection.fallbackLocation')

  return (
    <PageContainer className="flex h-dvh min-h-0 flex-col overflow-hidden" contentClassName="flex h-full min-h-0 flex-col">
      <PageHeader eyebrow={t('page.eyebrow')} title={t('page.title')} />
      <section aria-labelledby="connection-stage-title" className="mt-6 flex min-h-0 flex-1">
        <ConnectionStage
          connected={connected}
          latestDownload={trafficSample.download}
          latestUpload={trafficSample.upload}
          location={location}
          onToggleConnection={toggleConnection}
          ping={ping}
          profileName={profileName}
          sessionTraffic={sessionTraffic}
        />
      </section>
    </PageContainer>
  )
}
