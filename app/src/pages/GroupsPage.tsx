import { useMemo, useState, type FormEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { Check, ChevronDown, FolderPlus, Loader2, Plus, Radar } from 'lucide-react'

import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { Field, FieldDescription, FieldError, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { PageHeader } from '@/components/layout/PageHeader'
import { ProfileGroupCard } from '@/components/profiles/ProfileGroupCard'
import { ProfileGroupDialog } from '@/components/profiles/ProfileGroupDialog'
import { localizeResultValue } from '@/lib/result-copy'
import { mockApi } from '@/lib/mock-api'
import { sortProfiles } from '@/lib/profile-sorting'
import { useKagerouStore } from '@/store/kagerou-store'
import type { Profile, ProfileGroup, TestMethod } from '@/types/kagerou'

export function GroupsPage() {
  const { t } = useTranslation('profiles')
  const { t: tc } = useTranslation('common')
  const profiles = useKagerouStore((state) => state.profiles)
  const groups = useKagerouStore((state) => state.profileGroups)
  const selectProfile = useKagerouStore((state) => state.selectProfile)
  const setProfileGroupOpen = useKagerouStore((state) => state.setProfileGroupOpen)
  const addLocalProfile = useKagerouStore((state) => state.addLocalProfile)
  const renameProfile = useKagerouStore((state) => state.renameProfile)
  const addProfileGroup = useKagerouStore((state) => state.addProfileGroup)
  const renameProfileGroup = useKagerouStore((state) => state.renameProfileGroup)
  const deleteProfile = useKagerouStore((state) => state.deleteProfile)
  const moveProfileToGroup = useKagerouStore((state) => state.moveProfileToGroup)
  const groupSort = useKagerouStore((state) => state.settings.groupSort)
  const setTestResult = useKagerouStore((state) => state.setTestResult)

  const [addOpen, setAddOpen] = useState(false)
  const [name, setName] = useState('')
  const [key, setKey] = useState('')
  const [addError, setAddError] = useState('')
  const [deleteTarget, setDeleteTarget] = useState<Profile | null>(null)
  const [renameTarget, setRenameTarget] = useState<Profile | null>(null)
  const [renameValue, setRenameValue] = useState('')
  const [groupDialogOpen, setGroupDialogOpen] = useState(false)
  const [groupDialogTarget, setGroupDialogTarget] = useState<ProfileGroup | null>(null)
  const [runningTests, setRunningTests] = useState<Record<string, boolean>>({})
  const [feedback, setFeedback] = useState('')
  const [feedbackTone, setFeedbackTone] = useState<'muted' | 'good' | 'bad'>('muted')

  const profilesById = useMemo(() => new Map(profiles.map((profile) => [profile.id, profile])), [profiles])
  const visibleGroupProfiles = (group: ProfileGroup) => group.profileIds.map((id) => profilesById.get(id)).filter((profile): profile is Profile => Boolean(profile))
  const groupLabel = (group?: ProfileGroup) => group?.kind === 'default' ? t('group.defaultName') : group?.label ?? t('fallback.group')

  const setMessage = (message: string, tone: 'muted' | 'good' | 'bad' = 'muted') => {
    setFeedback(message)
    setFeedbackTone(tone)
  }

  const runTest = async (profileId: string, method: TestMethod) => {
    const keyName = `${profileId}:${method}`
    const methodLabel = method === 'tcp' ? t('test.tcpShort') : t('test.urlShort')
    const profile = profilesById.get(profileId)
    setRunningTests((state) => ({ ...state, [keyName]: true }))
    setMessage(t('feedback.testRunning', { method: methodLabel, name: profile?.name ?? t('fallback.vpn') }))
    const result = await mockApi.runProfileTest(profileId, method)
    setTestResult(profileId, method, result)
    setRunningTests((state) => {
      const next = { ...state }
      delete next[keyName]
      return next
    })
    setMessage(t('feedback.testFinished', { method: methodLabel, name: profile?.name ?? t('fallback.vpn'), value: localizeResultValue(result.value, tc) }), result.tone === 'bad' ? 'bad' : 'good')
  }

  const runAll = (method: TestMethod) => {
    const methodLabel = method === 'tcp' ? t('test.tcpShort') : t('test.urlShort')
    void Promise.all(profiles.map((profile) => runTest(profile.id, method)))
    setMessage(t('feedback.testRunningAll', { method: methodLabel }))
  }

  const submitAdd = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const trimmedName = name.trim()
    const trimmedKey = key.trim()
    if (!trimmedName) {
      setAddError(t('dialogs.add.nameRequired'))
      return
    }
    if (!/^(vless|vmess|trojan|ss|hysteria2):\/\/[^\s]+$/i.test(trimmedKey)) {
      setAddError(t('dialogs.add.keyInvalid'))
      return
    }
    addLocalProfile({ key: trimmedKey, name: trimmedName })
    setName('')
    setKey('')
    setAddError('')
    setAddOpen(false)
    setMessage(t('feedback.added', { name: trimmedName }), 'good')
  }

  const submitRename = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const trimmed = renameValue.trim()
    if (!renameTarget) return
    if (!trimmed) {
      setMessage(t('feedback.vpnNameEmpty'), 'bad')
      return
    }
    const renamed = renameProfile(renameTarget.id, trimmed)
    if (!renamed) {
      setMessage(t('feedback.renameLocalOnly'), 'bad')
      return
    }
    setMessage(t('feedback.renamed', { name: trimmed }), 'good')
    setRenameTarget(null)
  }

  const handleGroupSubmit = (label: string) => {
    if (groupDialogTarget) {
      const updated = renameProfileGroup(groupDialogTarget.id, label)
      if (updated) setMessage(t('feedback.groupRenamed', { name: label }), 'good')
      return updated
    }
    const id = addProfileGroup(label)
    if (id) setMessage(t('feedback.groupCreated', { name: label }), 'good')
    return Boolean(id)
  }

  const groupsForRender = groups
  const movableGroups = groups.filter((group) => group.kind !== 'subscription')
  const sortedGroupProfiles = (group: ProfileGroup) => sortProfiles(visibleGroupProfiles(group), groupSort)

  return (
    <div className="min-h-screen min-w-0 bg-canvas px-6 pb-10 pt-8 lg:px-12 lg:pt-10">
      <div className="mx-auto w-full max-w-[1040px]">
        <PageHeader
          actions={(
            <div className="flex flex-wrap justify-end gap-2">
              <Button className="h-10 gap-2 border-hairline bg-surface px-3.5 text-[12px] text-body hover:bg-raised hover:text-primary" onClick={() => { setGroupDialogTarget(null); setGroupDialogOpen(true) }} type="button" variant="outline"><FolderPlus aria-hidden="true" className="size-4" />{t('actions.addGroup')}</Button>
              <Button className="h-10 gap-2 bg-lavender px-3.5 text-[12px] font-semibold text-ink hover:bg-lavender-hi" onClick={() => setAddOpen(true)} type="button"><Plus aria-hidden="true" className="size-4" />{t('actions.addSingleKey')}</Button>
              <DropdownMenu>
                <DropdownMenuTrigger asChild><Button className="h-10 gap-2 border-hairline bg-surface px-3.5 text-[12px] text-body hover:bg-raised hover:text-primary" type="button" variant="outline"><Radar aria-hidden="true" className="size-4" />{t('actions.runTest')}<ChevronDown aria-hidden="true" className="size-3 text-muted-copy" /></Button></DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-48 border-hairline bg-popover text-[11px]">
                  <DropdownMenuLabel className="font-mono text-[9px] uppercase tracking-[0.08em] text-muted-copy">{t('test.allVpns')}</DropdownMenuLabel>
                  <DropdownMenuItem onSelect={() => runAll('tcp')}><Loader2 aria-hidden="true" className="size-3.5" />{t('test.tcp')}</DropdownMenuItem>
                  <DropdownMenuItem onSelect={() => runAll('url')}><Radar aria-hidden="true" className="size-3.5" />{t('test.url')}</DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem onSelect={() => { setMessage(t('test.aboutText')) }}>{t('test.about')}</DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          )}
          description={t('page.description')}
          eyebrow={t('page.eyebrow')}
          title={t('page.title')}
        />

        <div className="mt-7 flex flex-col gap-4">
          {groupsForRender.map((group) => (
            <ProfileGroupCard
              group={group}
              key={group.id}
              movableGroups={movableGroups}
              onDelete={setDeleteTarget}
              onMoveToGroup={(profileId, targetGroupId) => {
                const target = groups.find((candidate) => candidate.id === targetGroupId)
                const moved = moveProfileToGroup(profileId, targetGroupId)
                setMessage(moved ? t('feedback.moved', { group: groupLabel(target) }) : t('feedback.moveSubscription'), moved ? 'good' : 'bad')
              }}
              onRename={(profile) => { setRenameTarget(profile); setRenameValue(profile.name) }}
              onRenameGroup={(target) => { setGroupDialogTarget(target); setGroupDialogOpen(true) }}
              onSelect={(id) => { selectProfile(id); const selected = profilesById.get(id); if (selected) setMessage(t('feedback.selected', { name: selected.name }), 'good') }}
              onTest={runTest}
              onToggle={() => setProfileGroupOpen(group.id, !group.open)}
              profiles={sortedGroupProfiles(group)}
              runningTests={runningTests}
            />
          ))}
        </div>
        <p aria-live="polite" className={`mt-4 min-h-[17px] text-[11px] ${feedbackTone === 'good' ? 'text-good' : feedbackTone === 'bad' ? 'text-bad' : 'text-muted-copy'}`}>{feedback}</p>
        <p className="sr-only">{t('table.available', { count: profiles.length })}</p>
      </div>

      <ProfileGroupDialog key={`${groupDialogTarget?.id ?? 'new'}-${groupDialogOpen}`} group={groupDialogTarget} onOpenChange={(open) => { setGroupDialogOpen(open); if (!open) setGroupDialogTarget(null) }} onSubmit={handleGroupSubmit} open={groupDialogOpen} />

      <Dialog onOpenChange={(open) => { setAddOpen(open); if (!open) setAddError('') }} open={addOpen}>
        <DialogContent className="border-hairline bg-raised text-primary sm:max-w-[480px]">
          <DialogHeader>
            <DialogTitle className="type-display text-2xl text-primary">{t('dialogs.add.title')}</DialogTitle>
            <DialogDescription className="text-[12px] leading-5 text-muted-copy">{t('dialogs.add.description')}</DialogDescription>
          </DialogHeader>
          <form className="space-y-5" onSubmit={submitAdd}>
            <Field>
              <FieldLabel className="text-[12px] text-primary" htmlFor="new-profile-name">{t('dialogs.add.nameLabel')}</FieldLabel>
              <Input autoComplete="off" className="h-[42px] border-hairline bg-surface text-[13px]" id="new-profile-name" onChange={(event) => setName(event.target.value)} placeholder={t('dialogs.add.namePlaceholder')} value={name} />
            </Field>
            <Field>
              <FieldLabel className="text-[12px] text-primary" htmlFor="new-profile-key">{t('dialogs.add.keyLabel')}</FieldLabel>
              <Textarea className="min-h-[86px] resize-y border-hairline bg-surface font-mono text-[11px]" id="new-profile-key" onChange={(event) => setKey(event.target.value)} placeholder={t('dialogs.add.keyPlaceholder')} value={key} />
              <FieldDescription className="text-[11px] text-muted-copy">{t('dialogs.add.helper')}</FieldDescription>
            </Field>
            {addError ? <FieldError className="text-[11px]">{addError}</FieldError> : null}
            <DialogFooter>
              <Button onClick={() => setAddOpen(false)} type="button" variant="ghost">{t('dialogs.add.cancel')}</Button>
              <Button className="bg-lavender text-ink hover:bg-lavender-hi" type="submit">{t('dialogs.add.submit')}</Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

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
    </div>
  )
}
