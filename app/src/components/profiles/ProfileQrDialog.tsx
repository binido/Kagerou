import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Copy } from 'lucide-react'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import type { Profile } from '@/types/kagerou'

interface ProfileQrDialogProps {
  profile: Profile | null
  loadQr: (id: string) => Promise<string | null>
  onCopyLink: (profile: Profile) => void
  onOpenChange: (open: boolean) => void
}

export function ProfileQrDialog({
  profile,
  loadQr,
  onCopyLink,
  onOpenChange,
}: ProfileQrDialogProps) {
  const { t } = useTranslation('profiles')
  const [qr, setQr] = useState<{ id: string; svg: string } | null>(null)
  const profileId = profile?.id

  useEffect(() => {
    if (!profileId) return
    let current = true
    void loadQr(profileId).then((svg) => {
      if (current && svg) setQr({ id: profileId, svg })
    })
    return () => {
      current = false
    }
  }, [profileId, loadQr])

  const svg = qr && qr.id === profileId ? qr.svg : null

  return (
    <Dialog onOpenChange={onOpenChange} open={Boolean(profile)}>
      <DialogContent className="border-hairline bg-raised text-primary sm:max-w-[360px]">
        <DialogHeader>
          <DialogTitle className="type-display truncate text-2xl text-primary">
            {t('share.qrTitle', { name: profile?.name ?? '' })}
          </DialogTitle>
          <DialogDescription className="text-[12px] text-muted-copy">
            {t('share.qrDescription')}
          </DialogDescription>
        </DialogHeader>
        <div className="mx-auto flex size-[256px] items-center justify-center rounded-lg bg-white">
          {svg ? (
            <img
              alt={t('share.qrAlt', { name: profile?.name ?? '' })}
              className="size-full rounded-lg"
              src={`data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`}
            />
          ) : (
            <span className="text-[12px] text-neutral-500" role="status">
              {t('share.qrLoading')}
            </span>
          )}
        </div>
        <DialogFooter>
          <Button
            className="h-9 gap-2 text-[12px]"
            onClick={() => {
              if (profile) onCopyLink(profile)
            }}
            type="button"
            variant="outline"
          >
            <Copy aria-hidden="true" className="size-3.5" />
            {t('menu.copyLink')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
