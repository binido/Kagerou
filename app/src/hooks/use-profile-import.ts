import { useEffect, useEffectEvent, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

import type { ImportDraft } from '@/components/profiles/ImportDialog'
import { describeUnsupported } from '@/lib/unsupported'
import { useKagerouStore } from '@/store/kagerou-store'
import type { ImportAttempt, ImportOutcome } from '@/types/kagerou'

/** Pasting into a field, or anywhere inside an open dialog, is ordinary typing
 * rather than an import. */
const TYPING_TARGETS =
  'input, textarea, [contenteditable="true"], [role="dialog"], [role="alertdialog"]'

/** Adding VPNs: from the clipboard button, from the paste dialog, or from a
 * paste anywhere on the page.
 *
 * `groupLabel` is passed in because the name to announce has to be read
 * after the import, not from the render that started it: a brand new group
 * is not in this render's props yet.
 */
export function useProfileImport(groupLabel: (groupId: string) => string) {
  const { t } = useTranslation('profiles')
  const importText = useKagerouStore((state) => state.importText)
  const importFromClipboard = useKagerouStore((state) => state.importFromClipboard)
  const [importing, setImporting] = useState(false)
  const [draft, setDraft] = useState<ImportDraft | null>(null)

  const announce = (outcome: ImportOutcome, toastId: string | number) => {
    const options = { id: toastId, description: describeUnsupported(outcome.unsupported) }
    switch (outcome.kind) {
      case 'subscriptionAdded':
        return toast.success(
          t('import.subscriptionAdded', {
            name: groupLabel(outcome.groupId),
            count: outcome.added,
          }),
          options,
        )
      case 'subscriptionRefreshed':
        return toast.success(
          t('import.subscriptionRefreshed', { name: groupLabel(outcome.groupId) }),
          options,
        )
      case 'profileAdded':
        return toast.success(t('import.profileAdded', { name: outcome.name }), options)
      case 'groupAdded':
        return toast.success(
          outcome.skipped > 0
            ? t('import.groupAddedSkipped', {
                name: groupLabel(outcome.groupId),
                count: outcome.added,
                skipped: outcome.skipped,
              })
            : t('import.groupAdded', { name: groupLabel(outcome.groupId), count: outcome.added }),
          options,
        )
      case 'alreadyPresent':
        return toast.info(
          t('import.alreadyPresent', { group: groupLabel(outcome.groupId) }),
          options,
        )
      case 'nothingNew':
        return toast.info(t('import.nothingNew', { count: outcome.skipped }), options)
    }
  }

  // A subscription is fetched over the network and can take seconds, hence
  // the loading toast. Anything that fails lands in the paste dialog with its
  // text, where it can be fixed and sent again.
  const run = async (attempt: () => Promise<ImportAttempt>) => {
    if (importing) return
    setImporting(true)
    const toastId = toast.loading(t('import.working'))
    const result = await attempt()
    setImporting(false)
    if (result.status === 'imported') {
      setDraft(null)
      announce(result.outcome, toastId)
      return
    }
    toast.dismiss(toastId)
    setDraft({ text: result.text, error: result.error })
  }

  const onDocumentPaste = useEffectEvent((event: ClipboardEvent) => {
    if (event.target instanceof Element && event.target.closest(TYPING_TARGETS)) return
    const text = event.clipboardData?.getData('text/plain') ?? ''
    if (!text.trim()) return
    event.preventDefault()
    void run(() => importText(text))
  })

  useEffect(() => {
    const listener = (event: ClipboardEvent) => onDocumentPaste(event)
    document.addEventListener('paste', listener)
    return () => document.removeEventListener('paste', listener)
  }, [])

  return {
    importing,
    draft,
    dismissDraft: () => setDraft(null),
    fromClipboard: () => {
      void run(importFromClipboard)
    },
    fromText: (text: string) => {
      void run(() => importText(text))
    },
  }
}
