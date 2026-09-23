import type { ThemeId } from '@/themes/types'

export type RouteKey =
  'dashboard' | 'groups' | 'routing-rules' | 'connections' | 'logs' | 'settings'

export type ProfileOrigin = 'local' | 'imported'
export type ProfileGroupKind = 'default' | 'custom' | 'subscription'
export type ProfileProtocol = 'VLESS' | 'VMess' | 'Trojan' | 'Shadowsocks' | 'Hysteria2' | 'Tuic'
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
/** sing-box's config log levels - deliberately not the display `LogLevel` above. */
export type SingBoxLogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error' | 'fatal' | 'panic'

/** Mirrors `storage::models::TestOutcome`: what measuring a profile
 * produced. A kind and, for a latency, a number - not a sentence, so
 * sorting has something to compare and this side has something to
 * translate. */
export type TestOutcome =
  | { kind: 'notTested' }
  | { kind: 'latency'; millis: number }
  | { kind: 'timeout' }
  | { kind: 'noResponse' }
  | { kind: 'unavailable' }

/** The outcome plus the colour it implies. The thresholds that decide the
 * colour live in Rust, so the two can never disagree. */
export type TestResult = TestOutcome & { tone: TestTone }

export const UNTESTED: TestResult = { kind: 'notTested', tone: 'muted' }

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
  /** Unix milliseconds as text, empty when never refreshed. */
  lastRefresh: string
  originLabel: 'Remote URL'
  provider: ProviderInfo
}

/** Mirrors `storage::models::ProviderInfo`: what the provider said on the
 * last fetch, `null` wherever it said nothing. */
export interface ProviderInfo {
  /** Bytes, upload and download together. */
  trafficUsed: number | null
  /** Bytes. `null` is no limit. */
  trafficTotal: number | null
  /** Unix milliseconds. `null` is no end date. */
  expiresAt: number | null
  announce: string | null
  supportUrl: string | null
}

/** Mirrors `subscription::Unsupported`: an entry an import left out. */
export type Unsupported =
  | { kind: 'protocol'; name: string }
  | { kind: 'transport'; name: string }
  | { kind: 'balancer' }
  | { kind: 'chain' }
  | { kind: 'invalid' }

/** Mirrors `import::Imported`: what a piece of pasted text became, and what
 * it held that could not be imported. */
export type ImportOutcome = (
  | { kind: 'subscriptionAdded'; groupId: string; added: number }
  | { kind: 'subscriptionRefreshed'; groupId: string }
  | { kind: 'profileAdded'; profileId: string; name: string }
  | { kind: 'groupAdded'; groupId: string; added: number; skipped: number }
  | { kind: 'alreadyPresent'; groupId: string }
  | { kind: 'nothingNew'; skipped: number }
) & { unsupported: Unsupported[] }

/** A failed attempt carries its text back so it can be corrected by hand. An
 * empty `error` means there was nothing to import in the first place. */
export type ImportAttempt =
  { status: 'imported'; outcome: ImportOutcome } | { status: 'failed'; text: string; error: string }

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
  mixedPort: number
  clashApiPort: number
}

/** Mirrors `usecase::connection::ConnectionExit`. */
export type ConnectionExit =
  | { kind: 'profile'; name: string }
  | { kind: 'direct' }
  | { kind: 'block' }
  | { kind: 'other'; tag: string }

/** Mirrors `usecase::connection::LiveConnection`. */
export interface LiveConnection {
  id: string
  host: string
  port: string
  network: string
  rule: string
  exit: ConnectionExit
  upload: number
  download: number
  /** RFC 3339 with up to nine fraction digits, as the Clash API reports it. */
  start: string
}
