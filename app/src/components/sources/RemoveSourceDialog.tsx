import { useTranslation } from 'react-i18next'
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from '@/components/ui/alert-dialog'
import type { Source } from '@/types/kagerou'

interface RemoveSourceDialogProps {
  source?: Source | null
  onOpenChange: (open: boolean) => void
  onConfirm: () => void
}

export function RemoveSourceDialog({ source, onOpenChange, onConfirm }: RemoveSourceDialogProps) {
  const { t } = useTranslation('sources')
  const subscription = source?.type === 'url'

  return (
    <AlertDialog onOpenChange={onOpenChange} open={Boolean(source)}>
      <AlertDialogContent className="border-hairline bg-raised text-primary sm:max-w-[440px]">
        <AlertDialogHeader>
          <p className="type-eyebrow !text-bad">{t('removeDialog.eyebrow')}</p>
          <AlertDialogTitle className="type-display mt-2 text-[23px] text-primary">{t('removeDialog.title')}</AlertDialogTitle>
          <AlertDialogDescription className="text-[13px] leading-5 text-body">{subscription ? t('removeDialog.subscriptionDescription') : t('removeDialog.keyDescription')}</AlertDialogDescription>
        </AlertDialogHeader>
        <p className="rounded-lg bg-canvas px-3 py-2.5 font-mono text-[11px] text-muted-copy">{source?.name}</p>
        <AlertDialogFooter><AlertDialogCancel>{t('removeDialog.keep')}</AlertDialogCancel><AlertDialogAction className="bg-bad text-ink hover:bg-bad/85" onClick={onConfirm}>{t('removeDialog.remove')}</AlertDialogAction></AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
