import type { ThemeId } from '@/themes/types'

export type RouteKey =
  | 'dashboard'
  | 'groups'
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
export type TestTone = 'good' | 'warn' | 'bad' | 'muted'
export type SourceStatus = 'up-to-date' | 'ready' | 'refresh-due' | 'updating'
export type Outbound = 'Direct' | 'Proxy' | 'Block'
export const routeOutboundOptions: Outbound[] = ['Direct', 'Proxy', 'Block']
export type MatchKind = 'domain' | 'domain-suffix' | 'ip-cidr'
export type MatchWarning = 'wildcard' | 'url' | 'non-ascii' | 'invalid-prefix' | 'invalid-domain'

/** Mirrors `singbox::match_spec::MatchAnalysis`. `normalized` is what gets
 * stored, which is not always what was typed. */
export interface MatchAnalysis {
  normalized: string
  kind: MatchKind
  warning: MatchWarning | null
}
export type LogLevel = 'INFO' | 'WARN' | 'ERROR'
export type Language = 'en' | 'ru'
export type TunInterface = 'utun / tun0' | 'utun' | 'tun0'
export type SubscriptionUpdateInterval = '5' | '10' | '15' | '30' | '60' | 'custom'
export type GroupSortMode = 'ping' | 'name' | 'protocol'
/** sing-box's config log levels — deliberately not the display `LogLevel` above. */
export type SingBoxLogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error' | 'fatal' | 'panic'

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

/** The URL behind a subscription group. Single keys have no source. */
export interface Source {
  id: string
  name: string
  type: 'url'
  value: string
  status: SourceStatus
  lastRefresh: string
  originLabel: 'Remote URL'
}

/** Mirrors `import::ImportOutcome`: what a piece of pasted text became. */
export type ImportOutcome =
  | { kind: 'subscriptionAdded'; groupId: string; added: number }
  | { kind: 'subscriptionRefreshed'; groupId: string }
  | { kind: 'profileAdded'; profileId: string; name: string }
  | { kind: 'groupAdded'; groupId: string; added: number; skipped: number }
  | { kind: 'alreadyPresent'; groupId: string }
  | { kind: 'nothingNew'; skipped: number }

/** A failed attempt carries its text back so it can be corrected by hand. An
 * empty `error` means there was nothing to import in the first place. */
export type ImportAttempt =
  | { status: 'imported'; outcome: ImportOutcome }
  | { status: 'failed'; text: string; error: string }

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

/** A group test walking its profiles one at a time. */
export interface TestRun {
  groupId: string | null
  done: number
  total: number
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

/** Speed history behind the dashboard sparkline: one sample per second,
 * oldest first, capped at a minute. */
export const TRAFFIC_HISTORY_LIMIT = 60

/** Where the internet says the tunnel comes out, looked up through the
 * tunnel itself. `city` can be empty; the country cannot. */
export interface ExitLocation {
  ip: string
  city: string
  country: string
  countryCode: string
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
  autoConnect: boolean
  geoLookup: boolean
  tunInterface: TunInterface
  autoUpdateSubscriptions: boolean
  subscriptionUpdateInterval: SubscriptionUpdateInterval
  customSubscriptionUpdateMinutes: number
  groupSort: GroupSortMode
  logLevel: SingBoxLogLevel
  testUrl: string
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
  /** Rules were edited after the current connection came up, so the running
   * core is still on the config generated at connect time. */
  rulesChangedSinceConnect: boolean
  logs: LogEntry[]
  trafficSample: TrafficSample
  trafficHistory: TrafficSample[]
  /** Live connections reported by the last sample; `null` before the first
   * one arrives or when the Clash API fetch failed. */
  activeConnections: number | null
  /** Unix milliseconds the current connection came up, or `null` when down. */
  connectedSince: number | null
  exitLocation: ExitLocation | null
  exitLocationPending: boolean
  updateAvailable: UpdateInfo | null
  testRun: TestRun | null
  sessionTraffic: SessionTraffic
  settings: SettingsState
  hydrate: () => Promise<void>
  toggleSidebar: () => void
  toggleConnection: () => Promise<void>
  setProfileGroupOpen: (id: string, open: boolean) => void
  addProfileGroup: (label: string) => Promise<string | null>
  renameProfileGroup: (id: string, label: string) => Promise<boolean>
  selectProfile: (id: string) => Promise<void>
  renameProfile: (id: string, name: string) => Promise<boolean>
  deleteProfile: (id: string) => Promise<void>
  moveProfileToGroup: (profileId: string, targetGroupId: string) => Promise<boolean>
  moveProfile: (id: string, direction: 'up' | 'down') => Promise<boolean>
  reorderProfiles: (fromId: string, toId: string) => Promise<boolean>
  runProfileTest: (id: string) => Promise<TestResult | null>
  startGroupTest: (groupId: string | null) => Promise<void>
  cancelGroupTest: () => Promise<void>
  clearGroupTestResults: (groupId: string) => Promise<void>
  deleteUnavailableProfiles: (groupId: string) => Promise<number>
  importText: (text: string) => Promise<ImportAttempt>
  importFromClipboard: () => Promise<ImportAttempt>
  updateSource: (id: string, patch: Partial<Pick<Source, 'name' | 'value'>>) => Promise<boolean>
  refreshSource: (id: string) => Promise<void>
  deleteSubscription: (groupId: string) => Promise<boolean>
  setPreset: (id: string, enabled: boolean) => void
  selectRule: (id: string) => void
  updateRule: (id: string, patch: Partial<Pick<RoutingRule, 'match' | 'outbound'>>) => void
  addRule: (match: string, outbound: Outbound) => Promise<string | null>
  deleteRule: (id: string) => Promise<boolean>
  setTheme: (themeId: ThemeId) => void
  updateSettings: (patch: Partial<SettingsState>) => void
  refreshExitLocation: () => Promise<void>
}
