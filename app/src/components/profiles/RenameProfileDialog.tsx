import { useState, type FormEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { Check } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Field, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import type { Profile } from '@/types/kagerou'

interface RenameProfileDialogProps {
  profile: Profile | null
  onOpenChange: (open: boolean) => void
  /** Resolves to the trimmed name, or `null` when it was blank or refused. */
  onSubmit: (name: string) => Promise<string | null>
}

export function RenameProfileDialog({ profile, onOpenChange, onSubmit }: RenameProfileDialogProps) {
  const { t } = useTranslation('profiles')
  const [value, setValue] = useState('')
  const [prevProfile, setPrevProfile] = useState(profile)

  // Reset on the way in, not in an effect: the parent clears `profile` on
  // close and the dialog stays mounted through its exit animation, so an
  // effect would blank the field while it is still on screen.
  if (profile !== prevProfile) {
    setPrevProfile(profile)
    if (profile) setValue(profile.name)
  }

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    await onSubmit(value.trim())
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={Boolean(profile)}>
      <DialogContent className="border-hairline bg-raised text-primary sm:max-w-[420px]">
        <DialogHeader>
          <DialogTitle className="type-display text-2xl text-primary">{t('dialogs.rename.title')}</DialogTitle>
          <DialogDescription className="text-[12px] text-muted-copy">{t('dialogs.rename.description')}</DialogDescription>
        </DialogHeader>
        <form className="space-y-5" onSubmit={submit}>
          <Field>
            <FieldLabel className="text-[12px] text-primary" htmlFor="rename-profile">
              {t('dialogs.rename.nameLabel')}
            </FieldLabel>
            <Input
              autoFocus
              className="h-[42px] border-hairline bg-surface text-[13px]"
              id="rename-profile"
              onChange={(event) => setValue(event.target.value)}
              value={value}
            />
          </Field>
          <DialogFooter>
            <Button onClick={() => onOpenChange(false)} type="button" variant="ghost">{t('dialogs.rename.cancel')}</Button>
            <Button className="bg-lavender text-ink hover:bg-lavender-hi" type="submit">
              <Check aria-hidden="true" className="size-3.5" />
              {t('dialogs.rename.submit')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
