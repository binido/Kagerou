import { toast } from 'sonner'
import type { StateCreator } from 'zustand'

import i18n from '@/i18n'
import { backendErrorMessage } from '@/lib/errors'
import { kagerouApi, type AppSnapshot } from '@/lib/tauri-api'

import type { KagerouStore } from './types'

/** Every slice is written against the whole store, so one can call another's
 * actions through `get()`. */
export type Slice<T> = StateCreator<KagerouStore, [], [], T>

type Setter = (partial: Partial<KagerouStore>) => void

export const applySnapshot = (snapshot: AppSnapshot) => ({
  connected: snapshot.connected,
  connectedSince: snapshot.connectedSince,
  activeProfileId: snapshot.activeProfileId,
  profiles: snapshot.profiles,
  profileGroups: snapshot.profileGroups,
  sources: snapshot.sources,
  routingPresets: snapshot.routingPresets,
  routingRules: snapshot.routingRules,
  settings: snapshot.settings,
})

/** Re-reads everything from the backend.
 *
 * Used as the rollback for an optimistic write: the snapshot is what the
 * database actually accepted, so it cannot drift from it the way a captured
 * previous value can. */
export const refresh = async (set: Setter) => {
  set(applySnapshot(await kagerouApi.getAppState()))
}

/** How every failed action reports itself. The backend answers with a code,
 * which is what picks the sentence and its language; the key names what was
 * being attempted, for the cases the backend has no code for. */
export const report = (error: unknown, fallbackKey: string) => {
  toast.error(backendErrorMessage(error, i18n.t(fallbackKey as never)))
}
