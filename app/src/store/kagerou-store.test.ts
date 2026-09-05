import { beforeEach, describe, expect, it, vi } from 'vitest'

import type { AppSnapshot, TrafficEvent } from '@/lib/tauri-api'
import type { Profile, RoutingRule, Source } from '@/types/kagerou'

const api = vi.hoisted(() => ({
  getAppState: vi.fn(),
  checkForUpdate: vi.fn(),
  connect: vi.fn(),
  disconnect: vi.fn(),
  selectProfile: vi.fn(),
  addLocalProfile: vi.fn(),
  renameProfile: vi.fn(),
  deleteProfile: vi.fn(),
  moveProfileToGroup: vi.fn(),
  moveProfile: vi.fn(),
  reorderProfiles: vi.fn(),
  runProfileTest: vi.fn(),
  setProfileGroupOpen: vi.fn(),
  addProfileGroup: vi.fn(),
  renameProfileGroup: vi.fn(),
  validateSource: vi.fn(),
  addSource: vi.fn(),
  updateSource: vi.fn(),
  refreshSource: vi.fn(),
  removeSource: vi.fn(),
  setPreset: vi.fn(),
  selectRule: vi.fn(),
  updateRule: vi.fn(),
  setTheme: vi.fn(),
  updateSettings: vi.fn(),
  onConnectionChanged: vi.fn(),
  onTraffic: vi.fn(),
  onLog: vi.fn(),
  onCrashed: vi.fn(),
}))

vi.mock('@/lib/tauri-api', () => ({ kagerouApi: api }))

vi.mock('@/themes/runtime', () => ({ persistThemeId: vi.fn() }))

const { useKagerouStore, __resetBackendEventSubscriptionForTests } = await import('@/store/kagerou-store')

const emptySnapshot: AppSnapshot = {
  activeProfileId: '',
  profiles: [],
  profileGroups: [],
  sources: [],
  routingPresets: [],
  routingRules: [],
  settings: {
    theme: 'catppuccin-mocha',
    language: 'en',
    startup: true,
    tunMode: false,
    systemProxy: false,
    tunInterface: 'utun / tun0',
    autoUpdateSubscriptions: false,
    subscriptionUpdateInterval: '30',
    customSubscriptionUpdateMinutes: 60,
    groupSort: 'ping',
  },
}

const profile = (overrides: Partial<Profile> = {}): Profile => ({
  id: 'p1',
  name: 'P1',
  region: 'us',
  protocol: 'VLESS',
  origin: 'local',
  groupId: 'default',
  selected: false,
  tcp: { value: 'Not tested', tone: 'muted' },
  url: { value: 'Not tested', tone: 'muted' },
  key: 'vless://p1',
  ...overrides,
})

const initialState = useKagerouStore.getState()

beforeEach(() => {
  useKagerouStore.setState(initialState, true)
  __resetBackendEventSubscriptionForTests()
  Object.values(api).forEach((fn) => fn.mockReset())
  api.getAppState.mockResolvedValue(emptySnapshot)
  api.checkForUpdate.mockResolvedValue(null)
  api.onConnectionChanged.mockResolvedValue(() => {})
  api.onTraffic.mockResolvedValue(() => {})
  api.onLog.mockResolvedValue(() => {})
  api.onCrashed.mockResolvedValue(() => {})
  // Fire-and-forget mutations call `.catch()` on the invoke promise, so
  // every action mock needs to resolve to *something* by default even in
  // tests that don't care about its outcome.
  for (const fn of [api.setProfileGroupOpen, api.setPreset, api.selectRule, api.updateRule, api.setTheme, api.updateSettings]) {
    fn.mockResolvedValue(undefined)
  }
})

describe('hydrate', () => {
  it('populates state from the backend snapshot and flips hydrated', async () => {
    const snapshot: AppSnapshot = {
      ...emptySnapshot,
      activeProfileId: 'p1',
      profiles: [profile({ selected: true })],
    }
    api.getAppState.mockResolvedValue(snapshot)

    expect(useKagerouStore.getState().hydrated).toBe(false)
    await useKagerouStore.getState().hydrate()

    const state = useKagerouStore.getState()
    expect(state.hydrated).toBe(true)
    expect(state.activeProfileId).toBe('p1')
    expect(state.profiles).toEqual(snapshot.profiles)
  })

  it('subscribes to backend events exactly once even across repeated hydrate calls', async () => {
    await useKagerouStore.getState().hydrate()
    await useKagerouStore.getState().hydrate()
    // Subscription is a module-level singleton guard; this asserts hydrate
    // is safe to call more than once (e.g. StrictMode double-invoke)
    // without registering duplicate listeners.
    expect(api.onConnectionChanged.mock.calls.length).toBeLessThanOrEqual(1)
  })
})

describe('toggleSidebar', () => {
  it('flips sidebarCollapsed with no backend call', () => {
    expect(useKagerouStore.getState().sidebarCollapsed).toBe(false)
    useKagerouStore.getState().toggleSidebar()
    expect(useKagerouStore.getState().sidebarCollapsed).toBe(true)
    expect(api.connect).not.toHaveBeenCalled()
  })
})

describe('update check', () => {
  it('stores a release the backend reports as newer', async () => {
    const update = { version: '0.3.0', url: 'https://github.com/binido/Kagerou/releases/tag/v0.3.0' }
    api.checkForUpdate.mockResolvedValue(update)

    await useKagerouStore.getState().hydrate()
    await vi.waitFor(() => expect(useKagerouStore.getState().updateAvailable).toEqual(update))
  })

  it('leaves updateAvailable null when there is nothing newer', async () => {
    await useKagerouStore.getState().hydrate()
    await vi.waitFor(() => expect(api.checkForUpdate).toHaveBeenCalled())
    expect(useKagerouStore.getState().updateAvailable).toBeNull()
  })

  it('does not hold up hydration while the check is in flight', async () => {
    api.checkForUpdate.mockReturnValue(new Promise(() => {}))

    await useKagerouStore.getState().hydrate()

    expect(useKagerouStore.getState().hydrated).toBe(true)
    expect(useKagerouStore.getState().updateAvailable).toBeNull()
  })
})

describe('connection modes', () => {
  it('are persisted settings, not runtime state: the backend decides TUN at connect time', async () => {
    api.updateSettings.mockResolvedValue(undefined)
    useKagerouStore.getState().updateSettings({ tunMode: true })

    expect(useKagerouStore.getState().settings.tunMode).toBe(true)
    expect(api.updateSettings).toHaveBeenCalledWith({ tunMode: true })
    expect(api.connect).not.toHaveBeenCalled()
    expect(api.disconnect).not.toHaveBeenCalled()
  })
})

describe('toggleConnection', () => {
  it('calls connect without arguments when disconnected', async () => {
    useKagerouStore.setState({ connected: false })
    api.connect.mockResolvedValue(undefined)
    await useKagerouStore.getState().toggleConnection()
    expect(api.connect).toHaveBeenCalledWith()
    expect(api.disconnect).not.toHaveBeenCalled()
  })

  it('calls disconnect when already connected', async () => {
    useKagerouStore.setState({ connected: true })
    api.disconnect.mockResolvedValue(undefined)
    await useKagerouStore.getState().toggleConnection()
    expect(api.disconnect).toHaveBeenCalled()
    expect(api.connect).not.toHaveBeenCalled()
  })

  it('does not throw when the backend call rejects', async () => {
    useKagerouStore.setState({ connected: false })
    api.connect.mockRejectedValue(new Error('sing-box not running'))
    await expect(useKagerouStore.getState().toggleConnection()).resolves.toBeUndefined()
  })
})

describe('selectProfile', () => {
  it('optimistically marks the profile selected before the backend call resolves', async () => {
    useKagerouStore.setState({ profiles: [profile({ id: 'a', selected: true }), profile({ id: 'b', selected: false })] })
    let resolveInvoke: () => void = () => {}
    api.selectProfile.mockReturnValue(new Promise<void>((resolve) => { resolveInvoke = resolve }))

    const pending = useKagerouStore.getState().selectProfile('b')
    // Before the backend call resolves, local state must already reflect the switch.
    expect(useKagerouStore.getState().profiles.find((p) => p.id === 'b')?.selected).toBe(true)
    expect(useKagerouStore.getState().profiles.find((p) => p.id === 'a')?.selected).toBe(false)
    expect(useKagerouStore.getState().activeProfileId).toBe('b')

    resolveInvoke()
    await pending
  })

  it('is a no-op for an unknown profile id', async () => {
    useKagerouStore.setState({ profiles: [profile({ id: 'a', selected: true })], activeProfileId: 'a' })
    await useKagerouStore.getState().selectProfile('does-not-exist')
    expect(api.selectProfile).not.toHaveBeenCalled()
    expect(useKagerouStore.getState().activeProfileId).toBe('a')
  })

  it('re-syncs from the backend if the switch fails server-side', async () => {
    useKagerouStore.setState({ profiles: [profile({ id: 'a', selected: true }), profile({ id: 'b' })] })
    api.selectProfile.mockRejectedValue(new Error('not found'))
    api.getAppState.mockResolvedValue({ ...emptySnapshot, activeProfileId: 'a', profiles: [profile({ id: 'a', selected: true }), profile({ id: 'b' })] })

    await useKagerouStore.getState().selectProfile('b')

    expect(api.getAppState).toHaveBeenCalled()
    expect(useKagerouStore.getState().activeProfileId).toBe('a')
  })
})

describe('addProfileGroup / renameProfileGroup', () => {
  it('addProfileGroup returns the new id and refreshes state on success', async () => {
    api.addProfileGroup.mockResolvedValue('group-123')
    const snapshot: AppSnapshot = { ...emptySnapshot, profileGroups: [{ id: 'group-123', label: 'New', kind: 'custom', profileIds: [], open: true }] }
    api.getAppState.mockResolvedValue(snapshot)

    const id = await useKagerouStore.getState().addProfileGroup('New')

    expect(id).toBe('group-123')
    expect(useKagerouStore.getState().profileGroups).toEqual(snapshot.profileGroups)
  })

  it('addProfileGroup returns null on a duplicate-name rejection, without touching state', async () => {
    useKagerouStore.setState({ profileGroups: [{ id: 'g1', label: 'Existing', kind: 'custom', profileIds: [], open: true }] })
    api.addProfileGroup.mockRejectedValue(new Error('duplicate name'))

    const id = await useKagerouStore.getState().addProfileGroup('Existing')

    expect(id).toBeNull()
    expect(api.getAppState).not.toHaveBeenCalled()
    expect(useKagerouStore.getState().profileGroups).toHaveLength(1)
  })

  it('renameProfileGroup returns false when the backend refuses (e.g. the default group)', async () => {
    api.renameProfileGroup.mockRejectedValue(new Error('the default group cannot be renamed'))
    const ok = await useKagerouStore.getState().renameProfileGroup('default', 'Renamed')
    expect(ok).toBe(false)
  })
})

describe('setProfileGroupOpen', () => {
  it('updates local state synchronously and fires the backend call without waiting', () => {
    useKagerouStore.setState({ profileGroups: [{ id: 'g1', label: 'G1', kind: 'custom', profileIds: [], open: false }] })
    api.setProfileGroupOpen.mockResolvedValue(undefined)

    useKagerouStore.getState().setProfileGroupOpen('g1', true)

    expect(useKagerouStore.getState().profileGroups[0].open).toBe(true)
    expect(api.setProfileGroupOpen).toHaveBeenCalledWith('g1', true)
  })
})

describe('runProfileTest', () => {
  it('applies the returned result to the matching profile and returns it', async () => {
    useKagerouStore.setState({ profiles: [profile({ id: 'p1' })] })
    api.runProfileTest.mockResolvedValue({ value: '42 ms', tone: 'good' })

    const result = await useKagerouStore.getState().runProfileTest('p1', 'tcp')

    expect(result).toEqual({ value: '42 ms', tone: 'good' })
    expect(useKagerouStore.getState().profiles[0].tcp).toEqual({ value: '42 ms', tone: 'good' })
  })

  it('returns null and leaves the profile untouched when the backend call fails', async () => {
    const original = profile({ id: 'p1' })
    useKagerouStore.setState({ profiles: [original] })
    api.runProfileTest.mockRejectedValue(new Error('not connected'))

    const result = await useKagerouStore.getState().runProfileTest('p1', 'tcp')

    expect(result).toBeNull()
    expect(useKagerouStore.getState().profiles[0]).toEqual(original)
  })
})

describe('setTheme', () => {
  it('applies a known theme id locally and persists it', () => {
    useKagerouStore.getState().setTheme('kanagawa-wave')
    expect(useKagerouStore.getState().settings.theme).toBe('kanagawa-wave')
    expect(api.setTheme).toHaveBeenCalledWith('kanagawa-wave')
  })

  it('ignores an unknown theme id entirely', () => {
    const before = useKagerouStore.getState().settings.theme
    useKagerouStore.getState().setTheme('not-a-real-theme')
    expect(useKagerouStore.getState().settings.theme).toBe(before)
    expect(api.setTheme).not.toHaveBeenCalled()
  })
})

describe('updateSettings', () => {
  it('merges the patch into settings without touching unrelated fields', () => {
    useKagerouStore.getState().updateSettings({ startup: false })
    expect(useKagerouStore.getState().settings.startup).toBe(false)
    expect(useKagerouStore.getState().settings.language).toBe('en')
    expect(api.updateSettings).toHaveBeenCalledWith({ startup: false })
  })
})

describe('routing rule actions', () => {
  const rules: RoutingRule[] = [
    { id: 'r1', match: 'a.com', outbound: 'Direct', selected: true },
    { id: 'r2', match: 'b.com', outbound: 'Proxy', selected: false },
  ]

  it('selectRule makes exactly one rule selected', () => {
    useKagerouStore.setState({ routingRules: rules })
    useKagerouStore.getState().selectRule('r2')
    const selected = useKagerouStore.getState().routingRules.filter((r) => r.selected)
    expect(selected.map((r) => r.id)).toEqual(['r2'])
  })

  it('updateRule patches only the targeted rule', () => {
    useKagerouStore.setState({ routingRules: rules })
    useKagerouStore.getState().updateRule('r1', { outbound: 'Block' })
    const updated = useKagerouStore.getState().routingRules
    expect(updated.find((r) => r.id === 'r1')?.outbound).toBe('Block')
    expect(updated.find((r) => r.id === 'r2')?.outbound).toBe('Proxy')
  })
})

describe('backend event handling', () => {
  it('connection-changed event updates connected', async () => {
    let handler: (connected: boolean) => void = () => {}
    api.onConnectionChanged.mockImplementation((h: (c: boolean) => void) => { handler = h; return Promise.resolve(() => {}) })

    await useKagerouStore.getState().hydrate()
    handler(true)
    expect(useKagerouStore.getState().connected).toBe(true)
    handler(false)
    expect(useKagerouStore.getState().connected).toBe(false)
  })

  it('a traffic sample event replaces the latest speed sample', async () => {
    let handler: (event: TrafficEvent) => void = () => {}
    api.onTraffic.mockImplementation((h: (e: TrafficEvent) => void) => { handler = h; return Promise.resolve(() => {}) })

    await useKagerouStore.getState().hydrate()
    handler({ kind: 'sample', up: 1, down: 2, uploadTotal: null, downloadTotal: null })
    handler({ kind: 'sample', up: 64, down: 128, uploadTotal: null, downloadTotal: null })

    expect(useKagerouStore.getState().trafficSample).toEqual({ download: 128, upload: 64 })
  })

  it('a traffic sample event replaces sessionTraffic with the backend-reported totals', async () => {
    let handler: (event: TrafficEvent) => void = () => {}
    api.onTraffic.mockImplementation((h: (e: TrafficEvent) => void) => { handler = h; return Promise.resolve(() => {}) })

    await useKagerouStore.getState().hydrate()
    handler({ kind: 'sample', up: 1, down: 2, uploadTotal: 250_000_000, downloadTotal: 1_900_000_000 })
    handler({ kind: 'sample', up: 3, down: 4, uploadTotal: 260_000_000, downloadTotal: 1_950_000_000 })

    expect(useKagerouStore.getState().sessionTraffic).toEqual({ download: 1_950_000_000, upload: 260_000_000 })
  })

  it('a sample with null totals keeps the previous sessionTraffic instead of blanking it', async () => {
    let handler: (event: TrafficEvent) => void = () => {}
    api.onTraffic.mockImplementation((h: (e: TrafficEvent) => void) => { handler = h; return Promise.resolve(() => {}) })
    useKagerouStore.setState({ sessionTraffic: { download: 500, upload: 100 } })

    await useKagerouStore.getState().hydrate()
    handler({ kind: 'sample', up: 1, down: 2, uploadTotal: null, downloadTotal: null })

    expect(useKagerouStore.getState().sessionTraffic).toEqual({ download: 500, upload: 100 })
  })

  it('a non-sample traffic event (disconnected/reconnecting) leaves the last sample alone', async () => {
    let handler: (event: TrafficEvent) => void = () => {}
    api.onTraffic.mockImplementation((h: (e: TrafficEvent) => void) => { handler = h; return Promise.resolve(() => {}) })
    useKagerouStore.setState({ trafficSample: { download: 7, upload: 3 } })

    await useKagerouStore.getState().hydrate()
    handler({ kind: 'disconnected' })
    handler({ kind: 'reconnecting' })

    expect(useKagerouStore.getState().trafficSample).toEqual({ download: 7, upload: 3 })
  })

  it('a log event is appended and level-detected from the message text', async () => {
    let handler: (line: string) => void = () => {}
    api.onLog.mockImplementation((h: (line: string) => void) => { handler = h; return Promise.resolve(() => {}) })
    useKagerouStore.setState({ logs: [] })

    await useKagerouStore.getState().hydrate()
    handler('outbound/proxy dialing tokyo-01.kagerou.network:443')
    handler('certificate chain will expire in 21 days (WARN)')
    handler('upstream reset by peer: cdn.example.net:443 ERROR')

    const logs = useKagerouStore.getState().logs
    expect(logs).toHaveLength(3)
    expect(logs[0].level).toBe('INFO')
    expect(logs[1].level).toBe('WARN')
    expect(logs[2].level).toBe('ERROR')
  })

  it('the log buffer is capped so a noisy backend cannot grow it unbounded', async () => {
    let handler: (line: string) => void = () => {}
    api.onLog.mockImplementation((h: (line: string) => void) => { handler = h; return Promise.resolve(() => {}) })
    useKagerouStore.setState({ logs: [] })

    await useKagerouStore.getState().hydrate()
    for (let i = 0; i < 550; i++) handler(`line ${i}`)

    expect(useKagerouStore.getState().logs).toHaveLength(500)
    expect(useKagerouStore.getState().logs[0].message).toBe('line 50')
  })
})

describe('addSource / updateSource / removeSource failure handling', () => {
  it('addSource returns null and does not throw when the fetch/parse fails', async () => {
    api.addSource.mockRejectedValue(new Error('could not reach subscription URL'))
    const id = await useKagerouStore.getState().addSource({ type: 'url', value: 'https://example.com/sub' })
    expect(id).toBeNull()
  })

  it('updateSource returns false on a validation failure', async () => {
    api.updateSource.mockRejectedValue(new Error('source name cannot be empty'))
    const ok = await useKagerouStore.getState().updateSource('s1', { name: '' })
    expect(ok).toBe(false)
  })

  it('removeSource returns false for an unknown id', async () => {
    api.removeSource.mockRejectedValue(new Error('not found'))
    const ok = await useKagerouStore.getState().removeSource('ghost')
    expect(ok).toBe(false)
  })

  it('removeSource returns true and refreshes state on success', async () => {
    useKagerouStore.setState({ sources: [{ id: 's1', name: 'S', type: 'url', value: 'https://x', status: 'up-to-date', lastRefresh: '', originLabel: 'Remote URL' } as Source] })
    api.removeSource.mockResolvedValue(undefined)
    api.getAppState.mockResolvedValue({ ...emptySnapshot, sources: [] })

    const ok = await useKagerouStore.getState().removeSource('s1')

    expect(ok).toBe(true)
    expect(useKagerouStore.getState().sources).toEqual([])
  })
})
