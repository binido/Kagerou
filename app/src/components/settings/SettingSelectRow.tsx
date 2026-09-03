import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

interface SettingSelectRowProps {
  id: string
  label: string
  value: string
  options: string[]
  onChange: (value: string) => void
}

export function SettingSelectRow({ id, label, value, options, onChange }: SettingSelectRowProps) {
  return (
    <div className="flex min-h-14 items-center justify-between gap-8 border-b border-white/[0.055]">
      <label className="text-[14px] leading-5 text-body" htmlFor={id}>{label}</label>
      <Select onValueChange={onChange} value={value}><SelectTrigger className="w-[148px] border-0 bg-surface text-[13px] text-body hover:bg-raised" id={id}><SelectValue /></SelectTrigger><SelectContent className="border-hairline bg-raised text-body">{options.map((option) => <SelectItem key={option} value={option}>{option}</SelectItem>)}</SelectContent></Select>
    </div>
  )
}
