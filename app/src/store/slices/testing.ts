import { kagerouApi } from '@/lib/tauri-api'
import type { TestResult, TestRun } from '@/types/kagerou'

import { refresh, report, type Slice } from '../shared'

export interface TestingSlice {
  testRun: TestRun | null
  runProfileTest: (id: string) => Promise<TestResult | null>
  startGroupTest: (groupId: string | null) => Promise<void>
  cancelGroupTest: () => Promise<void>
  clearGroupTestResults: (groupId: string) => Promise<void>
  deleteUnavailableProfiles: (groupId: string) => Promise<number>
}

/** Measuring VPNs. A group run is the backend's, and reports back by event;
 * this side only starts it, stops it, and shows what arrives. */
export const createTestingSlice: Slice<TestingSlice> = (set) => ({
  testRun: null,

  runProfileTest: async (id) => {
    try {
      const result = await kagerouApi.runProfileTest(id)
      set((state) => ({
        profiles: state.profiles.map((profile) =>
          profile.id === id ? { ...profile, url: result } : profile,
        ),
      }))
      return result
    } catch (error) {
      console.error('runProfileTest failed', error)
      return null
    }
  },

  startGroupTest: async (groupId) => {
    // Optimistic so the progress bar appears on the click rather than after
    // the first profile has been measured, which can be seconds later.
    set({ testRun: { groupId, done: 0, total: 0 } })
    try {
      const total = await kagerouApi.startGroupTest(groupId)
      if (total === 0) set({ testRun: null })
    } catch (error) {
      report(error, 'common:feedback.groupTestStartFailed')
      set({ testRun: null })
    }
  },

  cancelGroupTest: async () => {
    try {
      await kagerouApi.cancelGroupTest()
    } catch (error) {
      report(error, 'common:feedback.groupTestCancelFailed')
    }
  },

  clearGroupTestResults: async (groupId) => {
    try {
      await kagerouApi.clearGroupTestResults(groupId)
      await refresh(set)
    } catch (error) {
      report(error, 'common:feedback.groupTestClearFailed')
    }
  },

  deleteUnavailableProfiles: async (groupId) => {
    try {
      const deleted = await kagerouApi.deleteUnavailableProfiles(groupId)
      await refresh(set)
      return deleted
    } catch (error) {
      report(error, 'common:feedback.deleteUnavailableFailed')
      return 0
    }
  },
})
