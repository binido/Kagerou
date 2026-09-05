import type { ThemeId } from '@/themes/types'

export type RouteKey =
  | 'dashboard'
  | 'groups'
  | 'sources'
  | 'routing-rules'
  | 'logs'
  | 'settings'

export type ProfileOrigin = 'local' | 'imported'
export type ProfileGroupKind = 'default' | 'custom' | 'subscription'
export type ProfileProtocol =
  | 'VLESS'
  | 'VMess'
  | 'Trojan'
  | 'Shadowsocks'
  | 'Hysteria2'
  | 'Tuic'
export type TestMethod = 'tcp' | 'url'
export type TestTone = 'good' | 'warn' | 'bad' | 'muted'
export type SourceType = 'url' | 'key'
export type SourceStatus = 'up-to-date' | 'ready' | 'refresh-due' | 'updating'
export type Outbound = 'Direct' | 'Proxy' | 'Block'
export const routeOutboundOptions: Outbound[] = ['Direct', 'Proxy', 'Block']
export type LogLevel = 'INFO' | 'WARN' | 'ERROR'
export type Language = 'en' | 'ru'
export type TunInterface = 'utun / tun0' | 'utun' | 'tun0'
export type SubscriptionUpdateInterval = '5' | '10' | '15' | '30' | '60' | 'custom'
export type GroupSortMode = 'ping' | 'name' | 'protocol'

export interface TestResult {
  value: string
  tone: TestTone
}

export interface Profile {
  id: string
  name: string
  region: string
  protocol: ProfileProtocol
  origin: ProfileOrigin
  groupId: string
  sourceId?: string
  selected: boolean
  tcp: TestResult
  url: TestResult
  key: string
}

export type ProfileDraft = Omit<Profile, 'id' | 'groupId' | 'selected'>

export interface ProfileGroup {
  id: string
  label: string
  kind: ProfileGroupKind
  profileIds: string[]
  open: boolean
  sourceId?: string
}

export interface Source {
  id: string
  name: string
  type: SourceType
  value: string
  status: SourceStatus
  lastRefresh: string
  originLabel: 'Remote URL' | 'Local key'
}

export interface AddSourceInput {
  type: SourceType
  name?: string
  value: string
}

export interface AddLocalProfileInput {
  name: string
  key: string
  groupId?: string
  sourceId?: string
}

export interface RoutingPreset {
  id: string
  label: string
  description: string
  enabled: boolean
}

export interface RoutingRule {
  id: string
  match: string
  outbound: Outbound
  selected: boolean
}

export interface LogEntry {
  id: string
  timestamp: string
  level: LogLevel
  message: string
}

/** A GitHub release newer than the running build. */
export interface UpdateInfo {
  version: string
  url: string
}

/** The most recent per-second speed sample from sing-box, in bytes/s. */
export interface TrafficSample {
  download: number
  upload: number
}

/** Cumulative bytes moved since sing-box started this session. */
export interface SessionTraffic {
  download: number
  upload: number
}

export interface SettingsState {
  theme: ThemeId
  language: Language
  startup: boolean
  tunMode: boolean
  systemProxy: boolean
  tunInterface: TunInterface
  autoUpdateSubscriptions: boolean
  subscriptionUpdateInterval: SubscriptionUpdateInterval
  customSubscriptionUpdateMinutes: number
  groupSort: GroupSortMode
}

export interface KagerouStore {
  hydrated: boolean
  sidebarCollapsed: boolean
  connected: boolean
  activeProfileId: string
  profiles: Profile[]
  profileGroups: ProfileGroup[]
  sources: Source[]
  routingPresets: RoutingPreset[]
  routingRules: RoutingRule[]
  logs: LogEntry[]
  trafficSample: TrafficSample
  updateAvailable: UpdateInfo | null
  sessionTraffic: SessionTraffic
  settings: SettingsState
  hydrate: () => Promise<void>
  toggleSidebar: () => void
  toggleConnection: () => Promise<void>
  setProfileGroupOpen: (id: string, open: boolean) => void
  addProfileGroup: (label: string) => Promise<string | null>
  renameProfileGroup: (id: string, label: string) => Promise<boolean>
  selectProfile: (id: string) => Promise<void>
  addLocalProfile: (input: AddLocalProfileInput) => Promise<string | null>
  renameProfile: (id: string, name: string) => Promise<boolean>
  deleteProfile: (id: string) => Promise<void>
  moveProfileToGroup: (profileId: string, targetGroupId: string) => Promise<boolean>
  moveProfile: (id: string, direction: 'up' | 'down') => Promise<boolean>
  reorderProfiles: (fromId: string, toId: string) => Promise<boolean>
  runProfileTest: (id: string, method: TestMethod) => Promise<TestResult | null>
  addSource: (input: AddSourceInput) => Promise<string | null>
  updateSource: (id: string, patch: Partial<Pick<Source, 'name' | 'value'>>) => Promise<boolean>
  refreshSource: (id: string) => Promise<void>
  removeSource: (id: string) => Promise<boolean>
  setPreset: (id: string, enabled: boolean) => void
  selectRule: (id: string) => void
  updateRule: (id: string, patch: Partial<Pick<RoutingRule, 'match' | 'outbound'>>) => void
  setTheme: (themeId: ThemeId) => void
  updateSettings: (patch: Partial<SettingsState>) => void
}
