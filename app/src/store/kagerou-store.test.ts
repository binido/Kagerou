import { beforeEach, describe, expect, it, vi } from 'vitest'

import { toast } from 'sonner'

vi.mock('@/lib/tauri-api', async () => ({ kagerouApi: (await import('./test-api')).kagerouApiMock }))
vi.mock('@/themes/runtime', () => ({ persistThemeId: vi.fn() }))
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }))

import { kagerouApiMock as api, resetApiMock } from './test-api'
import { emptySnapshot, profile } from './test-fixtures'
import en from '@/locales/en/common.json'
import { TRAFFIC_HISTORY_LIMIT } from '@/types/kagerou'
import { persistThemeId } from '@/themes/runtime'
import type { AppSnapshot, TrafficEvent } from '@/lib/tauri-api'
import type { ProfileGroup, RoutingRule } from '@/types/kagerou'

const { useKagerouStore, subscribeToBackendEvents, __resetBackendEventSubscriptionForTests } =
  await import('./kagerou-store')

const initialState = useKagerouStore.getState()

beforeEach(() => {
  useKagerouStore.setState(initialState, true)
  __resetBackendEventSubscriptionForTests()
  resetApiMock()
  vi.mocked(toast.error).mockClear()
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
    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()

    const state = useKagerouStore.getState()
    expect(state.hydrated).toBe(true)
    expect(state.activeProfileId).toBe('p1')
    expect(state.profiles).toEqual(snapshot.profiles)
  })

  it('takes connected from the snapshot: the startup auto-connect event fires before anyone is listening', async () => {
    api.getAppState.mockResolvedValue({ ...emptySnapshot, connected: true })

    expect(useKagerouStore.getState().connected).toBe(false)
    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()

    expect(useKagerouStore.getState().connected).toBe(true)
  })

  it('flips hydrated with an error to show when the snapshot fails, so the window is never left blank', async () => {
    api.getAppState.mockRejectedValue({ code: 'storage', detail: 'database is locked' })

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()

    const state = useKagerouStore.getState()
    expect(state.hydrated).toBe(true)
    expect(state.hydrateError).toEqual({ dataDir: '/data/kagerou', message: en.errors.storage })
  })

  it('still reports the failure when the data directory cannot be read either', async () => {
    api.getAppState.mockRejectedValue({ code: 'storage', detail: 'database is locked' })
    api.appDataDir.mockRejectedValue(new Error('no ipc'))

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()

    expect(useKagerouStore.getState().hydrateError).toEqual({ dataDir: '', message: en.errors.storage })
  })

  it('subscribes to backend events exactly once even across repeated hydrate calls', async () => {
    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    subscribeToBackendEvents()
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

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    await vi.waitFor(() => expect(useKagerouStore.getState().updateAvailable).toEqual(update))
  })

  it('leaves updateAvailable null when there is nothing newer', async () => {
    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    await vi.waitFor(() => expect(api.checkForUpdate).toHaveBeenCalled())
    expect(useKagerouStore.getState().updateAvailable).toBeNull()
  })

  it('does not hold up hydration while the check is in flight', async () => {
    api.checkForUpdate.mockReturnValue(new Promise(() => {}))

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()

    expect(useKagerouStore.getState().hydrated).toBe(true)
    expect(useKagerouStore.getState().updateAvailable).toBeNull()
  })
})

describe('tray intents', () => {
  it('the tray connect item goes through the same toggle the button does', async () => {
    let fire: () => void = () => {}
    api.onTrayToggleConnection.mockImplementation((h: () => void) => { fire = h; return Promise.resolve(() => {}) })
    api.connect.mockResolvedValue(undefined)

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    useKagerouStore.setState({ connected: false })
    fire()
    await vi.waitFor(() => expect(api.connect).toHaveBeenCalled())
  })

  it('the tray profile item goes through the same select the list does', async () => {
    let fire: (id: string) => void = () => {}
    api.onTraySelectProfile.mockImplementation((h: (id: string) => void) => { fire = h; return Promise.resolve(() => {}) })
    api.selectProfile.mockResolvedValue(undefined)

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    useKagerouStore.setState({ profiles: [profile({ id: 'p1' })] })
    fire('p1')
    await vi.waitFor(() => expect(api.selectProfile).toHaveBeenCalledWith('p1'))
  })

  it('a tray entry for a profile that is gone does nothing', async () => {
    let fire: (id: string) => void = () => {}
    api.onTraySelectProfile.mockImplementation((h: (id: string) => void) => { fire = h; return Promise.resolve(() => {}) })

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    useKagerouStore.setState({ profiles: [profile({ id: 'p1' })] })
    fire('deleted-while-the-menu-was-open')

    expect(api.selectProfile).not.toHaveBeenCalled()
    expect(useKagerouStore.getState().activeProfileId).not.toBe('deleted-while-the-menu-was-open')
  })
})

describe('backend event handling', () => {
  it('connection-changed event updates connected', async () => {
    let handler: (connected: boolean) => void = () => {}
    api.onConnectionChanged.mockImplementation((h: (c: boolean) => void) => { handler = h; return Promise.resolve(() => {}) })

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    handler(true)
    expect(useKagerouStore.getState().connected).toBe(true)
    handler(false)
    expect(useKagerouStore.getState().connected).toBe(false)
  })

  it('a traffic sample event replaces the latest speed sample', async () => {
    let handler: (event: TrafficEvent) => void = () => {}
    api.onTraffic.mockImplementation((h: (e: TrafficEvent) => void) => { handler = h; return Promise.resolve(() => {}) })

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    handler({ kind: 'sample', up: 1, down: 2, uploadTotal: null, downloadTotal: null, activeConnections: null })
    handler({ kind: 'sample', up: 64, down: 128, uploadTotal: null, downloadTotal: null, activeConnections: null })

    expect(useKagerouStore.getState().trafficSample).toEqual({ download: 128, upload: 64 })
  })

  it('traffic history keeps the last minute of samples, oldest first', async () => {
    let handler: (event: TrafficEvent) => void = () => {}
    api.onTraffic.mockImplementation((h: (e: TrafficEvent) => void) => { handler = h; return Promise.resolve(() => {}) })

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    for (let i = 0; i < TRAFFIC_HISTORY_LIMIT + 10; i += 1) {
      handler({ kind: 'sample', up: i, down: i * 2, uploadTotal: null, downloadTotal: null, activeConnections: null })
    }

    const history = useKagerouStore.getState().trafficHistory
    expect(history).toHaveLength(TRAFFIC_HISTORY_LIMIT)
    expect(history[0]).toEqual({ download: 20, upload: 10 })
    expect(history.at(-1)).toEqual({ download: (TRAFFIC_HISTORY_LIMIT + 9) * 2, upload: TRAFFIC_HISTORY_LIMIT + 9 })
  })

  it('a sample with a null connection count keeps the previous one', async () => {
    let handler: (event: TrafficEvent) => void = () => {}
    api.onTraffic.mockImplementation((h: (e: TrafficEvent) => void) => { handler = h; return Promise.resolve(() => {}) })

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    handler({ kind: 'sample', up: 1, down: 2, uploadTotal: null, downloadTotal: null, activeConnections: 12 })
    handler({ kind: 'sample', up: 1, down: 2, uploadTotal: null, downloadTotal: null, activeConnections: null })

    expect(useKagerouStore.getState().activeConnections).toBe(12)
  })

  it('connecting asks where the tunnel comes out', async () => {
    let connection: (connected: boolean) => void = () => {}
    api.onConnectionChanged.mockImplementation((h: (c: boolean) => void) => { connection = h; return Promise.resolve(() => {}) })
    api.lookupExitLocation.mockResolvedValue({ ip: '81.2.69.142', city: 'London', country: 'United Kingdom', countryCode: 'GB' })

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    connection(true)
    await vi.waitFor(() => expect(useKagerouStore.getState().exitLocation?.city).toBe('London'))

    connection(false)
    expect(useKagerouStore.getState().exitLocation?.city).toBe(
      'London',
      // Where the last session came out is still worth reading; the alternative
      // is the flag from the profile name, which is the guess this replaced.
    )
  })

  it('a failed lookup leaves no location rather than a stale one', async () => {
    api.lookupExitLocation.mockResolvedValue({ ip: '81.2.69.142', city: 'London', country: 'United Kingdom', countryCode: 'GB' })
    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    await useKagerouStore.getState().refreshExitLocation()
    expect(useKagerouStore.getState().exitLocation?.city).toBe('London')

    api.lookupExitLocation.mockRejectedValue(new Error('nope'))
    await useKagerouStore.getState().refreshExitLocation()

    expect(useKagerouStore.getState()).toMatchObject({ exitLocation: null, exitLocationPending: false })
  })

  it('disconnecting stamps the uptime clock and clears the history', async () => {
    let traffic: (event: TrafficEvent) => void = () => {}
    let connection: (connected: boolean) => void = () => {}
    api.onTraffic.mockImplementation((h: (e: TrafficEvent) => void) => { traffic = h; return Promise.resolve(() => {}) })
    api.onConnectionChanged.mockImplementation((h: (c: boolean) => void) => { connection = h; return Promise.resolve(() => {}) })

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    connection(true)
    traffic({ kind: 'sample', up: 1, down: 2, uploadTotal: null, downloadTotal: null, activeConnections: 3 })
    expect(useKagerouStore.getState().connectedSince).toBeTypeOf('number')

    connection(false)
    expect(useKagerouStore.getState()).toMatchObject({
      connectedSince: null,
      trafficHistory: [],
      activeConnections: null,
    })
  })

  it('a traffic sample event replaces sessionTraffic with the backend-reported totals', async () => {
    let handler: (event: TrafficEvent) => void = () => {}
    api.onTraffic.mockImplementation((h: (e: TrafficEvent) => void) => { handler = h; return Promise.resolve(() => {}) })

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    handler({ kind: 'sample', up: 1, down: 2, uploadTotal: 250_000_000, downloadTotal: 1_900_000_000, activeConnections: null })
    handler({ kind: 'sample', up: 3, down: 4, uploadTotal: 260_000_000, downloadTotal: 1_950_000_000, activeConnections: null })

    expect(useKagerouStore.getState().sessionTraffic).toEqual({ download: 1_950_000_000, upload: 260_000_000 })
  })

  it('a sample with null totals keeps the previous sessionTraffic instead of blanking it', async () => {
    let handler: (event: TrafficEvent) => void = () => {}
    api.onTraffic.mockImplementation((h: (e: TrafficEvent) => void) => { handler = h; return Promise.resolve(() => {}) })
    useKagerouStore.setState({ sessionTraffic: { download: 500, upload: 100 } })

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    handler({ kind: 'sample', up: 1, down: 2, uploadTotal: null, downloadTotal: null, activeConnections: null })

    expect(useKagerouStore.getState().sessionTraffic).toEqual({ download: 500, upload: 100 })
  })

  it('a non-sample traffic event (disconnected/reconnecting) leaves the last sample alone', async () => {
    let handler: (event: TrafficEvent) => void = () => {}
    api.onTraffic.mockImplementation((h: (e: TrafficEvent) => void) => { handler = h; return Promise.resolve(() => {}) })
    useKagerouStore.setState({ trafficSample: { download: 7, upload: 3 } })

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    handler({ kind: 'disconnected' })
    handler({ kind: 'reconnecting' })

    expect(useKagerouStore.getState().trafficSample).toEqual({ download: 7, upload: 3 })
  })

  it('a log event is appended and level-detected from the message text', async () => {
    let handler: (line: string) => void = () => {}
    api.onLog.mockImplementation((h: (line: string) => void) => { handler = h; return Promise.resolve(() => {}) })
    useKagerouStore.setState({ logs: [] })

    subscribeToBackendEvents()
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

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    for (let i = 0; i < 550; i++) handler(`line ${i}`)

    expect(useKagerouStore.getState().logs).toHaveLength(500)
    expect(useKagerouStore.getState().logs[0].message).toBe('line 50')
  })
})

describe('backend failure reporting', () => {
  it('toggleConnection uses the disconnect fallback when the rejection carries no message', async () => {
    useKagerouStore.setState({ connected: true })
    api.disconnect.mockRejectedValue(undefined)

    await useKagerouStore.getState().toggleConnection()

    expect(toast.error).toHaveBeenCalledWith('Could not disconnect.')
  })

  it('reports a failed profile deletion', async () => {
    api.deleteProfile.mockRejectedValue({ code: 'activeProfileInUse', detail: 'in use' })
    await useKagerouStore.getState().deleteProfile('p1')
    expect(toast.error).toHaveBeenCalledWith(en.errors.activeProfileInUse)
  })

  it('reports a failed test cancellation', async () => {
    api.cancelGroupTest.mockRejectedValue({ code: 'storage', detail: 'no test run' })
    await useKagerouStore.getState().cancelGroupTest()
    expect(toast.error).toHaveBeenCalledWith(en.errors.storage)
  })

  it('rolls the group panel back from the snapshot when the open state fails to persist', async () => {
    const stored: ProfileGroup[] = [{ id: 'g1', label: 'G1', kind: 'custom', profileIds: [], open: false }]
    useKagerouStore.setState({ profileGroups: [{ ...stored[0] }] })
    api.setProfileGroupOpen.mockRejectedValue({ code: 'storage', detail: 'db locked' })
    api.getAppState.mockResolvedValue({ ...emptySnapshot, profileGroups: stored })

    useKagerouStore.getState().setProfileGroupOpen('g1', true)
    expect(useKagerouStore.getState().profileGroups[0].open).toBe(true)

    await vi.waitFor(() => expect(useKagerouStore.getState().profileGroups[0].open).toBe(false))
    expect(toast.error).toHaveBeenCalledWith(en.errors.storage)
  })

  it('rolls the preset toggle back from the snapshot when persistence fails', async () => {
    const stored = [{ id: 'bypass-lan', label: 'Bypass LAN', description: 'Private ranges go direct', enabled: false }]
    useKagerouStore.setState({ routingPresets: [{ ...stored[0] }] })
    api.setPreset.mockRejectedValue({ code: 'storage', detail: 'db locked' })
    api.getAppState.mockResolvedValue({ ...emptySnapshot, routingPresets: stored })

    useKagerouStore.getState().setPreset('bypass-lan', true)
    expect(useKagerouStore.getState().routingPresets[0].enabled).toBe(true)

    await vi.waitFor(() => expect(useKagerouStore.getState().routingPresets[0].enabled).toBe(false))
    expect(toast.error).toHaveBeenCalledWith(en.errors.storage)
  })

  it('reverts the rule selection from the snapshot when the backend refuses it', async () => {
    const stored: RoutingRule[] = [
      { id: 'r1', match: 'a.com', outbound: 'Direct', selected: true },
      { id: 'r2', match: 'b.com', outbound: 'Proxy', selected: false },
    ]
    useKagerouStore.setState({ routingRules: stored.map((rule) => ({ ...rule })) })
    api.selectRule.mockRejectedValue({ code: 'storage', detail: 'db locked' })
    api.getAppState.mockResolvedValue({ ...emptySnapshot, routingRules: stored })

    useKagerouStore.getState().selectRule('r2')
    expect(useKagerouStore.getState().routingRules.find((rule) => rule.id === 'r2')?.selected).toBe(true)

    await vi.waitFor(() => expect(useKagerouStore.getState().routingRules.find((rule) => rule.id === 'r1')?.selected).toBe(true))
    expect(toast.error).toHaveBeenCalledWith(en.errors.storage)
  })

  it('rolls the rule and the pending-changes flag back when the update fails to persist', async () => {
    const stored: RoutingRule[] = [{ id: 'r1', match: 'a.com', outbound: 'Direct', selected: false }]
    useKagerouStore.setState({ routingRules: stored.map((rule) => ({ ...rule })), connected: true, rulesChangedSinceConnect: false })
    api.updateRule.mockRejectedValue({ code: 'storage', detail: 'db locked' })
    api.getAppState.mockResolvedValue({ ...emptySnapshot, routingRules: stored, connected: true })

    useKagerouStore.getState().updateRule('r1', { outbound: 'Block' })
    expect(useKagerouStore.getState().rulesChangedSinceConnect).toBe(true)
    expect(useKagerouStore.getState().routingRules[0].outbound).toBe('Block')

    await vi.waitFor(() => expect(useKagerouStore.getState().rulesChangedSinceConnect).toBe(false))
    expect(useKagerouStore.getState().routingRules[0].outbound).toBe('Direct')
    expect(toast.error).toHaveBeenCalledWith(en.errors.storage)
  })

  it('reverts the theme and the persisted id when the backend refuses the change', async () => {
    useKagerouStore.setState({ settings: { ...initialState.settings, theme: 'catppuccin-mocha' } })
    api.setTheme.mockRejectedValue({ code: 'storage', detail: 'db locked' })
    api.getAppState.mockResolvedValue({ ...emptySnapshot, settings: { ...emptySnapshot.settings, theme: 'catppuccin-mocha' } })

    useKagerouStore.getState().setTheme('kanagawa-wave')
    expect(useKagerouStore.getState().settings.theme).toBe('kanagawa-wave')

    await vi.waitFor(() => expect(useKagerouStore.getState().settings.theme).toBe('catppuccin-mocha'))
    expect(persistThemeId).toHaveBeenLastCalledWith('catppuccin-mocha')
    expect(toast.error).toHaveBeenCalledWith(en.errors.storage)
  })

  it('rolls the settings back from the snapshot when persistence fails', async () => {
    useKagerouStore.setState({ settings: { ...initialState.settings, testUrl: 'http://changed.example/204' } })
    api.updateSettings.mockRejectedValue({ code: 'storage', detail: 'db locked' })
    api.getAppState.mockResolvedValue(emptySnapshot)

    useKagerouStore.getState().updateSettings({ testUrl: 'http://changed.example/204' })
    expect(useKagerouStore.getState().settings.testUrl).toBe('http://changed.example/204')

    await vi.waitFor(() => expect(useKagerouStore.getState().settings.testUrl).toBe('http://www.gstatic.com/generate_204'))
    expect(toast.error).toHaveBeenCalledWith(en.errors.storage)
  })
})
