import { MapPin } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Card } from '@/components/ui/card'
import { ResultBadge } from '@/components/common/ResultBadge'
import { ConnectionDial } from '@/components/dashboard/ConnectionDial'
import { ModeSwitches } from '@/components/dashboard/ModeSwitches'
import type { TestResult } from '@/types/kagerou'

interface ConnectionStageProps {
  profileName: string
  location: string
  connected: boolean
  ping: TestResult
  tunMode: boolean
  systemProxy: boolean
  onToggleConnection: () => void
  onToggleMode: (mode: 'tun' | 'proxy') => void
}

export function ConnectionStage({ profileName, location, connected, ping, tunMode, systemProxy, onToggleConnection, onToggleMode }: ConnectionStageProps) {
  const { t } = useTranslation('dashboard')

  return (
    <Card className="relative flex min-h-[584px] flex-col overflow-hidden rounded-[10px] border border-hairline bg-surface p-8 shadow-none">
      <div aria-label={`${t('connection.ping')}: ${ping.value}`} className="absolute right-7 top-7 flex items-center gap-2" role="status"><span className="type-eyebrow !text-[9px] !tracking-[0.14em]">{t('connection.ping')}</span><ResultBadge tone={ping.tone} value={ping.value} /></div>
      <div><p className="type-eyebrow">{t('connection.activeVpn')}</p><h2 className="type-display mt-2 text-[22px] leading-tight tracking-[-0.01em] text-primary" id="connection-stage-title">{profileName}</h2><p className="mt-2 flex items-center gap-2 text-[14px] text-body"><MapPin aria-hidden="true" className="size-4 text-muted-copy" strokeWidth={1.7} />{location}</p></div>
      <div className="flex flex-1 flex-col items-center justify-center"><ConnectionDial connected={connected} onToggle={onToggleConnection} /><ModeSwitches systemProxy={systemProxy} tunMode={tunMode} onToggle={onToggleMode} /></div>
    </Card>
  )
}
