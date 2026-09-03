import { useKagerouStore } from '@/store/kagerou-store'

import { ConnectionStage } from '@/components/dashboard/ConnectionStage'
import { TelemetryPanel } from '@/components/dashboard/TelemetryPanel'
import { PageHeader } from '@/components/layout/PageHeader'

export function DashboardPage() {
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
        <PageHeader eyebrow="Connection" title="Dashboard" />
        <div className="mt-6 grid grid-cols-12 items-stretch gap-5 max-[1179px]:grid-cols-1">
          <section className="col-span-8 min-w-0 max-[1179px]:col-span-1" aria-labelledby="connection-stage-title">
            <ConnectionStage
              connected={connected}
              location={activeProfile ? `${activeProfile.name.split(' ')[0]}, United States` : 'Seattle, United States'}
              profileName={activeProfile ? `Aurora / ${activeProfile.name}` : 'Aurora / Seattle 03'}
              systemProxy={systemProxy}
              tunMode={tunMode}
              onDisconnect={toggleConnection}
              onToggleConnection={toggleConnection}
              onToggleMode={toggleMode}
            />
          </section>
          <aside className="col-span-4 min-w-0 max-[1179px]:col-span-1" aria-label="Live telemetry">
            <TelemetryPanel data={telemetry} />
          </aside>
        </div>
      </div>
    </div>
  )
}
