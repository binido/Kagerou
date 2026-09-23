import { create } from 'zustand'

import { kagerouApi } from '@/lib/tauri-api'
import { TRAFFIC_HISTORY_LIMIT } from '@/types/kagerou'

import { createConnectionSlice } from './slices/connection'
import { createConnectionsSlice } from './slices/connections'
import { appendLog, createLogsSlice } from './slices/logs'
import { createProfilesSlice } from './slices/profiles'
import { createRoutingSlice } from './slices/routing'
import { createSettingsSlice } from './slices/settings'
import { createSharingSlice } from './slices/sharing'
import { createShellSlice } from './slices/shell'
import { createSubscriptionsSlice } from './slices/subscriptions'
import { createTestingSlice } from './slices/testing'
import type { KagerouStore } from './types'

let subscribed = false

/** Test-only: lets each test re-arm event subscription instead of being
 * stuck with whichever mock handler happened to be installed first. */
export const __resetBackendEventSubscriptionForTests = () => {
  subscribed = false
}

export const useKagerouStore = create<KagerouStore>()((...args) => ({
  ...createShellSlice(...args),
  ...createConnectionSlice(...args),
  ...createConnectionsSlice(...args),
  ...createLogsSlice(...args),
  ...createProfilesSlice(...args),
  ...createRoutingSlice(...args),
  ...createSettingsSlice(...args),
  ...createSharingSlice(...args),
  ...createSubscriptionsSlice(...args),
  ...createTestingSlice(...args),
}))

/** What the backend pushes, and what it means for the store.
 *
 * Kept here rather than in a slice because the events cut across them: one
 * of them writes connection state, routing state and the exit location at
 * once. The other half of this list is `usecase::events::AppEvent`.
 */
export const subscribeToBackendEvents = () => {
  if (subscribed) return
  subscribed = true
  const { setState, getState } = useKagerouStore

  // The event fires in the same breath as the backend's own stamp, so
  // `Date.now()` here and `connectedSince` in the snapshot agree to within
  // a millisecond - and the snapshot is what a reload reads, which is the
  // case the frontend clock cannot serve on its own. Going down clears the
  // history so the sparkline stops drawing the previous session's shape.
  void kagerouApi.onConnectionChanged((connected) => {
    setState(
      connected
        ? { connected, connectedSince: Date.now(), rulesChangedSinceConnect: false }
        : {
            connected,
            connectedSince: null,
            trafficHistory: [],
            activeConnections: null,
            liveConnections: [],
          },
    )
    // The exit only exists while the core does, so this is one of the two
    // moments worth asking - the other is a profile switch.
    if (connected) void getState().refreshExitLocation()
  })

  void kagerouApi.onTraffic((event) => {
    if (event.kind !== 'sample') return
    const sample = { download: event.down, upload: event.up }
    setState((state) => ({
      trafficSample: sample,
      trafficHistory: [...state.trafficHistory.slice(-(TRAFFIC_HISTORY_LIMIT - 1)), sample],
      // Same rule as the totals below: a failed `/connections` fetch keeps
      // the previous count rather than blanking the readout.
      activeConnections: event.activeConnections ?? state.activeConnections,
      // A null total means the backend's `/connections` fetch failed for
      // this sample - keep the previous value rather than blanking it.
      sessionTraffic:
        event.downloadTotal !== null && event.uploadTotal !== null
          ? { download: event.downloadTotal, upload: event.uploadTotal }
          : state.sessionTraffic,
    }))
  })

  void kagerouApi.onTestProgress((event) => {
    setState((state) => ({
      testRun: state.testRun
        ? { ...state.testRun, done: event.done, total: event.total }
        : state.testRun,
      profiles: state.profiles.map((profile) =>
        profile.id === event.profileId ? { ...profile, url: event.result } : profile,
      ),
    }))
  })

  void kagerouApi.onTestFinished(() => setState({ testRun: null }))

  // The tray asks for the same two actions the UI offers, and goes through
  // the same code: a second path would be a second set of bugs.
  void kagerouApi.onTrayToggleConnection(() => {
    void getState().toggleConnection()
  })

  void kagerouApi.onTraySelectProfile((profileId) => {
    void getState().selectProfile(profileId)
  })

  void kagerouApi.onLog((line) => {
    setState((state) => ({ logs: appendLog(state.logs, line) }))
  })

  void kagerouApi.onCrashed(() =>
    setState({
      connected: false,
      connectedSince: null,
      trafficHistory: [],
      activeConnections: null,
      liveConnections: [],
    }),
  )
}
