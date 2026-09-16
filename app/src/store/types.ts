import type { ConnectionSlice } from './slices/connection'
import type { LogsSlice } from './slices/logs'
import type { ProfilesSlice } from './slices/profiles'
import type { RoutingSlice } from './slices/routing'
import type { SettingsSlice } from './slices/settings'
import type { ShellSlice } from './slices/shell'
import type { SubscriptionsSlice } from './slices/subscriptions'
import type { TestingSlice } from './slices/testing'

/** The whole store, as the sum of the areas it covers. One Zustand store
 * still, so `set` reaches everything: the split is about where a reader
 * looks for something, not about walling the areas off from each other. */
export type KagerouStore = ShellSlice
  & ConnectionSlice
  & LogsSlice
  & ProfilesSlice
  & RoutingSlice
  & SettingsSlice
  & SubscriptionsSlice
  & TestingSlice
