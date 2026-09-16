import { useState } from 'react'
import { useTranslation } from 'react-i18next'

import type { RemoveUnavailableTarget } from '@/components/profiles/RemoveUnavailableDialog'
import { isUnreachable, resultLabel } from '@/lib/result-copy'
import { useKagerouStore } from '@/store/kagerou-store'
import type { Profile, ProfileGroup } from '@/types/kagerou'

interface Options {
  profilesById: Map<string, Profile>
  groupLabel: (group?: ProfileGroup) => string
  say: (text: string, tone?: 'muted' | 'good' | 'bad') => void
}

/** Measuring profiles, and clearing out the ones that did not answer. */
export function useProfileTesting({ profilesById, groupLabel, say }: Options) {
  const { t } = useTranslation('profiles')
  const { t: tc } = useTranslation('common')
  const runProfileTest = useKagerouStore((state) => state.runProfileTest)
  const startGroupTest = useKagerouStore((state) => state.startGroupTest)
  const clearGroupTestResults = useKagerouStore((state) => state.clearGroupTestResults)
  const deleteUnavailableProfiles = useKagerouStore((state) => state.deleteUnavailableProfiles)
  const testRun = useKagerouStore((state) => state.testRun)
  const [running, setRunning] = useState<Record<string, boolean>>({})
  const [removeTarget, setRemoveTarget] = useState<RemoveUnavailableTarget | null>(null)

  const testOne = async (profileId: string) => {
    const name = profilesById.get(profileId)?.name ?? t('fallback.vpn')
    setRunning((state) => ({ ...state, [profileId]: true }))
    say(t('feedback.testRunning', { name }))
    const result = await runProfileTest(profileId)
    setRunning((state) => {
      const next = { ...state }
      delete next[profileId]
      return next
    })
    if (!result) {
      say(t('feedback.testFailed', { name }), 'bad')
      return
    }
    say(
      t('feedback.testFinished', { name, value: resultLabel(result, tc) }),
      result.tone === 'bad' ? 'bad' : 'good',
    )
  }

  const testEverything = () => {
    void startGroupTest(null)
    say(t('feedback.testRunningAll'))
  }

  const testGroup = (group: ProfileGroup) => {
    void startGroupTest(group.id)
    say(t('feedback.testRunningGroup', { group: groupLabel(group) }))
  }

  // A run holds the test core's selector, so nothing else may be aimed at it.
  const groupIsBusy = (group: ProfileGroup) =>
    Boolean(testRun) || group.profileIds.some((id) => running[id])

  const clearResults = (group: ProfileGroup) => {
    void clearGroupTestResults(group.id)
    say(t('feedback.resultsCleared', { group: groupLabel(group) }))
  }

  // The count excludes the active profile, which is kept, and the dialog
  // never opens when nothing failed. The backend re-evaluates the same
  // predicate in its transaction, so a stale count here is only cosmetic.
  const askToRemoveUnavailable = (group: ProfileGroup) => {
    const failing = group.profileIds.filter((id) => {
      const result = profilesById.get(id)?.url
      return result ? isUnreachable(result) : false
    })
    if (failing.length === 0) {
      say(t('feedback.nothingUnavailable', { group: groupLabel(group) }))
      return
    }
    const activeKept = failing.some((id) => profilesById.get(id)?.selected)
    setRemoveTarget({
      groupId: group.id,
      groupLabel: groupLabel(group),
      count: failing.length - (activeKept ? 1 : 0),
      activeKept,
    })
  }

  const confirmRemoveUnavailable = async () => {
    if (!removeTarget) return
    const target = removeTarget
    const deleted = await deleteUnavailableProfiles(target.groupId)
    setRemoveTarget(null)
    const message = t('feedback.deletedUnavailable', { count: deleted, group: target.groupLabel })
    say(target.activeKept ? `${message} ${t('feedback.activeKept')}` : message, 'good')
  }

  return {
    running,
    testOne,
    testEverything,
    testGroup,
    groupIsBusy,
    clearResults,
    removeTarget,
    askToRemoveUnavailable,
    confirmRemoveUnavailable,
    dismissRemoveTarget: () => setRemoveTarget(null),
  }
}
