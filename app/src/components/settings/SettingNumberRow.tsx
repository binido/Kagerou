import { useState } from 'react'

import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

interface SettingNumberRowProps {
  id: string
  label: string
  description?: string
  value: number
  onChange: (value: number) => void
}

const isPositiveInteger = (value: string) => /^\d+$/.test(value) && Number(value) > 0

export function SettingNumberRow({ id, label, description, value, onChange }: SettingNumberRowProps) {
  const [rawValue, setRawValue] = useState(String(value))
  const [error, setError] = useState('')

  const handleChange = (nextValue: string) => {
    setRawValue(nextValue)
    if (isPositiveInteger(nextValue)) {
      setError('')
      onChange(Number(nextValue))
      return
    }
    setError('Enter a whole number greater than 0.')
  }

  return (
    <div className="flex min-h-14 items-start justify-between gap-8 border-b border-white/[0.055] py-3">
      <div className="min-w-0 pt-1">
        <Label className="text-[14px] leading-5 text-body" htmlFor={id}>{label}</Label>
        {description ? <p className="mt-1 text-[11px] leading-4 text-muted-copy">{description}</p> : null}
      </div>
      <div className="w-[148px] shrink-0">
        <Input aria-describedby={`${id}-description${error ? ` ${id}-error` : ''}`} aria-invalid={Boolean(error)} className="h-9 border-0 bg-surface text-right text-[13px] text-body" id={id} inputMode="numeric" min={1} onBlur={() => { if (!isPositiveInteger(rawValue)) setError('Enter a whole number greater than 0.') }} onChange={(event) => handleChange(event.target.value)} step={1} type="number" value={rawValue} />
        <p className="sr-only" id={`${id}-description`}>Minutes between automatic subscription updates.</p>
        {error ? <p className="mt-1 text-right text-[10px] leading-4 text-bad" id={`${id}-error`} role="alert">{error}</p> : null}
      </div>
    </div>
  )
}
