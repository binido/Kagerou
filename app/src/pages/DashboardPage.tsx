import { useTranslation } from 'react-i18next'

import { ConnectionStage } from '@/components/dashboard/ConnectionStage'
import { TelemetryPanel } from '@/components/dashboard/TelemetryPanel'
import { PageHeader } from '@/components/layout/PageHeader'
import { useKagerouStore } from '@/store/kagerou-store'

export function DashboardPage() {
  const { t } = useTranslation('dashboard')
  const connected = useKagerouStore((state) => state.connected)
  const tunMode = useKagerouStore((state) => state.tunMode)
  const systemProxy = useKagerouStore((state) => state.systemProxy)
  const profiles = useKagerouStore((state) => state.profiles)
  const activeProfileId = useKagerouStore((state) => state.activeProfileId)
  const telemetry = useKagerouStore((state) => state.telemetry)
  const toggleConnection = useKagerouStore((state) => state.toggleConnection)
  const toggleMode = useKagerouStore((state) => state.toggleMode)
  const activeProfile = profiles.find((profile) => profile.id === activeProfileId) ?? profiles[0]

  return (
    <div className="min-h-screen min-w-0 bg-canvas px-6 py-7 lg:px-12 lg:py-9">
      <div className="mx-auto w-full max-w-[1120px]">
        <PageHeader eyebrow={t('page.eyebrow')} title={t('page.title')} />
        <div className="mt-6 grid grid-cols-12 items-stretch gap-5 max-[1179px]:grid-cols-1">
          <section className="col-span-8 min-w-0 max-[1179px]:col-span-1" aria-labelledby="connection-stage-title">
            <ConnectionStage
              connected={connected}
              location={activeProfile ? t('connection.location', { city: activeProfile.name.split(' ')[0] }) : t('connection.fallbackLocation')}
              profileName={activeProfile ? t('connection.profile', { name: activeProfile.name }) : t('connection.fallbackProfile')}
              systemProxy={systemProxy}
              tunMode={tunMode}
              onDisconnect={toggleConnection}
              onToggleConnection={toggleConnection}
              onToggleMode={toggleMode}
            />
          </section>
          <aside className="col-span-4 min-w-0 max-[1179px]:col-span-1" aria-label={t('telemetry.title')}>
            <TelemetryPanel data={telemetry} />
          </aside>
        </div>
      </div>
    </div>
  )
}
