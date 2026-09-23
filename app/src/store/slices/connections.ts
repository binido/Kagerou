import { kagerouApi } from '@/lib/tauri-api'
import type { LiveConnection } from '@/types/kagerou'

import { report, type Slice } from '../shared'

/** Unix milliseconds of a Clash API start time.
 *
 * The fraction is cut to milliseconds first. Go writes up to nine digits,
 * and WebKit's `Date.parse` is not obliged to read more than three. */
export const connectionStartMillis = (start: string): number =>
  Date.parse(start.replace(/(\.\d{3})\d+/, '$1'))

export interface ConnectionsSlice {
  /** The core's open connections, newest first. */
  liveConnections: LiveConnection[]
  refreshConnections: () => Promise<void>
  closeConnection: (id: string) => Promise<void>
  closeAllConnections: () => Promise<void>
}

export const createConnectionsSlice: Slice<ConnectionsSlice> = (set, get) => ({
  liveConnections: [],

  // Polled every second while the page is open, so a failed read keeps the
  // previous list instead of raising a toast per tick.
  refreshConnections: async () => {
    try {
      const listed = await kagerouApi.listConnections()
      set({
        liveConnections: [...listed].sort(
          (a, b) => connectionStartMillis(b.start) - connectionStartMillis(a.start),
        ),
      })
    } catch {
      // The next tick asks again.
    }
  },

  closeConnection: async (id) => {
    set((state) => ({
      liveConnections: state.liveConnections.filter((connection) => connection.id !== id),
    }))
    try {
      await kagerouApi.closeConnection(id)
    } catch (error) {
      report(error, 'connections:feedback.closeFailed')
      await get().refreshConnections()
    }
  },

  closeAllConnections: async () => {
    set({ liveConnections: [] })
    try {
      await kagerouApi.closeAllConnections()
    } catch (error) {
      report(error, 'connections:feedback.closeAllFailed')
      await get().refreshConnections()
    }
  },
})
