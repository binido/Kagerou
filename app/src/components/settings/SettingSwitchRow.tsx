import { useId } from 'react'

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
  const switchId = useId()

  return (
    <div className="flex min-h-14 items-center justify-between gap-8 border-b border-hairline/55">
      <div className="flex min-w-0 items-center gap-1.5">
        <label className="cursor-pointer text-[14px] leading-5 text-body" htmlFor={switchId}>
          {label}
        </label>
        {description ? <SettingHint description={description} /> : null}
      </div>
      <Switch
        checked={checked}
        className="shrink-0 data-checked:bg-lavender data-unchecked:bg-raised"
        disabled={disabled}
        id={switchId}
        onCheckedChange={onChange}
      />
    </div>
  )
}
