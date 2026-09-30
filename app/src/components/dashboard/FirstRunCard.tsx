import { ClipboardPaste } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { ImportDialog } from '@/components/profiles/ImportDialog'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { useProfileImport } from '@/hooks/use-profile-import'
import { useKagerouStore } from '@/store/kagerou-store'

/** Stands in for the connection dial until there is a VPN to connect to. */
export function FirstRunCard() {
  const { t } = useTranslation('dashboard')
  const { t: tp } = useTranslation('profiles')
  // Read from the store, not the render: the group an import creates is not
  // in this render's props yet.
  const labelOfId = (groupId: string) => {
    const group = useKagerouStore.getState().profileGroups.find((g) => g.id === groupId)
    return group?.kind === 'default'
      ? tp('group.defaultName')
      : (group?.label ?? tp('fallback.group'))
  }
  const imports = useProfileImport(labelOfId)

  return (
    <Card className="flex min-h-0 flex-1 flex-col items-center justify-center gap-4 rounded-[10px] border border-hairline bg-surface p-7 text-center shadow-none">
      <p className="type-eyebrow">{t('firstRun.eyebrow')}</p>
      <h2 className="type-display text-[22px] leading-tight text-primary" id="first-run-title">
        {t('firstRun.title')}
      </h2>
      <p className="max-w-[440px] text-[13px] leading-5 text-muted-copy">
        {t('firstRun.description')}
      </p>
      <Button
        className="h-10 gap-2 bg-lavender px-3.5 text-[12px] font-semibold text-ink hover:bg-lavender-hi"
        disabled={imports.importing}
        onClick={imports.fromClipboard}
        type="button"
      >
        <ClipboardPaste aria-hidden="true" className="size-4" />
        {tp('actions.addFromClipboard')}
      </Button>
      <ImportDialog
        draft={imports.draft}
        onOpenChange={(open) => {
          if (!open) imports.dismissDraft()
        }}
        onSubmit={imports.fromText}
        submitting={imports.importing}
      />
    </Card>
  )
}
