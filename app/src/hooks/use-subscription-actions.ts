import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

import { backendErrorMessage } from '@/lib/errors'
import { describeUnsupported } from '@/lib/unsupported'
import { useKagerouStore } from '@/store/kagerou-store'
import type { ProfileGroup, Source } from '@/types/kagerou'

interface Options {
  groups: ProfileGroup[]
  groupLabel: (group?: ProfileGroup) => string
}

/** What a subscription group's menu offers: refresh, change the URL, copy
 * it, delete the whole thing. */
export function useSubscriptionActions({ groups, groupLabel }: Options) {
  const { t } = useTranslation('profiles')
  const connected = useKagerouStore((state) => state.connected)
  const activeProfileId = useKagerouStore((state) => state.activeProfileId)
  const updateSource = useKagerouStore((state) => state.updateSource)
  const refreshSource = useKagerouStore((state) => state.refreshSource)
  const deleteSubscription = useKagerouStore((state) => state.deleteSubscription)
  const [refreshing, setRefreshing] = useState<Record<string, boolean>>({})
  const [urlTarget, setUrlTarget] = useState<Source | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<ProfileGroup | null>(null)

  const refresh = async (group: ProfileGroup | undefined, source: Source) => {
    if (refreshing[source.id]) return
    const name = groupLabel(group)
    setRefreshing((state) => ({ ...state, [source.id]: true }))
    const toastId = toast.loading(t('feedback.refreshing', { name }))
    try {
      const unsupported = await refreshSource(source.id)
      toast.success(t('feedback.refreshed', { name }), {
        id: toastId,
        description: describeUnsupported(unsupported),
      })
    } catch (error) {
      toast.error(backendErrorMessage(error, t('feedback.refreshFailed')), { id: toastId })
    } finally {
      setRefreshing((state) => {
        const next = { ...state }
        delete next[source.id]
        return next
      })
    }
  }

  const submitUrl = async (url: string) => {
    const target = urlTarget
    if (!target || !(await updateSource(target.id, { value: url }))) return false
    setUrlTarget(null)
    void refresh(
      groups.find((group) => group.sourceId === target.id),
      target,
    )
    return true
  }

  const copyUrl = async (source: Source) => {
    try {
      await navigator.clipboard.writeText(source.value)
      toast.success(t('feedback.urlCopied'))
    } catch {
      toast.error(t('feedback.urlCopyFailed'))
    }
  }

  // Checked here only to say it in the user's language before a dialog opens;
  // the backend refuses the same case on its own.
  const askToDelete = (group: ProfileGroup) => {
    if (connected && group.profileIds.includes(activeProfileId)) {
      toast.error(t('feedback.subscriptionInUse', { group: groupLabel(group) }))
      return
    }
    setDeleteTarget(group)
  }

  const confirmDelete = async () => {
    const target = deleteTarget
    if (!target) return
    setDeleteTarget(null)
    if (await deleteSubscription(target.id))
      toast.success(t('feedback.subscriptionDeleted', { group: groupLabel(target) }))
  }

  return {
    isRefreshing: (source?: Source) => Boolean(source && refreshing[source.id]),
    refresh,
    urlTarget,
    openUrlDialog: setUrlTarget,
    dismissUrlDialog: () => setUrlTarget(null),
    submitUrl,
    copyUrl,
    deleteTarget,
    askToDelete,
    confirmDelete,
    dismissDeleteTarget: () => setDeleteTarget(null),
  }
}
