import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from '@/components/ui/alert-dialog'
import type { Source } from '@/types/kagerou'

interface RemoveSourceDialogProps {
  source?: Source | null
  onOpenChange: (open: boolean) => void
  onConfirm: () => void
}

export function RemoveSourceDialog({ source, onOpenChange, onConfirm }: RemoveSourceDialogProps) {
  const subscription = source?.type === 'url'

  return (
    <AlertDialog onOpenChange={onOpenChange} open={Boolean(source)}>
      <AlertDialogContent className="border-hairline bg-raised text-primary sm:max-w-[440px]">
        <AlertDialogHeader>
          <p className="type-eyebrow !text-bad">Remove source</p>
          <AlertDialogTitle className="type-display mt-2 text-[23px] text-primary">Remove this source?</AlertDialogTitle>
          <AlertDialogDescription className="text-[13px] leading-5 text-body">{subscription ? 'The subscription will stop refreshing. Its profiles stay available in a new local group that you can reorganize.' : 'The key source will be removed. Its local profile stays in Default and remains available.'}</AlertDialogDescription>
        </AlertDialogHeader>
        <p className="rounded-lg bg-canvas px-3 py-2.5 font-mono text-[11px] text-muted-copy">{source?.name}</p>
        <AlertDialogFooter><AlertDialogCancel>Keep source</AlertDialogCancel><AlertDialogAction className="bg-bad text-[#21171a] hover:bg-bad/85" onClick={onConfirm}>Remove source</AlertDialogAction></AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
