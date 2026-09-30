import { SettingHint } from '@/components/settings/SettingHint'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

export interface SettingSelectOption {
  value: string
  label: string
}

interface SettingSelectRowProps {
  id: string
  label: string
  description?: string
  value: string
  options: ReadonlyArray<string | SettingSelectOption>
  onChange: (value: string) => void
}

const toOption = (option: string | SettingSelectOption): SettingSelectOption =>
  typeof option === 'string' ? { value: option, label: option } : option

export function SettingSelectRow({
  id,
  label,
  description,
  value,
  options,
  onChange,
}: SettingSelectRowProps) {
  return (
    <div className="flex min-h-14 items-center justify-between gap-8 border-b border-hairline/55">
      <div className="flex min-w-0 items-center gap-1.5">
        <Label className="text-[14px] leading-5 text-body" htmlFor={id}>
          {label}
        </Label>
        {description ? <SettingHint description={description} /> : null}
      </div>
      <Select onValueChange={onChange} value={value}>
        <SelectTrigger
          className="w-[148px] border-0 bg-surface text-[13px] text-body hover:bg-raised"
          id={id}
        >
          <SelectValue />
        </SelectTrigger>
        <SelectContent className="border-hairline bg-raised text-body">
          {options.map((option) => {
            const item = toOption(option)
            return (
              <SelectItem key={item.value} value={item.value}>
                {item.label}
              </SelectItem>
            )
          })}
        </SelectContent>
      </Select>
    </div>
  )
}
