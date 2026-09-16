import i18n from '@/i18n'
import { backendErrorMessage } from '@/lib/errors'
import { kagerouApi } from '@/lib/tauri-api'
import type { ImportAttempt, Source } from '@/types/kagerou'

import { refresh, report, type Slice } from '../shared'

export interface SubscriptionsSlice {
  sources: Source[]
  importText: (text: string) => Promise<ImportAttempt>
  importFromClipboard: () => Promise<ImportAttempt>
  updateSource: (id: string, patch: Partial<Pick<Source, 'name' | 'value'>>) => Promise<boolean>
  refreshSource: (id: string) => Promise<void>
  deleteSubscription: (groupId: string) => Promise<boolean>
}

/** Getting VPNs in, and keeping the subscriptions they came from current. */
export const createSubscriptionsSlice: Slice<SubscriptionsSlice> = (set, get) => ({
  sources: [],

  // Waits for the backend: only it knows what the text turns into. A failure
  // hands the text back rather than a toast, so the page can offer it for
  // correction.
  importText: async (text) => {
    try {
      const outcome = await kagerouApi.importFromText(text)
      await refresh(set)
      return { status: 'imported', outcome }
    } catch (error) {
      return {
        status: 'failed',
        text,
        error: backendErrorMessage(error, i18n.t('common:feedback.importFailed')),
      }
    }
  },

  importFromClipboard: async () => {
    let text = ''
    try {
      text = await kagerouApi.readClipboardText()
    } catch {
      // An empty clipboard rejects too; either way there is nothing to read
      // and the manual paste dialog takes over.
    }
    if (!text.trim()) return { status: 'failed', text: '', error: '' }
    return get().importText(text)
  },

  updateSource: async (id, patch) => {
    try {
      await kagerouApi.updateSource(id, patch)
      await refresh(set)
      return true
    } catch {
      return false
    }
  },

  // The one action that lets its rejection through: the page it is called
  // from puts the reason in the toast it already opened.
  refreshSource: async (id) => {
    await kagerouApi.refreshSource(id)
    await refresh(set)
  },

  deleteSubscription: async (groupId) => {
    try {
      await kagerouApi.deleteSubscription(groupId)
      await refresh(set)
      return true
    } catch (error) {
      report(error, 'common:feedback.subscriptionDeleteFailed')
      return false
    }
  },
})
