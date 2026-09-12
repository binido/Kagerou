import { useTranslation } from 'react-i18next'

import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from '@/components/ui/alert-dialog'
import type { ProfileGroup } from '@/types/kagerou'

interface DeleteSubscriptionDialogProps {
  group: ProfileGroup | null
  onOpenChange: (open: boolean) => void
  onConfirm: () => void
}

export function DeleteSubscriptionDialog({ group, onOpenChange, onConfirm }: DeleteSubscriptionDialogProps) {
  const { t } = useTranslation('profiles')

  return (
    <AlertDialog onOpenChange={onOpenChange} open={Boolean(group)}>
      <AlertDialogContent className="border-hairline bg-raised text-primary sm:max-w-[440px]">
        <AlertDialogHeader>
          <p className="type-eyebrow !text-bad">{t('dialogs.deleteSubscription.eyebrow')}</p>
          <AlertDialogTitle className="type-display mt-2 text-[23px] text-primary">{t('dialogs.deleteSubscription.title', { group: group?.label ?? '' })}</AlertDialogTitle>
          <AlertDialogDescription className="text-[13px] leading-5 text-body">{t('dialogs.deleteSubscription.description', { count: group?.profileIds.length ?? 0 })}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t('dialogs.deleteSubscription.keep')}</AlertDialogCancel>
          <AlertDialogAction className="bg-bad text-ink hover:bg-bad/85" onClick={onConfirm}>{t('dialogs.deleteSubscription.submit')}</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
