import { useEffect, useState } from 'react'
import { MapPin } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Card } from '@/components/ui/card'
import { ConnectionDial } from '@/components/dashboard/ConnectionDial'
import { ConnectionTrafficReadouts } from '@/components/dashboard/ConnectionTrafficReadouts'
import { SpeedSparkline } from '@/components/dashboard/SpeedSparkline'
import { formatUptime } from '@/lib/formatters'
import type { SessionTraffic, TestResult, TrafficSample } from '@/types/kagerou'

/** Ticks once a second while connected, and not at all otherwise. The
 * stored `now` is whatever the last tick saw, so the first second after a
 * connection reads as 0:00 — which is what it is. */
function useUptime(connectedSince: number | null) {
  const [now, setNow] = useState(() => Date.now())

  useEffect(() => {
    if (connectedSince === null) return
    const timer = window.setInterval(() => setNow(Date.now()), 1000)
    return () => window.clearInterval(timer)
  }, [connectedSince])

  return connectedSince === null ? null : formatUptime(now - connectedSince)
}

interface ConnectionStageProps {
  profileName: string
  location: string
  connected: boolean
  connectedSince: number | null
  ping: TestResult
  latestDownload: number
  latestUpload: number
  trafficHistory: TrafficSample[]
  activeConnections: number | null
  sessionTraffic: SessionTraffic
  onToggleConnection: () => void
}

export function ConnectionStage({
  profileName,
  location,
  connected,
  connectedSince,
  ping,
  latestDownload,
  latestUpload,
  trafficHistory,
  activeConnections,
  sessionTraffic,
  onToggleConnection,
}: ConnectionStageProps) {
  const { t } = useTranslation('dashboard')
  const uptime = useUptime(connectedSince)

  return (
    <Card className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-[10px] border border-hairline bg-surface p-0 shadow-none">
      <div className="grid min-h-0 flex-1 grid-cols-[auto_minmax(0,1fr)] items-stretch gap-7 p-7 max-[860px]:grid-cols-1 max-[860px]:justify-items-center max-[860px]:gap-5 max-[860px]:p-5">
        <div className="flex flex-col items-center justify-center gap-2.5">
          <ConnectionDial connected={connected} onToggle={onToggleConnection} />
          <p className="type-data text-center text-muted-copy">
            {uptime ? t('connection.uptime', { value: uptime }) : t('connection.idleHint')}
          </p>
        </div>

        <div className="flex min-h-0 min-w-0 flex-col max-[860px]:w-full max-[860px]:text-center">
          <p className="type-eyebrow">{t('connection.activeVpn')}</p>
          <h2 className="type-display mt-2 min-w-0 truncate text-[22px] leading-tight tracking-[-0.01em] text-primary" id="connection-stage-title">
            {profileName}
          </h2>
          <p className="mt-2 flex min-w-0 items-center gap-2 truncate text-[14px] text-body max-[860px]:justify-center">
            <MapPin aria-hidden="true" className="size-4 shrink-0 text-muted-copy" strokeWidth={1.7} />
            <span className="truncate">{location}</span>
          </p>

          <div className="mt-5 border-t border-hairline pt-4">
            <ConnectionTrafficReadouts
              activeConnections={activeConnections}
              latestDownload={latestDownload}
              latestUpload={latestUpload}
              ping={ping}
              sessionTraffic={sessionTraffic}
            />
          </div>

          <SpeedSparkline className="mt-4 max-h-[240px] flex-1" history={trafficHistory} />
        </div>
      </div>
    </Card>
  )
}
