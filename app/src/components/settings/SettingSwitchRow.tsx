import { Switch } from '@/components/ui/switch'

export function SettingSwitchRow({ label, checked, onChange }: { label: string; checked: boolean; onChange: (checked: boolean) => void }) {
  return (
    <div className="flex min-h-14 items-center justify-between gap-8 border-b border-white/[0.055]">
      <span className="text-[14px] leading-5 text-body">{label}</span>
      <Switch aria-label={label} checked={checked} className="data-checked:bg-lavender data-unchecked:bg-raised" onCheckedChange={onChange} />
    </div>
  )
}
