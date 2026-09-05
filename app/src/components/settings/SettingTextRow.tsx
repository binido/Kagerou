import { useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

interface SettingTextRowProps {
  id: string
  label: string
  description?: string
  value: string
  onChange: (value: string) => void
}

export function SettingTextRow({ id, label, description, value, onChange }: SettingTextRowProps) {
  const { t } = useTranslation('settings')
  const [rawValue, setRawValue] = useState(value)
  const [error, setError] = useState('')

  const handleChange = (nextValue: string) => {
    setRawValue(nextValue)
    if (nextValue.trim()) {
      setError('')
      onChange(nextValue.trim())
      return
    }
    setError(t('validation.nonEmpty'))
  }

  return (
    <div className="flex min-h-14 items-start justify-between gap-8 border-b border-hairline/55 py-3">
      <div className="min-w-0 pt-1">
        <Label className="text-[14px] leading-5 text-body" htmlFor={id}>{label}</Label>
        {description ? <p className="mt-1 text-[11px] leading-4 text-muted-copy" id={`${id}-description`}>{description}</p> : null}
      </div>
      <div className="w-[240px] shrink-0">
        <Input aria-describedby={`${id}-description${error ? ` ${id}-error` : ''}`} aria-invalid={Boolean(error)} className="h-9 border-0 bg-surface text-left text-[13px] text-body" id={id} onBlur={() => { if (!rawValue.trim()) setError(t('validation.nonEmpty')) }} onChange={(event) => handleChange(event.target.value)} type="text" value={rawValue} />
        {error ? <p className="mt-1 text-right text-[10px] leading-4 text-bad" id={`${id}-error`} role="alert">{error}</p> : null}
      </div>
    </div>
  )
}
