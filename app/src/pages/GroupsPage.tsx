import { useEffect, useEffectEvent, useMemo, useState, type FormEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { Check, ClipboardPaste, FolderPlus, Radar } from 'lucide-react'
import { toast } from 'sonner'

import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Field, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { PageContainer } from '@/components/layout/PageContainer'
import { PageHeader } from '@/components/layout/PageHeader'
import { DeleteSubscriptionDialog } from '@/components/profiles/DeleteSubscriptionDialog'
import { ImportDialog, type ImportDraft } from '@/components/profiles/ImportDialog'
import { ProfileGroupCard } from '@/components/profiles/ProfileGroupCard'
import { ProfileGroupDialog } from '@/components/profiles/ProfileGroupDialog'
import { RemoveUnavailableDialog, type RemoveUnavailableTarget } from '@/components/profiles/RemoveUnavailableDialog'
import { SubscriptionUrlDialog } from '@/components/profiles/SubscriptionUrlDialog'
import { TestRunBar } from '@/components/profiles/TestRunBar'
import { backendErrorMessage } from '@/lib/errors'
import { localizeResultValue } from '@/lib/result-copy'
import { sortProfiles } from '@/lib/profile-sorting'
import { useKagerouStore } from '@/store/kagerou-store'
import type { ImportAttempt, ImportOutcome, Profile, ProfileGroup, Source } from '@/types/kagerou'

/** Pasting into a field, or anywhere inside an open dialog, is ordinary typing
 * rather than an import. */
const TYPING_TARGETS = 'input, textarea, [contenteditable="true"], [role="dialog"], [role="alertdialog"]'

export function GroupsPage() {
  const { t } = useTranslation('profiles')
  const { t: tc } = useTranslation('common')
  const profiles = useKagerouStore((state) => state.profiles)
  const groups = useKagerouStore((state) => state.profileGroups)
  const sources = useKagerouStore((state) => state.sources)
  const connected = useKagerouStore((state) => state.connected)
  const activeProfileId = useKagerouStore((state) => state.activeProfileId)
  const selectProfile = useKagerouStore((state) => state.selectProfile)
  const setProfileGroupOpen = useKagerouStore((state) => state.setProfileGroupOpen)
  const renameProfile = useKagerouStore((state) => state.renameProfile)
  const addProfileGroup = useKagerouStore((state) => state.addProfileGroup)
  const renameProfileGroup = useKagerouStore((state) => state.renameProfileGroup)
  const deleteProfile = useKagerouStore((state) => state.deleteProfile)
  const moveProfileToGroup = useKagerouStore((state) => state.moveProfileToGroup)
  const groupSort = useKagerouStore((state) => state.settings.groupSort)
  const runProfileTest = useKagerouStore((state) => state.runProfileTest)
  const clearGroupTestResults = useKagerouStore((state) => state.clearGroupTestResults)
  const deleteUnavailableProfiles = useKagerouStore((state) => state.deleteUnavailableProfiles)
  const testRun = useKagerouStore((state) => state.testRun)
  const startGroupTest = useKagerouStore((state) => state.startGroupTest)
  const cancelGroupTest = useKagerouStore((state) => state.cancelGroupTest)
  const importText = useKagerouStore((state) => state.importText)
  const importFromClipboard = useKagerouStore((state) => state.importFromClipboard)
  const updateSource = useKagerouStore((state) => state.updateSource)
  const refreshSource = useKagerouStore((state) => state.refreshSource)
  const deleteSubscription = useKagerouStore((state) => state.deleteSubscription)

  const [importing, setImporting] = useState(false)
  const [importDraft, setImportDraft] = useState<ImportDraft | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<Profile | null>(null)
  const [renameTarget, setRenameTarget] = useState<Profile | null>(null)
  const [renameValue, setRenameValue] = useState('')
  const [groupDialogOpen, setGroupDialogOpen] = useState(false)
  const [groupDialogTarget, setGroupDialogTarget] = useState<ProfileGroup | null>(null)
  const [runningTests, setRunningTests] = useState<Record<string, boolean>>({})
  const [refreshingIds, setRefreshingIds] = useState<Record<string, boolean>>({})
  const [urlTarget, setUrlTarget] = useState<Source | null>(null)
  const [deleteSubscriptionTarget, setDeleteSubscriptionTarget] = useState<ProfileGroup | null>(null)
  const [removeUnavailableTarget, setRemoveUnavailableTarget] = useState<RemoveUnavailableTarget | null>(null)
  const [feedback, setFeedback] = useState('')
  const [feedbackTone, setFeedbackTone] = useState<'muted' | 'good' | 'bad'>('muted')

  const profilesById = useMemo(() => new Map(profiles.map((profile) => [profile.id, profile])), [profiles])
  const sourcesById = useMemo(() => new Map(sources.map((source) => [source.id, source])), [sources])
  const visibleGroupProfiles = (group: ProfileGroup) => group.profileIds.map((id) => profilesById.get(id)).filter((profile): profile is Profile => Boolean(profile))
  const groupLabel = (group?: ProfileGroup) => group?.kind === 'default' ? t('group.defaultName') : group?.label ?? t('fallback.group')

  const setMessage = (message: string, tone: 'muted' | 'good' | 'bad' = 'muted') => {
    setFeedback(message)
    setFeedbackTone(tone)
  }

  // Reads groups from the store rather than this render: the import has just
  // refreshed them, and a new group is not in the props yet.
  const announceImport = (outcome: ImportOutcome, toastId: string | number) => {
    const labelOf = (groupId: string) => groupLabel(useKagerouStore.getState().profileGroups.find((group) => group.id === groupId))
    switch (outcome.kind) {
      case 'subscriptionAdded':
        toast.success(t('import.subscriptionAdded', { name: labelOf(outcome.groupId), count: outcome.added }), { id: toastId })
        return
      case 'subscriptionRefreshed':
        toast.success(t('import.subscriptionRefreshed', { name: labelOf(outcome.groupId) }), { id: toastId })
        return
      case 'profileAdded':
        toast.success(t('import.profileAdded', { name: outcome.name }), { id: toastId })
        return
      case 'groupAdded':
        toast.success(outcome.skipped > 0
          ? t('import.groupAddedSkipped', { name: labelOf(outcome.groupId), count: outcome.added, skipped: outcome.skipped })
          : t('import.groupAdded', { name: labelOf(outcome.groupId), count: outcome.added }), { id: toastId })
        return
      case 'alreadyPresent':
        toast.info(t('import.alreadyPresent', { group: labelOf(outcome.groupId) }), { id: toastId })
        return
      case 'nothingNew':
        toast.info(t('import.nothingNew', { count: outcome.skipped }), { id: toastId })
    }
  }

  // A subscription is fetched over the network and can take seconds, hence
  // the loading toast. Anything that fails lands in the paste dialog with its
  // text, where it can be fixed and sent again.
  const runImport = async (attempt: () => Promise<ImportAttempt>) => {
    if (importing) return
    setImporting(true)
    const toastId = toast.loading(t('import.working'))
    const result = await attempt()
    setImporting(false)
    if (result.status === 'imported') {
      setImportDraft(null)
      announceImport(result.outcome, toastId)
      return
    }
    toast.dismiss(toastId)
    setImportDraft({ text: result.text, error: result.error })
  }

  const onDocumentPaste = useEffectEvent((event: ClipboardEvent) => {
    if (event.target instanceof Element && event.target.closest(TYPING_TARGETS)) return
    const text = event.clipboardData?.getData('text/plain') ?? ''
    if (!text.trim()) return
    event.preventDefault()
    void runImport(() => importText(text))
  })

  useEffect(() => {
    const listener = (event: ClipboardEvent) => onDocumentPaste(event)
    document.addEventListener('paste', listener)
    return () => document.removeEventListener('paste', listener)
  }, [])

  const runTest = async (profileId: string) => {
    const profile = profilesById.get(profileId)
    const name = profile?.name ?? t('fallback.vpn')
    setRunningTests((state) => ({ ...state, [profileId]: true }))
    setMessage(t('feedback.testRunning', { name }))
    const result = await runProfileTest(profileId)
    setRunningTests((state) => {
      const next = { ...state }
      delete next[profileId]
      return next
    })
    if (!result) {
      setMessage(t('feedback.testFailed', { name }), 'bad')
      return
    }
    setMessage(t('feedback.testFinished', { name, value: localizeResultValue(result.value, tc) }), result.tone === 'bad' ? 'bad' : 'good')
  }

  const runAll = () => {
    void startGroupTest(null)
    setMessage(t('feedback.testRunningAll'))
  }

  const runGroupTest = (group: ProfileGroup) => {
    void startGroupTest(group.id)
    setMessage(t('feedback.testRunningGroup', { group: groupLabel(group) }))
  }

  // A run holds the test core's selector, so nothing else may be aimed at it.
  const groupTestRunning = (group: ProfileGroup) =>
    Boolean(testRun) || group.profileIds.some((id) => runningTests[id])

  // Count excludes the active profile — it will be kept — and the dialog
  // never opens when nothing failed. The backend re-evaluates the same
  // predicate in its transaction, so a stale count here is only cosmetic.
  const openRemoveUnavailable = (group: ProfileGroup) => {
    const failing = group.profileIds.filter((id) => profilesById.get(id)?.url.tone === 'bad')
    if (failing.length === 0) {
      setMessage(t('feedback.nothingUnavailable', { group: groupLabel(group) }))
      return
    }
    const activeKept = failing.some((id) => profilesById.get(id)?.selected)
    setRemoveUnavailableTarget({
      groupId: group.id,
      groupLabel: groupLabel(group),
      count: failing.length - (activeKept ? 1 : 0),
      activeKept,
    })
  }

  const confirmRemoveUnavailable = async () => {
    const target = removeUnavailableTarget
    if (!target) return
    const deleted = await deleteUnavailableProfiles(target.groupId)
    setRemoveUnavailableTarget(null)
    const message = t('feedback.deletedUnavailable', { count: deleted, group: target.groupLabel })
    setMessage(target.activeKept ? `${message} ${t('feedback.activeKept')}` : message, 'good')
  }

  const refreshSubscription = async (group: ProfileGroup | undefined, source: Source) => {
    if (refreshingIds[source.id]) return
    const name = groupLabel(group)
    setRefreshingIds((state) => ({ ...state, [source.id]: true }))
    const toastId = toast.loading(t('feedback.refreshing', { name }))
    try {
      await refreshSource(source.id)
      toast.success(t('feedback.refreshed', { name }), { id: toastId })
    } catch (error) {
      toast.error(backendErrorMessage(error, t('feedback.refreshFailed')), { id: toastId })
    } finally {
      setRefreshingIds((state) => {
        const next = { ...state }
        delete next[source.id]
        return next
      })
    }
  }

  const submitSubscriptionUrl = async (url: string) => {
    const target = urlTarget
    if (!target || !(await updateSource(target.id, { value: url }))) return false
    setUrlTarget(null)
    void refreshSubscription(groups.find((group) => group.sourceId === target.id), target)
    return true
  }

  const copySubscriptionUrl = async (source: Source) => {
    try {
      await navigator.clipboard.writeText(source.value)
      toast.success(t('feedback.urlCopied'))
    } catch {
      toast.error(t('feedback.urlCopyFailed'))
    }
  }

  // Checked here only to say it in the user's language before a dialog opens;
  // the backend refuses the same case on its own.
  const openDeleteSubscription = (group: ProfileGroup) => {
    if (connected && group.profileIds.includes(activeProfileId)) {
      toast.error(t('feedback.subscriptionInUse', { group: groupLabel(group) }))
      return
    }
    setDeleteSubscriptionTarget(group)
  }

  const confirmDeleteSubscription = async () => {
    const target = deleteSubscriptionTarget
    if (!target) return
    setDeleteSubscriptionTarget(null)
    if (await deleteSubscription(target.id)) toast.success(t('feedback.subscriptionDeleted', { group: groupLabel(target) }))
  }

  const submitRename = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const trimmed = renameValue.trim()
    if (!renameTarget) return
    if (!trimmed) {
      setMessage(t('feedback.vpnNameEmpty'), 'bad')
      return
    }
    const renamed = await renameProfile(renameTarget.id, trimmed)
    if (!renamed) {
      setMessage(t('feedback.renameLocalOnly'), 'bad')
      return
    }
    setMessage(t('feedback.renamed', { name: trimmed }), 'good')
    setRenameTarget(null)
  }

  const handleGroupSubmit = async (label: string) => {
    if (groupDialogTarget) {
      const updated = await renameProfileGroup(groupDialogTarget.id, label)
      if (updated) setMessage(t('feedback.groupRenamed', { name: label }), 'good')
      return updated
    }
    const id = await addProfileGroup(label)
    if (id) setMessage(t('feedback.groupCreated', { name: label }), 'good')
    return Boolean(id)
  }

  const movableGroups = groups.filter((group) => group.kind !== 'subscription')
  const sortedGroupProfiles = (group: ProfileGroup) => sortProfiles(visibleGroupProfiles(group), groupSort)

  return (
    <PageContainer>
      <PageHeader
        actions={(
          <div className="flex flex-wrap justify-end gap-2">
            <Button className="h-10 gap-2 border-hairline bg-surface px-3.5 text-[12px] text-body hover:bg-raised hover:text-primary" onClick={() => { setGroupDialogTarget(null); setGroupDialogOpen(true) }} type="button" variant="outline"><FolderPlus aria-hidden="true" className="size-4" />{t('actions.addGroup')}</Button>
            <Button className="h-10 gap-2 bg-lavender px-3.5 text-[12px] font-semibold text-ink hover:bg-lavender-hi" disabled={importing} onClick={() => { void runImport(importFromClipboard) }} type="button"><ClipboardPaste aria-hidden="true" className="size-4" />{t('actions.addFromClipboard')}</Button>
            <Button className="h-10 gap-2 border-hairline bg-surface px-3.5 text-[12px] text-body hover:bg-raised hover:text-primary" onClick={runAll} type="button" variant="outline"><Radar aria-hidden="true" className="size-4" />{t('actions.runTest')}</Button>
          </div>
        )}
        description={t('page.description')}
        eyebrow={t('page.eyebrow')}
        title={t('page.title')}
      />

      {testRun ? <TestRunBar onCancel={() => { void cancelGroupTest() }} run={testRun} /> : null}
      <div className="mt-7 flex flex-col gap-4">
        {groups.map((group) => {
          const source = group.sourceId ? sourcesById.get(group.sourceId) : undefined
          return (
            <ProfileGroupCard
              group={group}
              key={group.id}
              movableGroups={movableGroups}
              onChangeUrl={() => { if (source) setUrlTarget(source) }}
              onClearResults={() => {
                void clearGroupTestResults(group.id)
                setMessage(t('feedback.resultsCleared', { group: groupLabel(group) }))
              }}
              onCopyUrl={() => { if (source) void copySubscriptionUrl(source) }}
              onDelete={setDeleteTarget}
              onDeleteSubscription={() => openDeleteSubscription(group)}
              onDeleteUnavailable={() => openRemoveUnavailable(group)}
              onMoveToGroup={(profileId, targetGroupId) => {
                const target = groups.find((candidate) => candidate.id === targetGroupId)
                void moveProfileToGroup(profileId, targetGroupId).then((moved) => {
                  setMessage(moved ? t('feedback.moved', { group: groupLabel(target) }) : t('feedback.moveSubscription'), moved ? 'good' : 'bad')
                })
              }}
              onRefresh={() => { if (source) void refreshSubscription(group, source) }}
              onRename={(profile) => { setRenameTarget(profile); setRenameValue(profile.name) }}
              onRenameGroup={(target) => { setGroupDialogTarget(target); setGroupDialogOpen(true) }}
              onSelect={(id) => { selectProfile(id); const selected = profilesById.get(id); if (selected) setMessage(t('feedback.selected', { name: selected.name }), 'good') }}
              onTest={runTest}
              onTestGroup={() => runGroupTest(group)}
              onToggle={() => setProfileGroupOpen(group.id, !group.open)}
              profiles={sortedGroupProfiles(group)}
              refreshing={Boolean(source && refreshingIds[source.id])}
              runningTests={runningTests}
              source={source}
              testRunning={groupTestRunning(group)}
            />
          )
        })}
      </div>
      <p aria-live="polite" className={`mt-4 min-h-[17px] text-[11px] ${feedbackTone === 'good' ? 'text-good' : feedbackTone === 'bad' ? 'text-bad' : 'text-muted-copy'}`}>{feedback}</p>
      <p className="sr-only">{t('table.available', { count: profiles.length })}</p>

      <ProfileGroupDialog group={groupDialogTarget} onOpenChange={(open) => { setGroupDialogOpen(open); if (!open) setGroupDialogTarget(null) }} onSubmit={handleGroupSubmit} open={groupDialogOpen} />

      <RemoveUnavailableDialog onConfirm={() => { void confirmRemoveUnavailable() }} onOpenChange={(open) => { if (!open) setRemoveUnavailableTarget(null) }} target={removeUnavailableTarget} />

      <ImportDialog draft={importDraft} onOpenChange={(open) => { if (!open) setImportDraft(null) }} onSubmit={(text) => { void runImport(() => importText(text)) }} submitting={importing} />

      <SubscriptionUrlDialog onOpenChange={(open) => { if (!open) setUrlTarget(null) }} onSubmit={submitSubscriptionUrl} source={urlTarget} />

      <DeleteSubscriptionDialog group={deleteSubscriptionTarget} onConfirm={() => { void confirmDeleteSubscription() }} onOpenChange={(open) => { if (!open) setDeleteSubscriptionTarget(null) }} />

      <Dialog onOpenChange={(open) => { if (!open) setRenameTarget(null) }} open={Boolean(renameTarget)}>
        <DialogContent className="border-hairline bg-raised text-primary sm:max-w-[420px]">
          <DialogHeader><DialogTitle className="type-display text-2xl text-primary">{t('dialogs.rename.title')}</DialogTitle><DialogDescription className="text-[12px] text-muted-copy">{t('dialogs.rename.description')}</DialogDescription></DialogHeader>
          <form className="space-y-5" onSubmit={submitRename}>
            <Field><FieldLabel className="text-[12px] text-primary" htmlFor="rename-profile">{t('dialogs.rename.nameLabel')}</FieldLabel><Input autoFocus className="h-[42px] border-hairline bg-surface text-[13px]" id="rename-profile" onChange={(event) => setRenameValue(event.target.value)} value={renameValue} /></Field>
            <DialogFooter><Button onClick={() => setRenameTarget(null)} type="button" variant="ghost">{t('dialogs.rename.cancel')}</Button><Button className="bg-lavender text-ink hover:bg-lavender-hi" type="submit"><Check aria-hidden="true" className="size-3.5" />{t('dialogs.rename.submit')}</Button></DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <AlertDialog onOpenChange={(open) => { if (!open) setDeleteTarget(null) }} open={Boolean(deleteTarget)}>
        <AlertDialogContent className="border-hairline bg-raised text-primary sm:max-w-[440px]">
          <AlertDialogHeader><AlertDialogTitle className="type-display text-2xl text-primary">{t('dialogs.delete.title')}</AlertDialogTitle><AlertDialogDescription className="text-[12px] leading-5 text-muted-copy">{t('dialogs.delete.description')}</AlertDialogDescription></AlertDialogHeader>
          <div className="border-l-2 border-bad bg-bad/10 px-3 py-2.5 text-[12px] leading-5 text-body">{t('dialogs.delete.warningPrefix')} <strong className="text-primary">{deleteTarget?.name}</strong>{t('dialogs.delete.warningSuffix')}</div>
          <AlertDialogFooter><AlertDialogCancel onClick={() => setDeleteTarget(null)}>{t('dialogs.delete.cancel')}</AlertDialogCancel><AlertDialogAction className="bg-bad text-primary hover:bg-bad/85" onClick={() => { if (!deleteTarget) return; const nameToDelete = deleteTarget.name; deleteProfile(deleteTarget.id); setDeleteTarget(null); setMessage(t('feedback.deleted', { name: nameToDelete }), 'good') }}>{t('dialogs.delete.submit')}</AlertDialogAction></AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </PageContainer>
  )
}
