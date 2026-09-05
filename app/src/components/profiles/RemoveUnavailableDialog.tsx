import { useTranslation } from 'react-i18next'
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from '@/components/ui/alert-dialog'

export interface RemoveUnavailableTarget {
  groupId: string
  groupLabel: string
  count: number
  activeKept: boolean
}

interface RemoveUnavailableDialogProps {
  target: RemoveUnavailableTarget | null
  onOpenChange: (open: boolean) => void
  onConfirm: () => void
}

export function RemoveUnavailableDialog({ target, onOpenChange, onConfirm }: RemoveUnavailableDialogProps) {
  const { t } = useTranslation('profiles')

  return (
    <AlertDialog onOpenChange={onOpenChange} open={Boolean(target)}>
      <AlertDialogContent className="border-hairline bg-raised text-primary sm:max-w-[440px]">
        <AlertDialogHeader>
          <p className="type-eyebrow !text-bad">{t('dialogs.removeUnavailable.eyebrow')}</p>
          <AlertDialogTitle className="type-display mt-2 text-[23px] text-primary">{t('dialogs.removeUnavailable.title')}</AlertDialogTitle>
          <AlertDialogDescription className="text-[13px] leading-5 text-body">{t('dialogs.removeUnavailable.description', { count: target?.count ?? 0, group: target?.groupLabel })}</AlertDialogDescription>
        </AlertDialogHeader>
        {target?.activeKept ? <p className="text-[12px] leading-5 text-body">{t('dialogs.removeUnavailable.activeKept')}</p> : null}
        <p className="rounded-lg bg-canvas px-3 py-2.5 font-mono text-[11px] text-muted-copy">{target?.groupLabel}</p>
        <AlertDialogFooter><AlertDialogCancel>{t('dialogs.removeUnavailable.cancel')}</AlertDialogCancel><AlertDialogAction className="bg-bad text-ink hover:bg-bad/85" onClick={onConfirm}>{t('dialogs.removeUnavailable.submit', { count: target?.count ?? 0 })}</AlertDialogAction></AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
