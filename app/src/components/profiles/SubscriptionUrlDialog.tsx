import { useState, type FormEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Field, FieldError, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import type { Source } from '@/types/kagerou'

interface SubscriptionUrlDialogProps {
  source: Source | null
  onOpenChange: (open: boolean) => void
  /** Resolves to false when the backend refused the URL. */
  onSubmit: (url: string) => Promise<boolean>
}

export function SubscriptionUrlDialog({ source, onOpenChange, onSubmit }: SubscriptionUrlDialogProps) {
  const { t } = useTranslation('profiles')
  const [value, setValue] = useState('')
  const [error, setError] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [prevSource, setPrevSource] = useState<Source | null>(null)

  // The field starts empty rather than showing the current URL: its token is
  // a credential, and it is never put on screen.
  if (source !== prevSource) {
    setPrevSource(source)
    if (source) {
      setValue('')
      setError('')
      setSubmitting(false)
    }
  }

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    setSubmitting(true)
    const saved = await onSubmit(value.trim())
    setSubmitting(false)
    if (!saved) setError(t('dialogs.subscriptionUrl.invalid'))
  }

  return (
    <Dialog onOpenChange={(open) => { if (!submitting) onOpenChange(open) }} open={Boolean(source)}>
      <DialogContent className="border-hairline bg-raised text-primary sm:max-w-[480px]">
        <DialogHeader>
          <DialogTitle className="type-display text-2xl text-primary">{t('dialogs.subscriptionUrl.title')}</DialogTitle>
          <DialogDescription className="text-[12px] leading-5 text-muted-copy">{t('dialogs.subscriptionUrl.description')}</DialogDescription>
        </DialogHeader>
        <form className="space-y-5" onSubmit={(event) => { void handleSubmit(event) }}>
          <Field>
            <FieldLabel className="text-[12px] text-primary" htmlFor="subscription-url">{t('dialogs.subscriptionUrl.label')}</FieldLabel>
            <Input aria-describedby={error ? 'subscription-url-error' : undefined} aria-invalid={Boolean(error)} autoComplete="off" autoFocus className="h-[42px] border-hairline bg-surface font-mono text-[12px]" disabled={submitting} id="subscription-url" inputMode="url" onChange={(event) => { setValue(event.target.value); setError('') }} placeholder={t('dialogs.subscriptionUrl.placeholder')} spellCheck={false} type="url" value={value} />
            {error ? <FieldError className="text-[11px]" id="subscription-url-error">{error}</FieldError> : null}
          </Field>
          <DialogFooter>
            <Button disabled={submitting} onClick={() => onOpenChange(false)} type="button" variant="ghost">{t('dialogs.subscriptionUrl.cancel')}</Button>
            <Button className="gap-1.5 bg-lavender text-ink hover:bg-lavender-hi" disabled={submitting || !value.trim()} type="submit">{submitting ? <Loader2 aria-hidden="true" className="size-3.5 animate-spin" /> : null}{t('dialogs.subscriptionUrl.submit')}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
