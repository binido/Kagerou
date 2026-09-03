import { useState, type FormEvent } from 'react'
import { Info, KeyRound, Loader2, Rss } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Field, FieldDescription, FieldError, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { mockApi } from '@/lib/mock-api'
import type { AddSourceInput, Source, SourceType } from '@/types/kagerou'

interface SourceDialogProps {
  open: boolean
  initialType: SourceType
  source?: Source | null
  onOpenChange: (open: boolean) => void
  onSubmit: (input: AddSourceInput) => void | Promise<void>
}

export function SourceDialog({ open, initialType, source, onOpenChange, onSubmit }: SourceDialogProps) {
  const [type, setType] = useState<SourceType>(source?.type ?? initialType)
  const [name, setName] = useState(source?.name ?? '')
  const [value, setValue] = useState(source?.value ?? '')
  const [error, setError] = useState('')
  const [submitting, setSubmitting] = useState(false)

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const trimmedValue = value.trim()
    const validationError = mockApi.validateSource(type, trimmedValue)
    if (validationError) {
      setError(validationError)
      return
    }

    setSubmitting(true)
    try {
      await onSubmit({ type, name: name.trim() || undefined, value: trimmedValue })
      setError('')
      onOpenChange(false)
    } catch (submitError) {
      setError(submitError instanceof Error ? submitError.message : 'The source could not be imported.')
    } finally {
      setSubmitting(false)
    }
  }

  const editing = Boolean(source)

  return (
    <Dialog onOpenChange={(nextOpen) => { if (!submitting) onOpenChange(nextOpen) }} open={open}>
      <DialogContent className="border-hairline bg-raised text-primary sm:max-w-[500px]">
        <DialogHeader>
          <p className="type-eyebrow">VPN source</p>
          <DialogTitle className="type-display mt-2 text-2xl text-primary">{editing ? 'Edit source' : 'Add source'}</DialogTitle>
          <DialogDescription className="text-[12px] leading-5 text-muted-copy">{editing ? 'Update the source label or its value. Subscription VPNs stay in the same locked group.' : 'Add a subscription or a single key. Subscriptions receive their own managed group.'}</DialogDescription>
        </DialogHeader>
        <ToggleGroup aria-label="Source type" className="grid w-full grid-cols-2 gap-1 rounded-lg bg-canvas p-1" disabled={editing || submitting} onValueChange={(next) => { if (next) { setType(next as SourceType); setError('') } }} type="single" value={type}>
          <ToggleGroupItem className="h-10 gap-2 rounded-md text-[12px] text-body data-[state=on]:bg-selected data-[state=on]:text-primary" value="url"><Rss aria-hidden="true" className="size-4" />Subscription URL</ToggleGroupItem>
          <ToggleGroupItem className="h-10 gap-2 rounded-md text-[12px] text-body data-[state=on]:bg-selected data-[state=on]:text-primary" value="key"><KeyRound aria-hidden="true" className="size-4" />Single key</ToggleGroupItem>
        </ToggleGroup>
        {editing ? <p className="text-[11px] text-muted-copy">Source type cannot be changed while editing.</p> : null}
        <form className="space-y-4" onSubmit={handleSubmit}>
          <Field>
            <FieldLabel className="text-[13px] text-primary" htmlFor="source-name">{type === 'url' ? 'Subscription name (optional)' : 'VPN name (optional)'}</FieldLabel>
            <Input autoComplete="off" className="h-11 border-white/10 bg-surface text-[13px]" disabled={submitting} id="source-name" onChange={(event) => setName(event.target.value)} placeholder={type === 'url' ? 'e.g. Personal / North America' : 'e.g. Emergency access'} value={name} />
            <FieldDescription className="text-[11px] leading-4 text-muted-copy">{type === 'url' ? 'Leave blank to use the subscription URL hostname.' : 'Leave blank to use the key scheme as the label.'}</FieldDescription>
          </Field>
          <Field>
            <FieldLabel className="text-[13px] text-primary" htmlFor="source-value">{type === 'url' ? 'Source URL' : 'VPN key'}</FieldLabel>
            <Input aria-describedby="source-helper" autoFocus={!name} className="h-11 border-white/10 bg-surface text-[13px] font-mono" disabled={submitting} id="source-value" onChange={(event) => setValue(event.target.value)} placeholder={type === 'url' ? 'https://example.com/vpn-list' : 'vless://your-key'} value={value} />
            <FieldDescription className="flex items-center gap-1.5 text-[11px] leading-4 text-muted-copy" id="source-helper"><Info aria-hidden="true" className="size-3.5" />{type === 'url' ? 'All imported VPNs stay together in the subscription group.' : 'Creates one local VPN in Default. It can be moved later.'}</FieldDescription>
          </Field>
          {error ? <FieldError className="text-[11px]">{error}</FieldError> : null}
          <DialogFooter>
            <Button disabled={submitting} onClick={() => onOpenChange(false)} type="button" variant="ghost">Cancel</Button>
            <Button className="gap-1.5 bg-lavender text-ink hover:bg-lavender-hi" disabled={submitting} type="submit">{submitting ? <Loader2 aria-hidden="true" className="size-3.5 animate-spin" /> : null}{editing ? 'Save changes' : type === 'url' ? 'Add subscription' : 'Add key'}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
