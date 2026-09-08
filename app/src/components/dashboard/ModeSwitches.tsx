import { Globe, Network } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Card } from '@/components/ui/card'
import { Switch } from '@/components/ui/switch'

interface ModeSwitchesProps {
  tunMode: boolean
  systemProxy: boolean
  onToggle: (mode: 'tun' | 'proxy') => void
}

export function ModeSwitches({ tunMode, systemProxy, onToggle }: ModeSwitchesProps) {
  const { t } = useTranslation('dashboard')

  return (
    <Card className="flex h-full flex-col gap-3 rounded-[10px] border border-hairline bg-surface p-5 shadow-none">
      <p className="type-eyebrow">{t('modes.title')}</p>
      <label className="flex cursor-pointer items-center justify-between gap-3 text-[13px] text-body">
        <span className="flex min-w-0 items-center gap-2">
          <Network aria-hidden="true" className="size-4 shrink-0 text-muted-copy" strokeWidth={1.7} />
          <span className="truncate">{t('modes.tun')}</span>
        </span>
        <Switch aria-label={t('modes.tun')} checked={tunMode} onCheckedChange={() => onToggle('tun')} />
      </label>
      <label className="flex cursor-pointer items-center justify-between gap-3 text-[13px] text-body">
        <span className="flex min-w-0 items-center gap-2">
          <Globe aria-hidden="true" className="size-4 shrink-0 text-muted-copy" strokeWidth={1.7} />
          <span className="truncate">{t('modes.systemProxy')}</span>
        </span>
        <Switch aria-label={t('modes.systemProxy')} checked={systemProxy} onCheckedChange={() => onToggle('proxy')} />
      </label>
      <p className="type-meta">{t('modes.hint')}</p>
    </Card>
  )
}
