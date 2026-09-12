import { useState, type FormEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Field, FieldError, FieldLabel } from '@/components/ui/field'
import { Textarea } from '@/components/ui/textarea'

/** What an import that needs a human leaves behind: the text it was given
 * and, unless there was simply nothing on the clipboard, why it failed. */
export interface ImportDraft {
  text: string
  error: string
}

interface ImportDialogProps {
  draft: ImportDraft | null
  submitting: boolean
  onOpenChange: (open: boolean) => void
  onSubmit: (text: string) => void
}

export function ImportDialog({ draft, submitting, onOpenChange, onSubmit }: ImportDialogProps) {
  const { t } = useTranslation('profiles')
  const [text, setText] = useState('')
  const [prevDraft, setPrevDraft] = useState<ImportDraft | null>(null)

  // A new draft replaces the field: it is either a fresh clipboard read or
  // the same text coming back with a new error.
  if (draft !== prevDraft) {
    setPrevDraft(draft)
    if (draft) setText(draft.text)
  }

  const clipboardWasEmpty = Boolean(draft && !draft.text && !draft.error)

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (text.trim()) onSubmit(text)
  }

  return (
    <Dialog onOpenChange={(open) => { if (!submitting) onOpenChange(open) }} open={Boolean(draft)}>
      <DialogContent className="border-hairline bg-raised text-primary sm:max-w-[520px]">
        <DialogHeader>
          <DialogTitle className="type-display text-2xl text-primary">{t('dialogs.import.title')}</DialogTitle>
          <DialogDescription className="text-[12px] leading-5 text-muted-copy">{clipboardWasEmpty ? t('dialogs.import.emptyClipboard') : t('dialogs.import.description')}</DialogDescription>
        </DialogHeader>
        <form className="space-y-5" onSubmit={handleSubmit}>
          <Field>
            <FieldLabel className="text-[12px] text-primary" htmlFor="import-text">{t('dialogs.import.label')}</FieldLabel>
            <Textarea aria-describedby={draft?.error ? 'import-error' : undefined} aria-invalid={Boolean(draft?.error)} autoFocus className="min-h-[120px] resize-y border-hairline bg-surface font-mono text-[11px]" disabled={submitting} id="import-text" onChange={(event) => setText(event.target.value)} placeholder={t('dialogs.import.placeholder')} spellCheck={false} value={text} />
            {draft?.error ? <FieldError className="text-[11px]" id="import-error">{draft.error}</FieldError> : null}
          </Field>
          <DialogFooter>
            <Button disabled={submitting} onClick={() => onOpenChange(false)} type="button" variant="ghost">{t('dialogs.import.cancel')}</Button>
            <Button className="gap-1.5 bg-lavender text-ink hover:bg-lavender-hi" disabled={submitting || !text.trim()} type="submit">{submitting ? <Loader2 aria-hidden="true" className="size-3.5 animate-spin" /> : null}{t('dialogs.import.submit')}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
