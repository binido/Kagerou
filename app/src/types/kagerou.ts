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
export type TestMethod = 'tcp' | 'url'
export type TestTone = 'good' | 'warn' | 'bad' | 'muted'
export type SourceType = 'url' | 'key'
export type SourceStatus = 'up-to-date' | 'ready' | 'refresh-due' | 'updating'
export type Outbound = 'Direct' | 'Proxy' | 'Block'
export type LogLevel = 'INFO' | 'WARN' | 'ERROR'
export type Language = 'en' | 'ru'
export type Theme = 'dark' | 'light'
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
  autoUpdateSubscriptions: boolean
  subscriptionUpdateInterval: SubscriptionUpdateInterval
  customSubscriptionUpdateMinutes: number
  groupSort: GroupSortMode
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
  addProfileGroup: (label: string) => string | null
  renameProfileGroup: (id: string, label: string) => boolean
  selectProfile: (id: string) => void
  addLocalProfile: (input: AddLocalProfileInput) => string | null
  renameProfile: (id: string, name: string) => boolean
  deleteProfile: (id: string) => void
  moveProfileToGroup: (profileId: string, targetGroupId: string) => boolean
  moveProfile: (id: string, direction: 'up' | 'down') => boolean
  reorderProfiles: (fromId: string, toId: string) => boolean
  setTestResult: (id: string, method: TestMethod, result: TestResult) => void
  replaceSubscriptionProfiles: (sourceId: string, profiles: ProfileDraft[]) => boolean
  addSource: (input: AddSourceInput, importedProfiles?: ProfileDraft[]) => string | null
  updateSource: (id: string, patch: Partial<Pick<Source, 'name' | 'value' | 'status' | 'lastRefresh'>>) => boolean
  removeSource: (id: string) => boolean
  setPreset: (id: string, enabled: boolean) => void
  selectRule: (id: string) => void
  updateRule: (id: string, patch: Partial<RoutingRule>) => void
  updateSettings: (patch: Partial<SettingsState>) => void
}
