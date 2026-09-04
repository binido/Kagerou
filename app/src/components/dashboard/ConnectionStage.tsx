import { MapPin } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Card } from '@/components/ui/card'
import { ConnectionDial } from '@/components/dashboard/ConnectionDial'
import { ConnectionTrafficReadouts } from '@/components/dashboard/ConnectionTrafficReadouts'
import type { SessionTraffic, TestResult } from '@/types/kagerou'

interface ConnectionStageProps {
  profileName: string
  location: string
  connected: boolean
  ping: TestResult
  latestDownload: number
  latestUpload: number
  sessionTraffic: SessionTraffic
  onToggleConnection: () => void
}

export function ConnectionStage({ profileName, location, connected, ping, latestDownload, latestUpload, sessionTraffic, onToggleConnection }: ConnectionStageProps) {
  const { t } = useTranslation('dashboard')

  return (
    <Card className="mx-auto flex min-h-0 w-full max-w-[1024px] flex-1 flex-col overflow-hidden rounded-[10px] border border-hairline bg-surface p-0 shadow-none">
      <div className="grid min-w-0 shrink-0 grid-cols-[minmax(0,1fr)_auto] items-start gap-6 border-b border-hairline px-8 py-7 max-[980px]:gap-4 max-[980px]:px-6 max-[980px]:py-6 max-[760px]:grid-cols-1 max-[760px]:gap-4 max-[760px]:px-5 max-[760px]:py-5">
        <div className="min-w-0">
          <p className="type-eyebrow">{t('connection.activeVpn')}</p>
          <h2 className="type-display mt-2 min-w-0 truncate text-[22px] leading-tight tracking-[-0.01em] text-primary" id="connection-stage-title">
            {profileName}
          </h2>
          <p className="mt-2 flex min-w-0 items-center gap-2 truncate text-[14px] text-body">
            <MapPin aria-hidden="true" className="size-4 shrink-0 text-muted-copy" strokeWidth={1.7} />
            <span className="truncate">{location}</span>
          </p>
        </div>
        <ConnectionTrafficReadouts latestDownload={latestDownload} latestUpload={latestUpload} ping={ping} sessionTraffic={sessionTraffic} />
      </div>
      <div className="flex min-h-0 flex-1 items-center justify-center p-8 max-[760px]:p-5">
        <ConnectionDial connected={connected} onToggle={onToggleConnection} />
      </div>
    </Card>
  )
}
