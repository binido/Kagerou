import { useTranslation } from 'react-i18next'

import { ConnectionStage } from '@/components/dashboard/ConnectionStage'
import { TelemetryPanel } from '@/components/dashboard/TelemetryPanel'
import { PageHeader } from '@/components/layout/PageHeader'
import { getReachabilityAwarePing } from '@/lib/profile-sorting'
import { regionToCountry } from '@/lib/formatters'
import { useKagerouStore } from '@/store/kagerou-store'

export function DashboardPage() {
  const { t, i18n } = useTranslation('dashboard')
  const { t: tp } = useTranslation('profiles')
  const connected = useKagerouStore((state) => state.connected)
  const tunMode = useKagerouStore((state) => state.tunMode)
  const systemProxy = useKagerouStore((state) => state.systemProxy)
  const profiles = useKagerouStore((state) => state.profiles)
  const profileGroups = useKagerouStore((state) => state.profileGroups)
  const activeProfileId = useKagerouStore((state) => state.activeProfileId)
  const telemetry = useKagerouStore((state) => state.telemetry)
  const sessionTraffic = useKagerouStore((state) => state.sessionTraffic)
  const toggleConnection = useKagerouStore((state) => state.toggleConnection)
  const toggleMode = useKagerouStore((state) => state.toggleMode)
  const activeProfile = profiles.find((profile) => profile.id === activeProfileId) ?? profiles[0]
  const group = activeProfile ? profileGroups.find((g) => g.id === activeProfile.groupId) : undefined
  const groupLabel = group?.kind === 'default' ? tp('group.defaultName') : group?.label
  const profileName = activeProfile
    ? groupLabel
      ? t('connection.profile', { group: groupLabel, name: activeProfile.name })
      : activeProfile.name
    : t('connection.fallbackProfile')
  const ping = activeProfile ? getReachabilityAwarePing(activeProfile) : { value: 'Not tested', tone: 'muted' as const }
  const location = regionToCountry(activeProfile?.region ?? '', i18n.resolvedLanguage ?? 'en') ?? t('connection.fallbackLocation')

  return (
    <div className="min-h-screen min-w-0 bg-canvas px-6 py-7 lg:px-12 lg:py-9">
      <div className="mx-auto w-full max-w-[1120px]">
        <PageHeader eyebrow={t('page.eyebrow')} title={t('page.title')} />
        <div className="mt-6 grid grid-cols-12 items-stretch gap-5 max-[1179px]:grid-cols-1">
          <section className="col-span-8 min-w-0 max-[1179px]:col-span-1" aria-labelledby="connection-stage-title">
            <ConnectionStage
              connected={connected}
              location={location}
              profileName={profileName}
              ping={ping}
              systemProxy={systemProxy}
              tunMode={tunMode}
              onToggleConnection={toggleConnection}
              onToggleMode={toggleMode}
            />
          </section>
          <aside className="col-span-4 min-w-0 max-[1179px]:col-span-1" aria-label={t('telemetry.title')}>
            <TelemetryPanel data={telemetry} sessionTraffic={sessionTraffic} />
          </aside>
        </div>
      </div>
    </div>
  )
}
