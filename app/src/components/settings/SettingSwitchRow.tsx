import { Switch } from '@/components/ui/switch'

interface SettingSwitchRowProps {
  label: string
  description?: string
  checked: boolean
  onChange: (checked: boolean) => void
}

export function SettingSwitchRow({ label, description, checked, onChange }: SettingSwitchRowProps) {
  return (
    <div className="flex min-h-14 items-center justify-between gap-8 border-b border-hairline/55">
      <div className="min-w-0">
        <span className="block text-[14px] leading-5 text-body">{label}</span>
        {description ? <p className="mt-1 text-[11px] leading-4 text-muted-copy">{description}</p> : null}
      </div>
      <Switch aria-label={label} checked={checked} className="shrink-0 data-checked:bg-lavender data-unchecked:bg-raised" onCheckedChange={onChange} />
    </div>
  )
}
