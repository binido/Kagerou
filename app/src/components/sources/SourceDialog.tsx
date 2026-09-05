import { useState, type FormEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { Info, KeyRound, Loader2, Rss } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Field, FieldDescription, FieldError, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { kagerouApi } from '@/lib/tauri-api'
import type { AddSourceInput, Source, SourceType } from '@/types/kagerou'

interface SourceDialogProps {
  open: boolean
  initialType: SourceType
  source?: Source | null
  onOpenChange: (open: boolean) => void
  onSubmit: (input: AddSourceInput) => void | Promise<void>
}

export function SourceDialog({ open, initialType, source, onOpenChange, onSubmit }: SourceDialogProps) {
  const { t } = useTranslation('sources')
  const [type, setType] = useState<SourceType>(initialType)
  const [name, setName] = useState('')
  const [value, setValue] = useState('')
  const [error, setError] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [prevOpen, setPrevOpen] = useState(open)
  // Snapshot rather than read live: the parent clears `source` on close, and
  // the dialog stays mounted through its exit animation. Deriving from the
  // prop would retitle it "add" for those 100ms on the way out.
  const [editing, setEditing] = useState(Boolean(source))

  // reset when open flips: the parent batches source/initialType with open, so the dialog never paints the previous target's values
  if (open !== prevOpen) {
    setPrevOpen(open)
    if (open) {
      setEditing(Boolean(source))
      setType(source?.type ?? initialType)
      setName(source?.name ?? '')
      setValue(source?.type === 'url' ? '' : source?.value ?? '')
      setError('')
      setSubmitting(false)
    }
  }

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const trimmedValue = value.trim()
    const nextValue = editing && type === 'url' && !trimmedValue ? source?.value ?? '' : trimmedValue
    setSubmitting(true)
    const validationError = await kagerouApi.validateSource(type, nextValue)
    if (validationError) {
      setError(validationError === 'invalidUrl' ? t('feedback.invalidUrl') : t('feedback.invalidKey'))
      setSubmitting(false)
      return
    }

    try {
      await onSubmit({ type, name: name.trim() || undefined, value: nextValue })
      setError('')
      onOpenChange(false)
    } catch (submitError) {
      setError(submitError instanceof Error ? submitError.message : t('feedback.importError'))
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <Dialog onOpenChange={(nextOpen) => { if (!submitting) onOpenChange(nextOpen) }} open={open}>
      <DialogContent className="border-hairline bg-raised text-primary sm:max-w-[500px]">
        <DialogHeader>
          <p className="type-eyebrow">{t('dialog.eyebrow')}</p>
          <DialogTitle className="type-display mt-2 text-2xl text-primary">{editing ? t('dialog.editTitle') : t('dialog.addTitle')}</DialogTitle>
          <DialogDescription className="text-[12px] leading-5 text-muted-copy">{editing ? t('dialog.editDescription') : t('dialog.addDescription')}</DialogDescription>
        </DialogHeader>
        <ToggleGroup aria-label={t('dialog.type')} className="grid w-full grid-cols-2 gap-1 rounded-lg bg-canvas p-1" disabled={editing || submitting} onValueChange={(next) => { if (next) { setType(next as SourceType); setError('') } }} type="single" value={type}>
          <ToggleGroupItem className="h-10 gap-2 rounded-md text-[12px] text-body data-[state=on]:bg-selected data-[state=on]:text-primary" value="url"><Rss aria-hidden="true" className="size-4" />{t('dialog.subscriptionUrl')}</ToggleGroupItem>
          <ToggleGroupItem className="h-10 gap-2 rounded-md text-[12px] text-body data-[state=on]:bg-selected data-[state=on]:text-primary" value="key"><KeyRound aria-hidden="true" className="size-4" />{t('dialog.singleKey')}</ToggleGroupItem>
        </ToggleGroup>
        {editing ? <p className="text-[11px] text-muted-copy">{t('dialog.typeLocked')}</p> : null}
        <form className="space-y-4" onSubmit={handleSubmit}>
          <Field>
            <FieldLabel className="text-[13px] text-primary" htmlFor="source-name">{type === 'url' ? t('dialog.subscriptionName') : t('dialog.vpnName')}</FieldLabel>
            <Input autoComplete="off" className="h-11 border-hairline/55 bg-surface text-[13px]" disabled={submitting} id="source-name" onChange={(event) => setName(event.target.value)} placeholder={type === 'url' ? t('dialog.subscriptionNamePlaceholder') : t('dialog.vpnNamePlaceholder')} value={name} />
            <FieldDescription className="text-[11px] leading-4 text-muted-copy">{type === 'url' ? t('dialog.urlNameHelp') : t('dialog.keyNameHelp')}</FieldDescription>
          </Field>
          <Field>
            <FieldLabel className="text-[13px] text-primary" htmlFor="source-value">{type === 'url' ? editing ? t('dialog.replaceSourceUrl') : t('dialog.sourceUrl') : t('dialog.vpnKey')}</FieldLabel>
            <Input aria-describedby="source-helper" autoComplete="off" autoFocus={!name} className="h-11 border-hairline/55 bg-surface text-[13px] font-mono" disabled={submitting} id="source-value" onChange={(event) => setValue(event.target.value)} placeholder={type === 'url' ? editing ? t('dialog.replaceUrlPlaceholder') : t('dialog.urlPlaceholder') : t('dialog.keyPlaceholder')} value={value} />
            <FieldDescription className="flex items-center gap-1.5 text-[11px] leading-4 text-muted-copy" id="source-helper"><Info aria-hidden="true" className="size-3.5" />{type === 'url' ? editing ? t('dialog.urlReplaceHelp') : t('dialog.urlValueHelp') : t('dialog.keyValueHelp')}</FieldDescription>
          </Field>
          {error ? <FieldError className="text-[11px]">{error}</FieldError> : null}
          <DialogFooter>
            <Button disabled={submitting} onClick={() => onOpenChange(false)} type="button" variant="ghost">{t('dialog.cancel')}</Button>
            <Button className="gap-1.5 bg-lavender text-ink hover:bg-lavender-hi" disabled={submitting} type="submit">{submitting ? <Loader2 aria-hidden="true" className="size-3.5 animate-spin" /> : null}{editing ? t('dialog.save') : type === 'url' ? t('dialog.addSubscription') : t('dialog.addKey')}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
