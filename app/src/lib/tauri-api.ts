import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

import type {
  AddLocalProfileInput,
  AddSourceInput,
  Profile,
  ProfileGroup,
  RoutingPreset,
  RoutingRule,
  SettingsState,
  Source,
  TestResult,
  UpdateInfo,
} from '@/types/kagerou'

export interface AppSnapshot {
  activeProfileId: string
  profiles: Profile[]
  profileGroups: ProfileGroup[]
  sources: Source[]
  routingPresets: RoutingPreset[]
  routingRules: RoutingRule[]
  settings: SettingsState
}

/** Session-wide byte totals piggybacked on each traffic sample (sourced
 * from the Clash API's `/connections`); `null` when that fetch failed and
 * the UI should keep showing the previous values. */
export type TrafficEvent =
  | { kind: 'sample'; up: number; down: number; uploadTotal: number | null; downloadTotal: number | null }
  | { kind: 'disconnected' }
  | { kind: 'reconnecting' }

/** Thin, typed wrapper around `invoke`/`listen` — the only place in the
 * frontend that knows the Tauri command/event names, so the store (and any
 * tests) can depend on this instead of scattering string literals. */
export const kagerouApi = {
  getAppState: () => invoke<AppSnapshot>('get_app_state'),
  checkForUpdate: () => invoke<UpdateInfo | null>('check_for_update'),
  connect: () => invoke<void>('connect'),
  disconnect: () => invoke<void>('disconnect'),

  selectProfile: (id: string) => invoke<void>('select_profile', { id }),
  addLocalProfile: (input: AddLocalProfileInput) => invoke<string>('add_local_profile', { input }),
  renameProfile: (id: string, name: string) => invoke<void>('rename_profile', { id, name }),
  deleteProfile: (id: string) => invoke<void>('delete_profile', { id }),
  moveProfileToGroup: (profileId: string, targetGroupId: string) => invoke<void>('move_profile_to_group', { profileId, targetGroupId }),
  moveProfile: (id: string, direction: 'up' | 'down') => invoke<void>('move_profile', { id, direction }),
  reorderProfiles: (fromId: string, toId: string) => invoke<void>('reorder_profiles', { fromId, toId }),
  runProfileTest: (profileId: string) => invoke<TestResult>('run_profile_test', { profileId }),
  clearGroupTestResults: (groupId: string) => invoke<void>('clear_test_results', { groupId }),
  deleteUnavailableProfiles: (groupId: string) => invoke<number>('delete_unavailable_profiles', { groupId }),

  setProfileGroupOpen: (id: string, open: boolean) => invoke<void>('set_profile_group_open', { id, open }),
  addProfileGroup: (label: string) => invoke<string>('add_profile_group', { label }),
  renameProfileGroup: (id: string, label: string) => invoke<void>('rename_profile_group', { id, label }),

  validateSource: (kind: 'url' | 'key', value: string) => invoke<string | null>('validate_source', { kind, value }),
  addSource: (input: AddSourceInput) => invoke<string>('add_source', { input }),
  updateSource: (id: string, patch: { name?: string; value?: string }) => invoke<void>('update_source', { id, patch }),
  refreshSource: (id: string) => invoke<void>('refresh_source', { id }),
  removeSource: (id: string) => invoke<void>('remove_source', { id }),

  setPreset: (id: string, enabled: boolean) => invoke<void>('set_preset', { id, enabled }),
  selectRule: (id: string) => invoke<void>('select_rule', { id }),
  updateRule: (id: string, patch: { match?: string; outbound?: string }) => invoke<void>('update_rule', { id, patch }),

  setTheme: (themeId: string) => invoke<void>('set_theme', { themeId }),
  updateSettings: (patch: Record<string, unknown>) => invoke<void>('update_settings', { patch }),

  onConnectionChanged: (handler: (connected: boolean) => void) => listen<boolean>('kagerou://connection-changed', (event) => handler(event.payload)),
  onTraffic: (handler: (event: TrafficEvent) => void) => listen<TrafficEvent>('kagerou://traffic', (event) => handler(event.payload)),
  onLog: (handler: (line: string) => void) => listen<string>('kagerou://log', (event) => handler(event.payload)),
  onCrashed: (handler: (exitCode: number | null) => void) => listen<number | null>('kagerou://crashed', (event) => handler(event.payload)),
}
