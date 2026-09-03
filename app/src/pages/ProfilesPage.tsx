import { useMemo, useState, type FormEvent } from 'react'
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
import { mockApi } from '@/lib/mock-api'
import { formatProfileCount } from '@/lib/formatters'
import { useKagerouStore } from '@/store/kagerou-store'
import type { Profile, ProfileGroup, TestMethod } from '@/types/kagerou'

export function ProfilesPage() {
  const profiles = useKagerouStore((state) => state.profiles)
  const groups = useKagerouStore((state) => state.profileGroups)
  const selectProfile = useKagerouStore((state) => state.selectProfile)
  const setProfileGroupOpen = useKagerouStore((state) => state.setProfileGroupOpen)
  const addLocalProfile = useKagerouStore((state) => state.addLocalProfile)
  const renameProfile = useKagerouStore((state) => state.renameProfile)
  const addProfileGroup = useKagerouStore((state) => state.addProfileGroup)
  const renameProfileGroup = useKagerouStore((state) => state.renameProfileGroup)
  const deleteProfile = useKagerouStore((state) => state.deleteProfile)
  const moveProfile = useKagerouStore((state) => state.moveProfile)
  const moveProfileToGroup = useKagerouStore((state) => state.moveProfileToGroup)
  const reorderProfiles = useKagerouStore((state) => state.reorderProfiles)
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

  const setMessage = (message: string, tone: 'muted' | 'good' | 'bad' = 'muted') => {
    setFeedback(message)
    setFeedbackTone(tone)
  }

  const runTest = async (profileId: string, method: TestMethod) => {
    const keyName = `${profileId}:${method}`
    setRunningTests((state) => ({ ...state, [keyName]: true }))
    const profile = profilesById.get(profileId)
    setMessage(`${method === 'tcp' ? 'TCP' : 'URL'} test running for ${profile?.name ?? 'profile'}…`)
    const result = await mockApi.runProfileTest(profileId, method)
    setTestResult(profileId, method, result)
    setRunningTests((state) => {
      const next = { ...state }
      delete next[keyName]
      return next
    })
    setMessage(`${method === 'tcp' ? 'TCP' : 'URL'} test finished for ${profile?.name ?? 'profile'}: ${result.value}.`, result.tone === 'bad' ? 'bad' : 'good')
  }

  const runAll = (method: TestMethod) => {
    void Promise.all(profiles.map((profile) => runTest(profile.id, method)))
    setMessage(`${method === 'tcp' ? 'TCP' : 'URL'} test running for all profiles…`)
  }

  const submitAdd = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const trimmedName = name.trim()
    const trimmedKey = key.trim()
    if (!trimmedName) {
      setAddError('Enter a profile name.')
      return
    }
    if (!/^(vless|vmess|trojan|ss|hysteria2):\/\/[^\s]+$/i.test(trimmedKey)) {
      setAddError('Paste a supported profile key such as vless://…')
      return
    }
    addLocalProfile({ key: trimmedKey, name: trimmedName })
    setName('')
    setKey('')
    setAddError('')
    setAddOpen(false)
    setMessage(`${trimmedName} added to Default.`, 'good')
  }

  const submitRename = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const trimmed = renameValue.trim()
    if (!renameTarget) return
    if (!trimmed) {
      setMessage('Profile name cannot be empty.', 'bad')
      return
    }
    const renamed = renameProfile(renameTarget.id, trimmed)
    if (!renamed) {
      setMessage('Only local profiles can be renamed.', 'bad')
      return
    }
    setMessage(`Profile renamed to ${trimmed}.`, 'good')
    setRenameTarget(null)
  }

  const handleGroupSubmit = (label: string) => {
    if (groupDialogTarget) {
      const updated = renameProfileGroup(groupDialogTarget.id, label)
      if (updated) setMessage(`Group renamed to ${label}.`, 'good')
      return updated
    }
    const id = addProfileGroup(label)
    if (id) setMessage(`${label} group created.`, 'good')
    return Boolean(id)
  }

  const groupsForRender = groups
  const movableGroups = groups.filter((group) => group.kind !== 'subscription')

  return (
    <div className="min-h-screen min-w-0 bg-canvas px-6 pb-10 pt-8 lg:px-12 lg:pt-10">
      <div className="mx-auto w-full max-w-[1040px]">
        <PageHeader
          actions={(
            <div className="flex flex-wrap justify-end gap-2">
              <Button className="h-10 gap-2 border-hairline bg-surface px-3.5 text-[12px] text-body hover:bg-raised hover:text-primary" onClick={() => { setGroupDialogTarget(null); setGroupDialogOpen(true) }} type="button" variant="outline"><FolderPlus aria-hidden="true" className="size-4" />Add group</Button>
              <Button className="h-10 gap-2 bg-lavender px-3.5 text-[12px] font-semibold text-ink hover:bg-lavender-hi" onClick={() => setAddOpen(true)} type="button"><Plus aria-hidden="true" className="size-4" />Add single key</Button>
              <DropdownMenu>
                <DropdownMenuTrigger asChild><Button className="h-10 gap-2 border-hairline bg-surface px-3.5 text-[12px] text-body hover:bg-raised hover:text-primary" type="button" variant="outline"><Radar aria-hidden="true" className="size-4" />Run test<ChevronDown aria-hidden="true" className="size-3 text-muted-copy" /></Button></DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-48 border-hairline bg-popover text-[11px]">
                  <DropdownMenuLabel className="font-mono text-[9px] uppercase tracking-[0.08em] text-muted-copy">All profiles</DropdownMenuLabel>
                  <DropdownMenuItem onSelect={() => runAll('tcp')}><Loader2 aria-hidden="true" className="size-3.5" />TCP connection</DropdownMenuItem>
                  <DropdownMenuItem onSelect={() => runAll('url')}><Radar aria-hidden="true" className="size-3.5" />URL check</DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem onSelect={() => { setMessage('Diagnostics use mock service data in this prototype.') }}>About diagnostics</DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          )}
          description="Organize single keys into local groups while subscriptions stay together."
          eyebrow="Kagerou  /  Connection profiles"
          title="Profiles"
        />

        <div className="mt-7 flex flex-col gap-4">
          {groupsForRender.map((group) => (
            <ProfileGroupCard
              group={group}
              key={group.id}
              movableGroups={movableGroups}
              onDelete={setDeleteTarget}
              onMove={(id, direction) => { const moved = moveProfile(id, direction); setMessage(moved ? `Profile moved ${direction}.` : 'Profile is already at the edge of its group.', moved ? 'good' : 'muted') }}
              onMoveToGroup={(profileId, targetGroupId) => {
                const target = groups.find((candidate) => candidate.id === targetGroupId)
                const moved = moveProfileToGroup(profileId, targetGroupId)
                setMessage(moved ? `Profile moved to ${target?.label ?? 'group'}.` : 'Subscription profiles cannot be moved to another group.', moved ? 'good' : 'bad')
              }}
              onRename={(profile) => { setRenameTarget(profile); setRenameValue(profile.name) }}
              onRenameGroup={(target) => { setGroupDialogTarget(target); setGroupDialogOpen(true) }}
              onReorder={(fromId, toId) => { const moved = reorderProfiles(fromId, toId); setMessage(moved ? 'Profile order updated.' : 'Profiles can only be reordered within their group.', moved ? 'good' : 'bad') }}
              onSelect={(id) => { selectProfile(id); const selected = profilesById.get(id); if (selected) setMessage(`${selected.name} is ready for the next connection.`, 'good') }}
              onTest={runTest}
              onToggle={() => setProfileGroupOpen(group.id, !group.open)}
              profiles={visibleGroupProfiles(group)}
              runningTests={runningTests}
            />
          ))}
        </div>
        <p aria-live="polite" className={`mt-4 min-h-[17px] text-[11px] ${feedbackTone === 'good' ? 'text-good' : feedbackTone === 'bad' ? 'text-bad' : 'text-muted-copy'}`}>{feedback}</p>
        <p className="sr-only">{formatProfileCount(profiles.length)} available.</p>
      </div>

      <ProfileGroupDialog key={`${groupDialogTarget?.id ?? 'new'}-${groupDialogOpen}`} group={groupDialogTarget} onOpenChange={(open) => { setGroupDialogOpen(open); if (!open) setGroupDialogTarget(null) }} onSubmit={handleGroupSubmit} open={groupDialogOpen} />

      <Dialog onOpenChange={(open) => { setAddOpen(open); if (!open) setAddError('') }} open={addOpen}>
        <DialogContent className="border-[#3b3a45] bg-raised text-primary sm:max-w-[480px]">
          <DialogHeader>
            <DialogTitle className="type-display text-2xl text-primary">Add single key</DialogTitle>
            <DialogDescription className="text-[12px] leading-5 text-muted-copy">Create a local profile in Default. You can move it to a custom group later.</DialogDescription>
          </DialogHeader>
          <form className="space-y-5" onSubmit={submitAdd}>
            <Field>
              <FieldLabel className="text-[12px] text-primary" htmlFor="new-profile-name">Profile name</FieldLabel>
              <Input autoComplete="off" className="h-[42px] border-white/10 bg-surface text-[13px]" id="new-profile-name" onChange={(event) => setName(event.target.value)} placeholder="e.g. Home · Seattle" value={name} />
            </Field>
            <Field>
              <FieldLabel className="text-[12px] text-primary" htmlFor="new-profile-key">Profile key</FieldLabel>
              <Textarea className="min-h-[86px] resize-y border-white/10 bg-surface font-mono text-[11px]" id="new-profile-key" onChange={(event) => setKey(event.target.value)} placeholder="vless://…" value={key} />
              <FieldDescription className="text-[11px] text-muted-copy">Single keys always start in Default and can be moved between local groups.</FieldDescription>
            </Field>
            {addError ? <FieldError className="text-[11px]">{addError}</FieldError> : null}
            <DialogFooter>
              <Button onClick={() => setAddOpen(false)} type="button" variant="ghost">Cancel</Button>
              <Button className="bg-lavender text-ink hover:bg-lavender-hi" type="submit">Add key</Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <Dialog onOpenChange={(open) => { if (!open) setRenameTarget(null) }} open={Boolean(renameTarget)}>
        <DialogContent className="border-[#3b3a45] bg-raised text-primary sm:max-w-[420px]">
          <DialogHeader><DialogTitle className="type-display text-2xl text-primary">Rename profile</DialogTitle><DialogDescription className="text-[12px] text-muted-copy">Update the local display name without changing its connection key.</DialogDescription></DialogHeader>
          <form className="space-y-5" onSubmit={submitRename}>
            <Field><FieldLabel className="text-[12px] text-primary" htmlFor="rename-profile">Profile name</FieldLabel><Input autoFocus className="h-[42px] border-white/10 bg-surface text-[13px]" id="rename-profile" onChange={(event) => setRenameValue(event.target.value)} value={renameValue} /></Field>
            <DialogFooter><Button onClick={() => setRenameTarget(null)} type="button" variant="ghost">Cancel</Button><Button className="bg-lavender text-ink hover:bg-lavender-hi" type="submit"><Check aria-hidden="true" className="size-3.5" />Save name</Button></DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <AlertDialog onOpenChange={(open) => { if (!open) setDeleteTarget(null) }} open={Boolean(deleteTarget)}>
        <AlertDialogContent className="border-[#3b3a45] bg-raised text-primary sm:max-w-[440px]">
          <AlertDialogHeader><AlertDialogTitle className="type-display text-2xl text-primary">Delete profile?</AlertDialogTitle><AlertDialogDescription className="text-[12px] leading-5 text-muted-copy">This removes the local profile from its group.</AlertDialogDescription></AlertDialogHeader>
          <div className="border-l-2 border-bad bg-bad/10 px-3 py-2.5 text-[12px] leading-5 text-body">You are about to delete <strong className="text-primary">{deleteTarget?.name}</strong>. Subscription profiles remain managed by their source.</div>
          <AlertDialogFooter><AlertDialogCancel onClick={() => setDeleteTarget(null)}>Cancel</AlertDialogCancel><AlertDialogAction className="bg-bad text-primary hover:bg-bad/85" onClick={() => { if (!deleteTarget) return; const nameToDelete = deleteTarget.name; deleteProfile(deleteTarget.id); setDeleteTarget(null); setMessage(`${nameToDelete} deleted from local profiles.`, 'good') }}>Delete profile</AlertDialogAction></AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
