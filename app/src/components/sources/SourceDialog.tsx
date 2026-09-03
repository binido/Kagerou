import { useState, type FormEvent } from 'react'
import { Info, KeyRound, Rss } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Field, FieldDescription, FieldError, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { mockApi } from '@/lib/mock-api'
import type { Source, SourceType } from '@/types/kagerou'

interface SourceDialogProps {
  open: boolean
  initialType: SourceType
  source?: Source | null
  onOpenChange: (open: boolean) => void
  onSubmit: (type: SourceType, value: string) => void
}

export function SourceDialog({ open, initialType, source, onOpenChange, onSubmit }: SourceDialogProps) {
  const [type, setType] = useState<SourceType>(source?.type ?? initialType)
  const [value, setValue] = useState(source?.value ?? '')
  const [error, setError] = useState('')

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const trimmed = value.trim()
    const validationError = mockApi.validateSource(type, trimmed)
    if (validationError) {
      setError(validationError)
      return
    }
    onSubmit(type, trimmed)
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent className="border-hairline bg-raised text-primary sm:max-w-[500px]">
        <DialogHeader>
          <p className="type-eyebrow">Profile source</p>
          <DialogTitle className="type-display mt-2 text-2xl text-primary">{source ? 'Edit source' : 'Add source'}</DialogTitle>
          <DialogDescription className="sr-only">{source ? 'Edit a profile source.' : 'Add a subscription URL or single key source.'}</DialogDescription>
        </DialogHeader>
        <ToggleGroup aria-label="Source type" className="grid w-full grid-cols-2 gap-1 rounded-lg bg-canvas p-1" onValueChange={(next) => { if (next) { setType(next as SourceType); setError('') } }} type="single" value={type}>
          <ToggleGroupItem className="h-10 gap-2 rounded-md text-[12px] text-body data-[state=on]:bg-selected data-[state=on]:text-primary" value="url"><Rss aria-hidden="true" className="size-4" />Subscription URL</ToggleGroupItem>
          <ToggleGroupItem className="h-10 gap-2 rounded-md text-[12px] text-body data-[state=on]:bg-selected data-[state=on]:text-primary" value="key"><KeyRound aria-hidden="true" className="size-4" />Single key</ToggleGroupItem>
        </ToggleGroup>
        <form className="space-y-4" onSubmit={handleSubmit}>
          <Field>
            <FieldLabel className="text-[13px] text-primary" htmlFor="source-value">{type === 'url' ? 'Source URL' : 'Profile key'}</FieldLabel>
            <Input aria-describedby="source-helper" autoFocus className="h-11 border-white/10 bg-surface text-[13px] font-mono" id="source-value" onChange={(event) => setValue(event.target.value)} placeholder={type === 'url' ? 'https://example.com/profiles' : 'vless://your-key'} value={value} />
            <FieldDescription className="flex items-center gap-1.5 text-[11px] leading-4 text-muted-copy" id="source-helper"><Info aria-hidden="true" className="size-3.5" />{type === 'url' ? 'Imports profiles into a group and can be refreshed later.' : 'Creates one profile. The key stays managed locally.'}</FieldDescription>
          </Field>
          {error ? <FieldError className="text-[11px]">{error}</FieldError> : null}
          <DialogFooter>
            <Button onClick={() => onOpenChange(false)} type="button" variant="ghost">Cancel</Button>
            <Button className="bg-lavender text-ink hover:bg-lavender-hi" type="submit">{source ? 'Save changes' : 'Add source'}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
