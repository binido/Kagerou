import { MapPin } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { ConnectionDial } from '@/components/dashboard/ConnectionDial'
import { ModeSwitches } from '@/components/dashboard/ModeSwitches'

interface ConnectionStageProps {
  profileName: string
  location: string
  connected: boolean
  tunMode: boolean
  systemProxy: boolean
  onToggleConnection: () => void
  onDisconnect: () => void
  onToggleMode: (mode: 'tun' | 'proxy') => void
}

export function ConnectionStage({ profileName, location, connected, tunMode, systemProxy, onToggleConnection, onDisconnect, onToggleMode }: ConnectionStageProps) {
  return (
    <Card className="relative flex min-h-[584px] flex-col overflow-hidden rounded-[10px] border border-hairline bg-surface p-8 shadow-none">
      <div aria-hidden="true" className="pointer-events-none absolute right-7 top-7 flex items-center gap-2"><span className="h-px w-7 bg-hairline" /><span className="size-1.5 rounded-full bg-lavender" /></div>
      <div><p className="type-eyebrow">Active profile</p><h2 className="type-display mt-2 text-[22px] leading-tight tracking-[-0.01em] text-primary" id="connection-stage-title">{profileName}</h2><p className="mt-2 flex items-center gap-2 text-[14px] text-body"><MapPin aria-hidden="true" className="size-4 text-muted-copy" strokeWidth={1.7} />{location}</p></div>
      <div className="flex flex-1 flex-col items-center justify-center"><ConnectionDial connected={connected} onToggle={onToggleConnection} /><Button className="mt-6 text-[12px] text-lavender underline decoration-lavender/50 underline-offset-4 hover:bg-transparent hover:text-lavender-hi" onClick={onDisconnect} type="button" variant="ghost">{connected ? 'Disconnect' : 'Connect'}</Button><ModeSwitches systemProxy={systemProxy} tunMode={tunMode} onToggle={onToggleMode} /></div>
    </Card>
  )
}
