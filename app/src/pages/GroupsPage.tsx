import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ClipboardPaste, FolderPlus, Radar } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { PageContainer } from '@/components/layout/PageContainer'
import { PageHeader } from '@/components/layout/PageHeader'
import { DeleteProfileDialog } from '@/components/profiles/DeleteProfileDialog'
import { DeleteSubscriptionDialog } from '@/components/profiles/DeleteSubscriptionDialog'
import { ImportDialog } from '@/components/profiles/ImportDialog'
import { ProfileGroupCard } from '@/components/profiles/ProfileGroupCard'
import { ProfileGroupDialog } from '@/components/profiles/ProfileGroupDialog'
import { RemoveUnavailableDialog } from '@/components/profiles/RemoveUnavailableDialog'
import { RenameProfileDialog } from '@/components/profiles/RenameProfileDialog'
import { SubscriptionUrlDialog } from '@/components/profiles/SubscriptionUrlDialog'
import { TestRunBar } from '@/components/profiles/TestRunBar'
import { useProfileImport } from '@/hooks/use-profile-import'
import { useProfileTesting } from '@/hooks/use-profile-testing'
import { useStatusMessage } from '@/hooks/use-status-message'
import { useSubscriptionActions } from '@/hooks/use-subscription-actions'
import { sortProfiles } from '@/lib/profile-sorting'
import { useKagerouStore } from '@/store/kagerou-store'
import type { Profile, ProfileGroup } from '@/types/kagerou'

const toneClasses = {
  good: 'text-good',
  bad: 'text-bad',
  muted: 'text-muted-copy',
} as const

export function GroupsPage() {
  const { t } = useTranslation('profiles')
  const profiles = useKagerouStore((state) => state.profiles)
  const groups = useKagerouStore((state) => state.profileGroups)
  const sources = useKagerouStore((state) => state.sources)
  const groupSort = useKagerouStore((state) => state.settings.groupSort)
  const selectProfile = useKagerouStore((state) => state.selectProfile)
  const setProfileGroupOpen = useKagerouStore((state) => state.setProfileGroupOpen)
  const renameProfile = useKagerouStore((state) => state.renameProfile)
  const deleteProfile = useKagerouStore((state) => state.deleteProfile)
  const moveProfileToGroup = useKagerouStore((state) => state.moveProfileToGroup)
  const openSupportUrl = useKagerouStore((state) => state.openSupportUrl)
  const addProfileGroup = useKagerouStore((state) => state.addProfileGroup)
  const renameProfileGroup = useKagerouStore((state) => state.renameProfileGroup)
  const testRun = useKagerouStore((state) => state.testRun)
  const cancelGroupTest = useKagerouStore((state) => state.cancelGroupTest)

  const [deleteTarget, setDeleteTarget] = useState<Profile | null>(null)
  const [renameTarget, setRenameTarget] = useState<Profile | null>(null)
  const [groupDialogOpen, setGroupDialogOpen] = useState(false)
  const [groupDialogTarget, setGroupDialogTarget] = useState<ProfileGroup | null>(null)

  const profilesById = useMemo(
    () => new Map(profiles.map((profile) => [profile.id, profile])),
    [profiles],
  )
  const sourcesById = useMemo(
    () => new Map(sources.map((source) => [source.id, source])),
    [sources],
  )

  const groupLabel = (group?: ProfileGroup) =>
    group?.kind === 'default' ? t('group.defaultName') : (group?.label ?? t('fallback.group'))
  // Reads from the store rather than this render: an import has just
  // refreshed the groups, and a new one is not in the props yet.
  const labelOfId = (groupId: string) =>
    groupLabel(useKagerouStore.getState().profileGroups.find((group) => group.id === groupId))

  const { message, say } = useStatusMessage()
  const imports = useProfileImport(labelOfId)
  const testing = useProfileTesting({ profilesById, groupLabel, say })
  const subscriptions = useSubscriptionActions({ groups, groupLabel })

  const submitRename = async (name: string) => {
    if (!renameTarget) return null
    if (!name) {
      say(t('feedback.vpnNameEmpty'), 'bad')
      return null
    }
    if (!(await renameProfile(renameTarget.id, name))) {
      say(t('feedback.renameLocalOnly'), 'bad')
      return null
    }
    say(t('feedback.renamed', { name }), 'good')
    setRenameTarget(null)
    return name
  }

  const submitGroup = async (label: string) => {
    if (groupDialogTarget) {
      const renamed = await renameProfileGroup(groupDialogTarget.id, label)
      if (renamed) say(t('feedback.groupRenamed', { name: label }), 'good')
      return renamed
    }
    const id = await addProfileGroup(label)
    if (id) say(t('feedback.groupCreated', { name: label }), 'good')
    return Boolean(id)
  }

  const confirmDelete = (profile: Profile) => {
    deleteProfile(profile.id)
    setDeleteTarget(null)
    say(t('feedback.deleted', { name: profile.name }), 'good')
  }

  const moveToGroup = async (profileId: string, targetGroupId: string) => {
    const target = groups.find((candidate) => candidate.id === targetGroupId)
    const moved = await moveProfileToGroup(profileId, targetGroupId)
    say(
      moved ? t('feedback.moved', { group: groupLabel(target) }) : t('feedback.moveSubscription'),
      moved ? 'good' : 'bad',
    )
  }

  const select = (id: string) => {
    void selectProfile(id)
    const selected = profilesById.get(id)
    if (selected) say(t('feedback.selected', { name: selected.name }), 'good')
  }

  const movableGroups = groups.filter((group) => group.kind !== 'subscription')
  const visibleProfiles = (group: ProfileGroup) =>
    sortProfiles(
      group.profileIds
        .map((id) => profilesById.get(id))
        .filter((profile): profile is Profile => Boolean(profile)),
      groupSort,
    )

  return (
    <PageContainer>
      <PageHeader
        actions={
          <div className="flex flex-wrap justify-end gap-2">
            <Button
              className="h-10 gap-2 border-hairline bg-surface px-3.5 text-[12px] text-body hover:bg-raised hover:text-primary"
              onClick={() => {
                setGroupDialogTarget(null)
                setGroupDialogOpen(true)
              }}
              type="button"
              variant="outline"
            >
              <FolderPlus aria-hidden="true" className="size-4" />
              {t('actions.addGroup')}
            </Button>
            <Button
              className="h-10 gap-2 bg-lavender px-3.5 text-[12px] font-semibold text-ink hover:bg-lavender-hi"
              disabled={imports.importing}
              onClick={imports.fromClipboard}
              type="button"
            >
              <ClipboardPaste aria-hidden="true" className="size-4" />
              {t('actions.addFromClipboard')}
            </Button>
            <Button
              className="h-10 gap-2 border-hairline bg-surface px-3.5 text-[12px] text-body hover:bg-raised hover:text-primary"
              onClick={testing.testEverything}
              type="button"
              variant="outline"
            >
              <Radar aria-hidden="true" className="size-4" />
              {t('actions.runTest')}
            </Button>
          </div>
        }
        description={t('page.description')}
        eyebrow={t('page.eyebrow')}
        title={t('page.title')}
      />

      {testRun ? (
        <TestRunBar
          onCancel={() => {
            void cancelGroupTest()
          }}
          run={testRun}
        />
      ) : null}

      <div className="mt-7 flex flex-col gap-4">
        {groups.map((group) => {
          const source = group.sourceId ? sourcesById.get(group.sourceId) : undefined
          return (
            <ProfileGroupCard
              group={group}
              key={group.id}
              movableGroups={movableGroups}
              onChangeUrl={() => {
                if (source) subscriptions.openUrlDialog(source)
              }}
              onClearResults={() => testing.clearResults(group)}
              onCopyUrl={() => {
                if (source) void subscriptions.copyUrl(source)
              }}
              onDelete={setDeleteTarget}
              onDeleteSubscription={() => subscriptions.askToDelete(group)}
              onOpenSupport={() => {
                if (source) void openSupportUrl(source.id)
              }}
              onDeleteUnavailable={() => testing.askToRemoveUnavailable(group)}
              onMoveToGroup={(profileId, targetGroupId) => {
                void moveToGroup(profileId, targetGroupId)
              }}
              onRefresh={() => {
                if (source) void subscriptions.refresh(group, source)
              }}
              onRename={setRenameTarget}
              onRenameGroup={(target) => {
                setGroupDialogTarget(target)
                setGroupDialogOpen(true)
              }}
              onSelect={select}
              onTest={testing.testOne}
              onTestGroup={() => testing.testGroup(group)}
              onToggle={() => setProfileGroupOpen(group.id, !group.open)}
              profiles={visibleProfiles(group)}
              refreshing={subscriptions.isRefreshing(source)}
              runningTests={testing.running}
              source={source}
              testRunning={testing.groupIsBusy(group)}
            />
          )
        })}
      </div>

      <p
        aria-live="polite"
        className={`mt-4 min-h-[17px] text-[11px] ${toneClasses[message.tone]}`}
      >
        {message.text}
      </p>
      <p className="sr-only">{t('table.available', { count: profiles.length })}</p>

      <ProfileGroupDialog
        group={groupDialogTarget}
        onOpenChange={(open) => {
          setGroupDialogOpen(open)
          if (!open) setGroupDialogTarget(null)
        }}
        onSubmit={submitGroup}
        open={groupDialogOpen}
      />
      <RemoveUnavailableDialog
        onConfirm={() => {
          void testing.confirmRemoveUnavailable()
        }}
        onOpenChange={(open) => {
          if (!open) testing.dismissRemoveTarget()
        }}
        target={testing.removeTarget}
      />
      <ImportDialog
        draft={imports.draft}
        onOpenChange={(open) => {
          if (!open) imports.dismissDraft()
        }}
        onSubmit={imports.fromText}
        submitting={imports.importing}
      />
      <SubscriptionUrlDialog
        onOpenChange={(open) => {
          if (!open) subscriptions.dismissUrlDialog()
        }}
        onSubmit={subscriptions.submitUrl}
        source={subscriptions.urlTarget}
      />
      <DeleteSubscriptionDialog
        group={subscriptions.deleteTarget}
        onConfirm={() => {
          void subscriptions.confirmDelete()
        }}
        onOpenChange={(open) => {
          if (!open) subscriptions.dismissDeleteTarget()
        }}
      />
      <RenameProfileDialog
        onOpenChange={(open) => {
          if (!open) setRenameTarget(null)
        }}
        onSubmit={submitRename}
        profile={renameTarget}
      />
      <DeleteProfileDialog
        onConfirm={confirmDelete}
        onOpenChange={(open) => {
          if (!open) setDeleteTarget(null)
        }}
        profile={deleteTarget}
      />
    </PageContainer>
  )
}
