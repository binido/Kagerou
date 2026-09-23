import { SettingHint } from '@/components/settings/SettingHint'
import { Switch } from '@/components/ui/switch'

interface SettingSwitchRowProps {
  label: string
  description?: string
  checked: boolean
  disabled?: boolean
  onChange: (checked: boolean) => void
}

export function SettingSwitchRow({
  label,
  description,
  checked,
  disabled = false,
  onChange,
}: SettingSwitchRowProps) {
  return (
    <div className="flex min-h-14 items-center justify-between gap-8 border-b border-hairline/55">
      <div className="flex min-w-0 items-center gap-1.5">
        <span className="text-[14px] leading-5 text-body">{label}</span>
        {description ? <SettingHint description={description} /> : null}
      </div>
      <Switch
        aria-label={label}
        checked={checked}
        className="shrink-0 data-checked:bg-lavender data-unchecked:bg-raised"
        disabled={disabled}
        onCheckedChange={onChange}
      />
    </div>
  )
}
