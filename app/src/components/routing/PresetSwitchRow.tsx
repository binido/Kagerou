import { useTranslation } from 'react-i18next'

import { Switch } from '@/components/ui/switch'
import type { RoutingPreset } from '@/types/kagerou'

type PresetLabelKey = 'presets.bypassLan.label' | 'presets.blockAds.label'
type PresetDescriptionKey = 'presets.bypassLan.description' | 'presets.blockAds.description'

const presetCopyKeys = (id: string): { label: PresetLabelKey; description: PresetDescriptionKey } => id === 'block-ads'
  ? { label: 'presets.blockAds.label', description: 'presets.blockAds.description' }
  : { label: 'presets.bypassLan.label', description: 'presets.bypassLan.description' }

export function PresetSwitchRow({ preset, onChange }: { preset: RoutingPreset; onChange: (enabled: boolean) => void }) {
  const { t } = useTranslation('routing')
  const { t: tc } = useTranslation('common')
  const copyKeys = presetCopyKeys(preset.id)
  const label = t(copyKeys.label)

  return (
    <div className="flex min-h-[62px] items-center justify-between gap-8 border-b border-hairline px-5 py-3 last:border-b-0">
      <div className="min-w-0"><p className="text-[13px] font-medium text-[#e3e0e9]">{label}</p><p className="mt-1 text-[11px] leading-4 text-quiet">{t(copyKeys.description)}</p></div>
      <div className="flex shrink-0 items-center gap-3"><span className={preset.enabled ? 'text-[11px] font-medium text-lavender-hi' : 'text-[11px] font-medium text-muted-copy'}>{preset.enabled ? tc('status.on') : tc('status.off')}</span><Switch aria-label={t('presets.toggle', { name: label })} checked={preset.enabled} className="data-checked:bg-lavender data-unchecked:bg-raised" onCheckedChange={onChange} /></div>
    </div>
  )
}
