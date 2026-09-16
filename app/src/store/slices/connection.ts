import { kagerouApi } from '@/lib/tauri-api'
import type { ExitLocation, SessionTraffic, TrafficSample } from '@/types/kagerou'

import { report, type Slice } from '../shared'

export interface ConnectionSlice {
  connected: boolean
  /** Unix milliseconds the current connection came up, or `null` when down. */
  connectedSince: number | null
  trafficSample: TrafficSample
  trafficHistory: TrafficSample[]
  /** Live connections reported by the last sample; `null` before the first
   * one arrives or when the Clash API fetch failed. */
  activeConnections: number | null
  sessionTraffic: SessionTraffic
  exitLocation: ExitLocation | null
  exitLocationPending: boolean
  toggleConnection: () => Promise<void>
  refreshExitLocation: () => Promise<void>
}

/** The tunnel: whether it is up, what is going through it, and where it
 * comes out. */
export const createConnectionSlice: Slice<ConnectionSlice> = (set, get) => ({
  connected: false,
  connectedSince: null,
  trafficSample: { download: 0, upload: 0 },
  trafficHistory: [],
  activeConnections: null,
  sessionTraffic: { download: 0, upload: 0 },
  exitLocation: null,
  exitLocationPending: false,

  toggleConnection: async () => {
    const { connected } = get()
    try {
      if (connected) await kagerouApi.disconnect()
      else await kagerouApi.connect()
    } catch (error) {
      report(error, connected ? 'common:feedback.disconnectFailed' : 'common:feedback.connectFailed')
    }
  },

  /** Failure is not worth surfacing: a null location is the signal, and the
   * dashboard falls back to the profile's own flag. */
  refreshExitLocation: async () => {
    set({ exitLocationPending: true })
    try {
      set({ exitLocation: await kagerouApi.lookupExitLocation() })
    } catch (error) {
      console.error('lookupExitLocation failed', error)
      set({ exitLocation: null })
    } finally {
      set({ exitLocationPending: false })
    }
  },
})
