export type RouteKey =
  | 'dashboard'
  | 'profiles'
  | 'sources'
  | 'routing-rules'
  | 'logs'
  | 'settings'

export type ProfileOrigin = 'local' | 'imported'
export type ProfileProtocol =
  | 'VLESS'
  | 'VMess'
  | 'Trojan'
  | 'Shadowsocks'
  | 'Hysteria2'
export type TestMethod = 'tcp' | 'url'
export type TestTone = 'good' | 'warn' | 'bad' | 'muted'
export type SourceType = 'url' | 'key'
export type SourceStatus = 'up-to-date' | 'ready' | 'refresh-due' | 'updating'
export type Outbound = 'Direct' | 'Proxy' | 'Block'
export type LogLevel = 'INFO' | 'WARN' | 'ERROR'
export type Language = 'English' | '中文' | '日本語'
export type Theme = 'dark' | 'light'
export type TunInterface = 'utun / tun0' | 'utun' | 'tun0'

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
  sourceId?: string
  selected: boolean
  tcp: TestResult
  url: TestResult
  key: string
}

export interface ProfileGroup {
  id: string
  label: string
  profileIds: string[]
  open: boolean
  managed?: boolean
}

export interface Source {
  id: string
  name: string
  type: SourceType
  value: string
  profileCount: number
  status: SourceStatus
  lastRefresh: string
  originLabel: 'Remote URL' | 'Local key'
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

export interface TelemetryPoint {
  label: string
  download: number
  upload: number
}

export interface SettingsState {
  theme: Theme
  language: Language
  startup: boolean
  tunInterface: TunInterface
}

export interface KagerouStore {
  sidebarCollapsed: boolean
  connected: boolean
  tunMode: boolean
  systemProxy: boolean
  activeProfileId: string
  profiles: Profile[]
  profileGroups: ProfileGroup[]
  sources: Source[]
  routingPresets: RoutingPreset[]
  routingRules: RoutingRule[]
  logs: LogEntry[]
  telemetry: TelemetryPoint[]
  settings: SettingsState
  toggleSidebar: () => void
  toggleConnection: () => void
  toggleMode: (mode: 'tun' | 'proxy') => void
  setProfileGroupOpen: (id: string, open: boolean) => void
  selectProfile: (id: string) => void
  addProfile: (input: Pick<Profile, 'name' | 'key'>) => void
  renameProfile: (id: string, name: string) => void
  deleteProfile: (id: string) => void
  moveProfile: (id: string, direction: 'up' | 'down') => void
  reorderProfiles: (fromId: string, toId: string) => void
  setTestResult: (id: string, method: TestMethod, result: TestResult) => void
  addSource: (source: Source) => void
  updateSource: (id: string, patch: Partial<Source>) => void
  removeSource: (id: string) => void
  setPreset: (id: string, enabled: boolean) => void
  selectRule: (id: string) => void
  updateRule: (id: string, patch: Partial<RoutingRule>) => void
  updateSettings: (patch: Partial<SettingsState>) => void
}
