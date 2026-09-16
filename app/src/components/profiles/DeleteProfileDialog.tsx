import { useTranslation } from 'react-i18next'

import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from '@/components/ui/alert-dialog'
import type { Profile } from '@/types/kagerou'

interface DeleteProfileDialogProps {
  profile: Profile | null
  onOpenChange: (open: boolean) => void
  onConfirm: (profile: Profile) => void
}

export function DeleteProfileDialog({ profile, onOpenChange, onConfirm }: DeleteProfileDialogProps) {
  const { t } = useTranslation('profiles')

  return (
    <AlertDialog onOpenChange={onOpenChange} open={Boolean(profile)}>
      <AlertDialogContent className="border-hairline bg-raised text-primary sm:max-w-[440px]">
        <AlertDialogHeader>
          <AlertDialogTitle className="type-display text-2xl text-primary">
            {t('dialogs.delete.title')}
          </AlertDialogTitle>
          <AlertDialogDescription className="text-[12px] leading-5 text-muted-copy">
            {t('dialogs.delete.description')}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <div className="border-l-2 border-bad bg-bad/10 px-3 py-2.5 text-[12px] leading-5 text-body">
          {t('dialogs.delete.warningPrefix')} <strong className="text-primary">{profile?.name}</strong>{t('dialogs.delete.warningSuffix')}
        </div>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={() => onOpenChange(false)}>
            {t('dialogs.delete.cancel')}
          </AlertDialogCancel>
          <AlertDialogAction
            className="bg-bad text-primary hover:bg-bad/85"
            onClick={() => { if (profile) onConfirm(profile) }}
          >
            {t('dialogs.delete.submit')}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
