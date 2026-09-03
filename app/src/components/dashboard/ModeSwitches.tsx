import { useTranslation } from 'react-i18next'

import { Switch } from '@/components/ui/switch'

interface ModeSwitchesProps {
  tunMode: boolean
  systemProxy: boolean
  onToggle: (mode: 'tun' | 'proxy') => void
}

export function ModeSwitches({ tunMode, systemProxy, onToggle }: ModeSwitchesProps) {
  const { t } = useTranslation('dashboard')

  return (
    <div className="mt-10 flex items-center rounded-lg border border-hairline bg-canvas p-1">
      <label className="flex h-10 cursor-pointer items-center gap-3 rounded-md px-3 text-[13px] text-body transition-colors hover:bg-selected">
        <span>{t('modes.tun')}</span>
        <Switch aria-label={t('modes.tun')} checked={tunMode} onCheckedChange={() => onToggle('tun')} />
      </label>
      <span aria-hidden="true" className="my-2 h-6 w-px bg-hairline" />
      <label className="flex h-10 cursor-pointer items-center gap-3 rounded-md px-3 text-[13px] text-body transition-colors hover:bg-selected">
        <span>{t('modes.systemProxy')}</span>
        <Switch aria-label={t('modes.systemProxy')} checked={systemProxy} onCheckedChange={() => onToggle('proxy')} />
      </label>
    </div>
  )
}
