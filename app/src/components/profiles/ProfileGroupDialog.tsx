import { useState, type FormEvent } from 'react'
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
  onSubmit: (label: string) => boolean
}

export function ProfileGroupDialog({ open, group, onOpenChange, onSubmit }: ProfileGroupDialogProps) {
  const [label, setLabel] = useState(group?.label ?? '')
  const [error, setError] = useState('')
  const isRename = Boolean(group)

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const trimmed = label.trim().replace(/\s+/g, ' ')
    if (!trimmed) {
      setError('Group name cannot be empty.')
      return
    }
    if (!onSubmit(trimmed)) {
      setError('A group with this name already exists.')
      return
    }
    setError('')
    onOpenChange(false)
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent className="border-hairline bg-raised text-primary sm:max-w-[440px]">
        <DialogHeader>
          <p className="type-eyebrow">Profile groups</p>
          <DialogTitle className="type-display mt-2 text-2xl text-primary">{isRename ? 'Rename group' : 'New group'}</DialogTitle>
          <DialogDescription className="text-[12px] leading-5 text-muted-copy">
            {isRename ? 'Change the label shown above this profile collection.' : 'Create a local group for single keys you want to keep together.'}
          </DialogDescription>
        </DialogHeader>
        <form className="space-y-5" onSubmit={handleSubmit}>
          <Field>
            <FieldLabel className="text-[12px] text-primary" htmlFor="profile-group-name">Group name</FieldLabel>
            <Input aria-describedby="profile-group-helper" autoFocus className="h-[42px] border-white/10 bg-surface text-[13px]" id="profile-group-name" onChange={(event) => { setLabel(event.target.value); setError('') }} placeholder="e.g. Travel keys" value={label} />
            <FieldDescription className="text-[11px] leading-4 text-muted-copy" id="profile-group-helper">Names are shared across Default, local groups, and subscription groups.</FieldDescription>
          </Field>
          {error ? <FieldError className="text-[11px]">{error}</FieldError> : null}
          <DialogFooter>
            <Button onClick={() => onOpenChange(false)} type="button" variant="ghost">Cancel</Button>
            <Button className="gap-1.5 bg-lavender text-ink hover:bg-lavender-hi" type="submit"><Check aria-hidden="true" className="size-3.5" />{isRename ? 'Save group name' : 'Create group'}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
