import { useState, type FormEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { Check } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Field, FieldDescription, FieldError, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import type { ProfileGroup } from '@/types/kagerou'

interface ProfileGroupDialogProps {
  open: boolean
  group?: ProfileGroup | null
  onOpenChange: (open: boolean) => void
  onSubmit: (label: string) => boolean | Promise<boolean>
}

export function ProfileGroupDialog({ open, group, onOpenChange, onSubmit }: ProfileGroupDialogProps) {
  const { t } = useTranslation('profiles')
  const [label, setLabel] = useState('')
  const [error, setError] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [prevOpen, setPrevOpen] = useState(open)
  const isRename = Boolean(group)

  // reset when open flips: the parent batches group with open, so the dialog never paints the previous target's values
  if (open !== prevOpen) {
    setPrevOpen(open)
    if (open) {
      setLabel(group?.label ?? '')
      setError('')
      setSubmitting(false)
    }
  }

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const trimmed = label.trim().replace(/\s+/g, ' ')
    if (!trimmed) {
      setError(t('dialogs.group.empty'))
      return
    }
    setSubmitting(true)
    const ok = await onSubmit(trimmed)
    setSubmitting(false)
    if (!ok) {
      setError(t('dialogs.group.duplicate'))
      return
    }
    setError('')
    onOpenChange(false)
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent className="border-hairline bg-raised text-primary sm:max-w-[440px]">
        <DialogHeader>
          <p className="type-eyebrow">{t('dialogs.group.eyebrow')}</p>
          <DialogTitle className="type-display mt-2 text-2xl text-primary">{isRename ? t('dialogs.group.renameTitle') : t('dialogs.group.newTitle')}</DialogTitle>
          <DialogDescription className="text-[12px] leading-5 text-muted-copy">
            {isRename ? t('dialogs.group.renameDescription') : t('dialogs.group.newDescription')}
          </DialogDescription>
        </DialogHeader>
        <form className="space-y-5" onSubmit={handleSubmit}>
          <Field>
            <FieldLabel className="text-[12px] text-primary" htmlFor="profile-group-name">{t('dialogs.group.nameLabel')}</FieldLabel>
            <Input aria-describedby="profile-group-helper" autoFocus className="h-[42px] border-hairline bg-surface text-[13px]" id="profile-group-name" onChange={(event) => { setLabel(event.target.value); setError('') }} placeholder={t('dialogs.group.placeholder')} value={label} />
            <FieldDescription className="text-[11px] leading-4 text-muted-copy" id="profile-group-helper">{t('dialogs.group.helper')}</FieldDescription>
          </Field>
          {error ? <FieldError className="text-[11px]">{error}</FieldError> : null}
          <DialogFooter>
            <Button onClick={() => onOpenChange(false)} type="button" variant="ghost">{t('dialogs.group.cancel')}</Button>
            <Button className="gap-1.5 bg-lavender text-ink hover:bg-lavender-hi" disabled={submitting} type="submit"><Check aria-hidden="true" className="size-3.5" />{isRename ? t('dialogs.group.save') : t('dialogs.group.create')}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
