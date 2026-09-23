import { useId } from 'react'
import { useTranslation } from 'react-i18next'

import { Switch } from '@/components/ui/switch'
import type { RoutingPreset } from '@/types/kagerou'

type PresetLabelKey = 'presets.bypassLan.label' | 'presets.blockAds.label'
type PresetDescriptionKey = 'presets.bypassLan.description' | 'presets.blockAds.description'

// Only the presets seeded by migration 0001 are translated, any other one shows the backend copy.
const presetCopyKeys: Record<
  string,
  { label: PresetLabelKey; description: PresetDescriptionKey } | undefined
> = {
  'bypass-lan': { label: 'presets.bypassLan.label', description: 'presets.bypassLan.description' },
  'block-ads': { label: 'presets.blockAds.label', description: 'presets.blockAds.description' },
}

export function PresetSwitchRow({
  preset,
  onChange,
}: {
  preset: RoutingPreset
  onChange: (enabled: boolean) => void
}) {
  const { t } = useTranslation('routing')
  const { t: tc } = useTranslation('common')
  const switchId = useId()
  const copyKeys = presetCopyKeys[preset.id]
  const label = copyKeys ? t(copyKeys.label) : preset.label
  const description = copyKeys ? t(copyKeys.description) : preset.description

  return (
    <div className="flex min-h-[62px] items-center justify-between gap-8 border-b border-hairline px-5 py-3 last:border-b-0">
      <div className="min-w-0">
        <label
          className="block cursor-pointer text-[13px] font-medium text-primary"
          htmlFor={switchId}
        >
          {label}
        </label>
        <p className="mt-1 text-[11px] leading-4 text-quiet" id={`${switchId}-description`}>
          {description}
        </p>
      </div>
      <div className="flex shrink-0 items-center gap-3">
        <span
          className={
            preset.enabled
              ? 'text-[11px] font-medium text-lavender-hi'
              : 'text-[11px] font-medium text-muted-copy'
          }
        >
          {preset.enabled ? tc('status.on') : tc('status.off')}
        </span>
        <Switch
          aria-describedby={`${switchId}-description`}
          checked={preset.enabled}
          className="data-checked:bg-lavender data-unchecked:bg-raised"
          id={switchId}
          onCheckedChange={onChange}
        />
      </div>
    </div>
  )
}
