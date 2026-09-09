import { useTranslation } from 'react-i18next'

import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from '@/components/ui/alert-dialog'
import type { RoutingRule } from '@/types/kagerou'

interface DeleteRuleDialogProps {
  rule?: RoutingRule | null
  onOpenChange: (open: boolean) => void
  onConfirm: () => void
}

export function DeleteRuleDialog({ rule, onOpenChange, onConfirm }: DeleteRuleDialogProps) {
  const { t } = useTranslation('routing')

  return (
    <AlertDialog onOpenChange={onOpenChange} open={Boolean(rule)}>
      <AlertDialogContent className="border-hairline bg-raised text-primary sm:max-w-[440px]">
        <AlertDialogHeader>
          <p className="type-eyebrow !text-bad">{t('deleteDialog.eyebrow')}</p>
          <AlertDialogTitle className="type-display mt-2 text-[23px] text-primary">{t('deleteDialog.title')}</AlertDialogTitle>
          <AlertDialogDescription className="text-[13px] leading-5 text-body">{t('deleteDialog.description')}</AlertDialogDescription>
        </AlertDialogHeader>
        <p className="rounded-lg bg-canvas px-3 py-2.5 font-mono text-[11px] text-muted-copy">{rule?.match}</p>
        <AlertDialogFooter><AlertDialogCancel>{t('deleteDialog.keep')}</AlertDialogCancel><AlertDialogAction className="bg-bad text-ink hover:bg-bad/85" onClick={onConfirm}>{t('deleteDialog.delete')}</AlertDialogAction></AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
