import { Switch } from '@/components/ui/switch'
import type { RoutingPreset } from '@/types/kagerou'

export function PresetSwitchRow({ preset, onChange }: { preset: RoutingPreset; onChange: (enabled: boolean) => void }) {
  return (
    <div className="flex min-h-[62px] items-center justify-between gap-8 border-b border-white/[0.055] px-5 py-3 last:border-b-0">
      <div className="min-w-0"><p className="text-[13px] font-medium text-[#e3e0e9]">{preset.label}</p><p className="mt-1 text-[11px] leading-4 text-quiet">{preset.description}</p></div>
      <div className="flex shrink-0 items-center gap-3"><span className={preset.enabled ? 'text-[11px] font-medium text-lavender-hi' : 'text-[11px] font-medium text-muted-copy'}>{preset.enabled ? 'On' : 'Off'}</span><Switch aria-label={`Toggle ${preset.label}`} checked={preset.enabled} className="data-checked:bg-lavender data-unchecked:bg-raised" onCheckedChange={onChange} /></div>
    </div>
  )
}
