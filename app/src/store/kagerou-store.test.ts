import { beforeEach, describe, expect, it, vi } from 'vitest'

import { toast } from 'sonner'

import type { AppSnapshot, TrafficEvent } from '@/lib/tauri-api'
import type { ExitLocation, ImportOutcome, Profile, ProfileGroup, RoutingRule, TestResult } from '@/types/kagerou'
import { TRAFFIC_HISTORY_LIMIT } from '@/types/kagerou'
import { persistThemeId } from '@/themes/runtime'

const api = vi.hoisted(() => ({
  getAppState: vi.fn(),
  checkForUpdate: vi.fn(),
  connect: vi.fn(),
  disconnect: vi.fn(),
  selectProfile: vi.fn(),
  renameProfile: vi.fn(),
  deleteProfile: vi.fn(),
  moveProfileToGroup: vi.fn(),
  moveProfile: vi.fn(),
  reorderProfiles: vi.fn(),
  runProfileTest: vi.fn(),
  clearGroupTestResults: vi.fn(),
  deleteUnavailableProfiles: vi.fn(),
  setProfileGroupOpen: vi.fn(),
  addProfileGroup: vi.fn(),
  renameProfileGroup: vi.fn(),
  readClipboardText: vi.fn(),
  importFromText: vi.fn(),
  updateSource: vi.fn(),
  refreshSource: vi.fn(),
  deleteSubscription: vi.fn(),
  setPreset: vi.fn(),
  selectRule: vi.fn(),
  updateRule: vi.fn(),
  addRoutingRule: vi.fn(),
  deleteRoutingRule: vi.fn(),
  analyzeRuleMatch: vi.fn(),
  setTheme: vi.fn(),
  updateSettings: vi.fn(),
  onConnectionChanged: vi.fn(),
  onTraffic: vi.fn(),
  onLog: vi.fn(),
  onTestProgress: vi.fn(),
  onTestFinished: vi.fn(),
  onTrayToggleConnection: vi.fn(),
  onTraySelectProfile: vi.fn(),
  startGroupTest: vi.fn(),
  cancelGroupTest: vi.fn(),
  onCrashed: vi.fn(),
  lookupExitLocation: vi.fn(async (): Promise<ExitLocation | null> => null),
}))

vi.mock('@/lib/tauri-api', () => ({ kagerouApi: api }))

vi.mock('@/themes/runtime', () => ({ persistThemeId: vi.fn() }))

vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }))

const { useKagerouStore, __resetBackendEventSubscriptionForTests } = await import('@/store/kagerou-store')

const emptySnapshot: AppSnapshot = {
  connected: false,
  connectedSince: null,
  activeProfileId: '',
  profiles: [],
  profileGroups: [],
  sources: [],
  routingPresets: [],
  routingRules: [],
  settings: {
    theme: 'catppuccin-mocha',
    language: 'en',
    startup: false,
    geoLookup: true,
    tunMode: false,
    systemProxy: false,
    autoConnect: false,
    tunInterface: 'utun / tun0',
    autoUpdateSubscriptions: false,
    subscriptionUpdateInterval: '30',
    customSubscriptionUpdateMinutes: 60,
    groupSort: 'ping',
    logLevel: 'info',
    testUrl: 'http://www.gstatic.com/generate_204',
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
  url: { value: 'Not tested', tone: 'muted' },
  key: 'vless://p1',
  ...overrides,
})

const initialState = useKagerouStore.getState()

beforeEach(() => {
  useKagerouStore.setState(initialState, true)
  __resetBackendEventSubscriptionForTests()
  Object.values(api).forEach((fn) => fn.mockReset())
  vi.mocked(toast.error).mockClear()
  api.getAppState.mockResolvedValue(emptySnapshot)
  api.checkForUpdate.mockResolvedValue(null)
  api.onConnectionChanged.mockResolvedValue(() => {})
  api.onTraffic.mockResolvedValue(() => {})
  api.onTestProgress.mockResolvedValue(() => {})
  api.onTestFinished.mockResolvedValue(() => {})
  api.onTrayToggleConnection.mockResolvedValue(() => {})
  api.onTraySelectProfile.mockResolvedValue(() => {})
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

  it('takes connected from the snapshot: the startup auto-connect event fires before anyone is listening', async () => {
    api.getAppState.mockResolvedValue({ ...emptySnapshot, connected: true })

    expect(useKagerouStore.getState().connected).toBe(false)
    await useKagerouStore.getState().hydrate()

    expect(useKagerouStore.getState().connected).toBe(true)
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
    expect(toast.error).toHaveBeenCalledWith('sing-box not running')
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

    expect(toast.error).toHaveBeenCalledWith('not found')
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

describe('tray intents', () => {
  it('the tray connect item goes through the same toggle the button does', async () => {
    let fire: () => void = () => {}
    api.onTrayToggleConnection.mockImplementation((h: () => void) => { fire = h; return Promise.resolve(() => {}) })
    api.connect.mockResolvedValue(undefined)

    await useKagerouStore.getState().hydrate()
    useKagerouStore.setState({ connected: false })
    fire()
    await vi.waitFor(() => expect(api.connect).toHaveBeenCalled())
  })

  it('the tray profile item goes through the same select the list does', async () => {
    let fire: (id: string) => void = () => {}
    api.onTraySelectProfile.mockImplementation((h: (id: string) => void) => { fire = h; return Promise.resolve(() => {}) })
    api.selectProfile.mockResolvedValue(undefined)

    await useKagerouStore.getState().hydrate()
    useKagerouStore.setState({ profiles: [profile({ id: 'p1' })] })
    fire('p1')
    await vi.waitFor(() => expect(api.selectProfile).toHaveBeenCalledWith('p1'))
  })

  it('a tray entry for a profile that is gone does nothing', async () => {
    let fire: (id: string) => void = () => {}
    api.onTraySelectProfile.mockImplementation((h: (id: string) => void) => { fire = h; return Promise.resolve(() => {}) })

    await useKagerouStore.getState().hydrate()
    useKagerouStore.setState({ profiles: [profile({ id: 'p1' })] })
    fire('deleted-while-the-menu-was-open')

    expect(api.selectProfile).not.toHaveBeenCalled()
    expect(useKagerouStore.getState().activeProfileId).not.toBe('deleted-while-the-menu-was-open')
  })
})

describe('group test run', () => {
  it('shows the run on the click, before the first result arrives', async () => {
    api.startGroupTest.mockResolvedValue(12)

    const started = useKagerouStore.getState().startGroupTest('g1')
    expect(useKagerouStore.getState().testRun).toEqual({ groupId: 'g1', done: 0, total: 0 })
    await started
  })

  it('clears the run when the group turns out to be empty', async () => {
    api.startGroupTest.mockResolvedValue(0)
    await useKagerouStore.getState().startGroupTest('empty')
    expect(useKagerouStore.getState().testRun).toBeNull()
  })

  it('clears the run when the backend refuses to start one', async () => {
    api.startGroupTest.mockRejectedValue('a test run is already in progress')
    await useKagerouStore.getState().startGroupTest('g1')
    expect(useKagerouStore.getState().testRun).toBeNull()
    expect(toast.error).toHaveBeenCalledWith('a test run is already in progress')
  })

  it('a progress event advances the count and applies the profile result', async () => {
    let handler: (event: { profileId: string; result: TestResult; done: number; total: number }) => void = () => {}
    api.onTestProgress.mockImplementation((h: typeof handler) => { handler = h; return Promise.resolve(() => {}) })
    api.startGroupTest.mockResolvedValue(2)

    await useKagerouStore.getState().hydrate()
    useKagerouStore.setState({ profiles: [profile({ id: 'p1' })] })
    await useKagerouStore.getState().startGroupTest('g1')
    handler({ profileId: 'p1', result: { value: '42 ms', tone: 'good' }, done: 1, total: 2 })

    expect(useKagerouStore.getState().testRun).toEqual({ groupId: 'g1', done: 1, total: 2 })
    expect(useKagerouStore.getState().profiles[0].url).toEqual({ value: '42 ms', tone: 'good' })
  })

  it('a progress event for an unknown profile does not invent one', async () => {
    let handler: (event: { profileId: string; result: TestResult; done: number; total: number }) => void = () => {}
    api.onTestProgress.mockImplementation((h: typeof handler) => { handler = h; return Promise.resolve(() => {}) })

    await useKagerouStore.getState().hydrate()
    useKagerouStore.setState({ profiles: [profile({ id: 'p1' })] })
    handler({ profileId: 'ghost', result: { value: '9 ms', tone: 'good' }, done: 1, total: 1 })

    expect(useKagerouStore.getState().profiles).toHaveLength(1)
    expect(useKagerouStore.getState().profiles[0].url).toEqual({ value: 'Not tested', tone: 'muted' })
  })

  it('the finished event ends the run, cancelled or not', async () => {
    let finish: () => void = () => {}
    api.onTestFinished.mockImplementation((h: () => void) => { finish = h; return Promise.resolve(() => {}) })
    api.startGroupTest.mockResolvedValue(3)

    await useKagerouStore.getState().hydrate()
    await useKagerouStore.getState().startGroupTest('g1')
    expect(useKagerouStore.getState().testRun).not.toBeNull()

    finish()
    expect(useKagerouStore.getState().testRun).toBeNull()
  })
})

describe('runProfileTest', () => {
  it('applies the returned result to the matching profile and returns it', async () => {
    useKagerouStore.setState({ profiles: [profile({ id: 'p1' })] })
    api.runProfileTest.mockResolvedValue({ value: '42 ms', tone: 'good' })

    const result = await useKagerouStore.getState().runProfileTest('p1')

    expect(result).toEqual({ value: '42 ms', tone: 'good' })
    expect(useKagerouStore.getState().profiles[0].url).toEqual({ value: '42 ms', tone: 'good' })
  })

  it('returns null and leaves the profile untouched when the backend call fails', async () => {
    const original = profile({ id: 'p1' })
    useKagerouStore.setState({ profiles: [original] })
    api.runProfileTest.mockRejectedValue(new Error('not connected'))

    const result = await useKagerouStore.getState().runProfileTest('p1')

    expect(result).toBeNull()
    expect(useKagerouStore.getState().profiles[0]).toEqual(original)
  })
})

describe('clearGroupTestResults', () => {
  it('calls the backend for the group and refreshes from the snapshot', async () => {
    const refreshed = [profile({ id: 'p1' })]
    api.clearGroupTestResults.mockResolvedValue(undefined)
    api.getAppState.mockResolvedValue({ ...emptySnapshot, profiles: refreshed })

    await useKagerouStore.getState().clearGroupTestResults('g1')

    expect(api.clearGroupTestResults).toHaveBeenCalledWith('g1')
    expect(useKagerouStore.getState().profiles).toEqual(refreshed)
  })

  it('keeps state when the backend call fails', async () => {
    const original = [profile({ id: 'p1' })]
    useKagerouStore.setState({ profiles: original })
    api.clearGroupTestResults.mockRejectedValue(new Error('boom'))

    await useKagerouStore.getState().clearGroupTestResults('g1')

    expect(useKagerouStore.getState().profiles).toEqual(original)
    expect(toast.error).toHaveBeenCalledWith('boom')
  })
})

describe('deleteUnavailableProfiles', () => {
  it('returns the deleted count and refreshes from the snapshot', async () => {
    api.deleteUnavailableProfiles.mockResolvedValue(3)
    api.getAppState.mockResolvedValue({ ...emptySnapshot, profiles: [profile({ id: 'p1' })] })

    const deleted = await useKagerouStore.getState().deleteUnavailableProfiles('g1')

    expect(deleted).toBe(3)
    expect(api.deleteUnavailableProfiles).toHaveBeenCalledWith('g1')
    expect(useKagerouStore.getState().profiles.map((p) => p.id)).toEqual(['p1'])
  })

  it('returns 0 and keeps state when the backend call fails', async () => {
    const original = [profile({ id: 'p1' })]
    useKagerouStore.setState({ profiles: original })
    api.deleteUnavailableProfiles.mockRejectedValue(new Error('boom'))

    const deleted = await useKagerouStore.getState().deleteUnavailableProfiles('g1')

    expect(deleted).toBe(0)
    expect(useKagerouStore.getState().profiles).toEqual(original)
    expect(toast.error).toHaveBeenCalledWith('boom')
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

describe('backend failure reporting', () => {
  it('toggleConnection uses the disconnect fallback when the rejection carries no message', async () => {
    useKagerouStore.setState({ connected: true })
    api.disconnect.mockRejectedValue(undefined)

    await useKagerouStore.getState().toggleConnection()

    expect(toast.error).toHaveBeenCalledWith('Could not disconnect.')
  })

  it('reports a failed profile deletion', async () => {
    api.deleteProfile.mockRejectedValue('the profile is in use')
    await useKagerouStore.getState().deleteProfile('p1')
    expect(toast.error).toHaveBeenCalledWith('the profile is in use')
  })

  it('reports a failed test cancellation', async () => {
    api.cancelGroupTest.mockRejectedValue('no test run')
    await useKagerouStore.getState().cancelGroupTest()
    expect(toast.error).toHaveBeenCalledWith('no test run')
  })

  it('rolls the group panel back from the snapshot when the open state fails to persist', async () => {
    const stored: ProfileGroup[] = [{ id: 'g1', label: 'G1', kind: 'custom', profileIds: [], open: false }]
    useKagerouStore.setState({ profileGroups: [{ ...stored[0] }] })
    api.setProfileGroupOpen.mockRejectedValue(new Error('db locked'))
    api.getAppState.mockResolvedValue({ ...emptySnapshot, profileGroups: stored })

    useKagerouStore.getState().setProfileGroupOpen('g1', true)
    expect(useKagerouStore.getState().profileGroups[0].open).toBe(true)

    await vi.waitFor(() => expect(useKagerouStore.getState().profileGroups[0].open).toBe(false))
    expect(toast.error).toHaveBeenCalledWith('db locked')
  })

  it('rolls the preset toggle back from the snapshot when persistence fails', async () => {
    const stored = [{ id: 'bypass-lan', label: 'Bypass LAN', description: 'Private ranges go direct', enabled: false }]
    useKagerouStore.setState({ routingPresets: [{ ...stored[0] }] })
    api.setPreset.mockRejectedValue(new Error('db locked'))
    api.getAppState.mockResolvedValue({ ...emptySnapshot, routingPresets: stored })

    useKagerouStore.getState().setPreset('bypass-lan', true)
    expect(useKagerouStore.getState().routingPresets[0].enabled).toBe(true)

    await vi.waitFor(() => expect(useKagerouStore.getState().routingPresets[0].enabled).toBe(false))
    expect(toast.error).toHaveBeenCalledWith('db locked')
  })

  it('reverts the rule selection from the snapshot when the backend refuses it', async () => {
    const stored: RoutingRule[] = [
      { id: 'r1', match: 'a.com', outbound: 'Direct', selected: true },
      { id: 'r2', match: 'b.com', outbound: 'Proxy', selected: false },
    ]
    useKagerouStore.setState({ routingRules: stored.map((rule) => ({ ...rule })) })
    api.selectRule.mockRejectedValue(new Error('db locked'))
    api.getAppState.mockResolvedValue({ ...emptySnapshot, routingRules: stored })

    useKagerouStore.getState().selectRule('r2')
    expect(useKagerouStore.getState().routingRules.find((rule) => rule.id === 'r2')?.selected).toBe(true)

    await vi.waitFor(() => expect(useKagerouStore.getState().routingRules.find((rule) => rule.id === 'r1')?.selected).toBe(true))
    expect(toast.error).toHaveBeenCalledWith('db locked')
  })

  it('rolls the rule and the pending-changes flag back when the update fails to persist', async () => {
    const stored: RoutingRule[] = [{ id: 'r1', match: 'a.com', outbound: 'Direct', selected: false }]
    useKagerouStore.setState({ routingRules: stored.map((rule) => ({ ...rule })), connected: true, rulesChangedSinceConnect: false })
    api.updateRule.mockRejectedValue(new Error('db locked'))
    api.getAppState.mockResolvedValue({ ...emptySnapshot, routingRules: stored, connected: true })

    useKagerouStore.getState().updateRule('r1', { outbound: 'Block' })
    expect(useKagerouStore.getState().rulesChangedSinceConnect).toBe(true)
    expect(useKagerouStore.getState().routingRules[0].outbound).toBe('Block')

    await vi.waitFor(() => expect(useKagerouStore.getState().rulesChangedSinceConnect).toBe(false))
    expect(useKagerouStore.getState().routingRules[0].outbound).toBe('Direct')
    expect(toast.error).toHaveBeenCalledWith('db locked')
  })

  it('reverts the theme and the persisted id when the backend refuses the change', async () => {
    useKagerouStore.setState({ settings: { ...initialState.settings, theme: 'catppuccin-mocha' } })
    api.setTheme.mockRejectedValue(new Error('db locked'))
    api.getAppState.mockResolvedValue({ ...emptySnapshot, settings: { ...emptySnapshot.settings, theme: 'catppuccin-mocha' } })

    useKagerouStore.getState().setTheme('kanagawa-wave')
    expect(useKagerouStore.getState().settings.theme).toBe('kanagawa-wave')

    await vi.waitFor(() => expect(useKagerouStore.getState().settings.theme).toBe('catppuccin-mocha'))
    expect(persistThemeId).toHaveBeenLastCalledWith('catppuccin-mocha')
    expect(toast.error).toHaveBeenCalledWith('db locked')
  })

  it('rolls the settings back from the snapshot when persistence fails', async () => {
    useKagerouStore.setState({ settings: { ...initialState.settings, testUrl: 'http://changed.example/204' } })
    api.updateSettings.mockRejectedValue(new Error('db locked'))
    api.getAppState.mockResolvedValue(emptySnapshot)

    useKagerouStore.getState().updateSettings({ testUrl: 'http://changed.example/204' })
    expect(useKagerouStore.getState().settings.testUrl).toBe('http://changed.example/204')

    await vi.waitFor(() => expect(useKagerouStore.getState().settings.testUrl).toBe('http://www.gstatic.com/generate_204'))
    expect(toast.error).toHaveBeenCalledWith('db locked')
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

  it('addRule refetches so the new rule arrives with its backend id', async () => {
    api.addRoutingRule.mockResolvedValue('r3')
    api.getAppState.mockResolvedValue({
      ...emptySnapshot,
      routingRules: [...rules, { id: 'r3', match: 'c.com', outbound: 'Block', selected: false }],
    })
    const id = await useKagerouStore.getState().addRule('c.com', 'Block')
    expect(id).toBe('r3')
    expect(useKagerouStore.getState().routingRules.map((r) => r.id)).toEqual(['r1', 'r2', 'r3'])
  })

  it('addRule keeps the rules it has when the backend refuses', async () => {
    useKagerouStore.setState({ routingRules: rules })
    api.addRoutingRule.mockRejectedValue(new Error('nope'))
    expect(await useKagerouStore.getState().addRule('c.com', 'Block')).toBeNull()
    expect(useKagerouStore.getState().routingRules).toHaveLength(2)
  })

  it('deleteRule drops the rule and leaves the rest alone', async () => {
    useKagerouStore.setState({ routingRules: rules })
    api.deleteRoutingRule.mockResolvedValue(undefined)
    expect(await useKagerouStore.getState().deleteRule('r1')).toBe(true)
    expect(useKagerouStore.getState().routingRules.map((r) => r.id)).toEqual(['r2'])
  })

  it('deleteRule keeps the rule when the backend refuses', async () => {
    useKagerouStore.setState({ routingRules: rules })
    api.deleteRoutingRule.mockRejectedValue(new Error('nope'))
    expect(await useKagerouStore.getState().deleteRule('r1')).toBe(false)
    expect(useKagerouStore.getState().routingRules).toHaveLength(2)
  })
})

describe('rules changed since connect', () => {
  const rules: RoutingRule[] = [{ id: 'r1', match: 'a.com', outbound: 'Direct', selected: false }]

  it('stays false while the connection is down, since nothing is running yet', () => {
    useKagerouStore.setState({ routingRules: rules, connected: false, rulesChangedSinceConnect: false })
    useKagerouStore.getState().updateRule('r1', { outbound: 'Block' })
    expect(useKagerouStore.getState().rulesChangedSinceConnect).toBe(false)
  })

  it('flips once a rule changes on a live connection', async () => {
    useKagerouStore.setState({ routingRules: rules, connected: true, rulesChangedSinceConnect: false })
    useKagerouStore.getState().updateRule('r1', { outbound: 'Block' })
    expect(useKagerouStore.getState().rulesChangedSinceConnect).toBe(true)

    api.deleteRoutingRule.mockResolvedValue(undefined)
    useKagerouStore.setState({ rulesChangedSinceConnect: false })
    await useKagerouStore.getState().deleteRule('r1')
    expect(useKagerouStore.getState().rulesChangedSinceConnect).toBe(true)
  })

  it('clears when a new connection comes up on a freshly generated config', async () => {
    let handler: (connected: boolean) => void = () => {}
    api.onConnectionChanged.mockImplementation((h: (c: boolean) => void) => { handler = h; return Promise.resolve(() => {}) })
    await useKagerouStore.getState().hydrate()
    useKagerouStore.setState({ rulesChangedSinceConnect: true })
    handler(true)
    expect(useKagerouStore.getState().rulesChangedSinceConnect).toBe(false)
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
    handler({ kind: 'sample', up: 1, down: 2, uploadTotal: null, downloadTotal: null, activeConnections: null })
    handler({ kind: 'sample', up: 64, down: 128, uploadTotal: null, downloadTotal: null, activeConnections: null })

    expect(useKagerouStore.getState().trafficSample).toEqual({ download: 128, upload: 64 })
  })

  it('traffic history keeps the last minute of samples, oldest first', async () => {
    let handler: (event: TrafficEvent) => void = () => {}
    api.onTraffic.mockImplementation((h: (e: TrafficEvent) => void) => { handler = h; return Promise.resolve(() => {}) })

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

    await useKagerouStore.getState().hydrate()
    handler({ kind: 'sample', up: 1, down: 2, uploadTotal: null, downloadTotal: null, activeConnections: 12 })
    handler({ kind: 'sample', up: 1, down: 2, uploadTotal: null, downloadTotal: null, activeConnections: null })

    expect(useKagerouStore.getState().activeConnections).toBe(12)
  })

  it('connecting asks where the tunnel comes out', async () => {
    let connection: (connected: boolean) => void = () => {}
    api.onConnectionChanged.mockImplementation((h: (c: boolean) => void) => { connection = h; return Promise.resolve(() => {}) })
    api.lookupExitLocation.mockResolvedValue({ ip: '81.2.69.142', city: 'London', country: 'United Kingdom', countryCode: 'GB' })

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

    await useKagerouStore.getState().hydrate()
    handler({ kind: 'sample', up: 1, down: 2, uploadTotal: 250_000_000, downloadTotal: 1_900_000_000, activeConnections: null })
    handler({ kind: 'sample', up: 3, down: 4, uploadTotal: 260_000_000, downloadTotal: 1_950_000_000, activeConnections: null })

    expect(useKagerouStore.getState().sessionTraffic).toEqual({ download: 1_950_000_000, upload: 260_000_000 })
  })

  it('a sample with null totals keeps the previous sessionTraffic instead of blanking it', async () => {
    let handler: (event: TrafficEvent) => void = () => {}
    api.onTraffic.mockImplementation((h: (e: TrafficEvent) => void) => { handler = h; return Promise.resolve(() => {}) })
    useKagerouStore.setState({ sessionTraffic: { download: 500, upload: 100 } })

    await useKagerouStore.getState().hydrate()
    handler({ kind: 'sample', up: 1, down: 2, uploadTotal: null, downloadTotal: null, activeConnections: null })

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

describe('importing pasted text', () => {
  const outcome: ImportOutcome = { kind: 'profileAdded', profileId: 'p1', name: 'Tokyo' }

  it('refreshes state and reports what the text became', async () => {
    api.importFromText.mockResolvedValue(outcome)
    api.getAppState.mockResolvedValue({ ...emptySnapshot, profiles: [profile({ name: 'Tokyo' })] })

    const attempt = await useKagerouStore.getState().importText('vless://tokyo')

    expect(api.importFromText).toHaveBeenCalledWith('vless://tokyo')
    expect(attempt).toEqual({ status: 'imported', outcome })
    expect(useKagerouStore.getState().profiles.map((p) => p.name)).toEqual(['Tokyo'])
  })

  it('hands the text back with the backend reason when the import fails', async () => {
    api.importFromText.mockRejectedValue('could not recognize subscription format')

    const attempt = await useKagerouStore.getState().importText('garbage')

    expect(attempt).toEqual({ status: 'failed', text: 'garbage', error: 'could not recognize subscription format' })
    expect(api.getAppState).not.toHaveBeenCalled()
    expect(toast.error).not.toHaveBeenCalled()
  })

  it('imports whatever the clipboard holds', async () => {
    api.readClipboardText.mockResolvedValue('https://sub.example/list\n')
    api.importFromText.mockResolvedValue(outcome)

    const attempt = await useKagerouStore.getState().importFromClipboard()

    expect(api.importFromText).toHaveBeenCalledWith('https://sub.example/list\n')
    expect(attempt.status).toBe('imported')
  })

  it('treats a blank clipboard as nothing to import, without asking the backend', async () => {
    api.readClipboardText.mockResolvedValue('   \n')

    const attempt = await useKagerouStore.getState().importFromClipboard()

    expect(attempt).toEqual({ status: 'failed', text: '', error: '' })
    expect(api.importFromText).not.toHaveBeenCalled()
  })

  it('treats an unreadable clipboard the same way', async () => {
    api.readClipboardText.mockRejectedValue('clipboard contents were not available')

    const attempt = await useKagerouStore.getState().importFromClipboard()

    expect(attempt).toEqual({ status: 'failed', text: '', error: '' })
    expect(api.importFromText).not.toHaveBeenCalled()
  })
})

describe('subscription actions', () => {
  it('updateSource returns false when the backend refuses the URL', async () => {
    api.updateSource.mockRejectedValue('invalid input: not an http(s) URL')
    const ok = await useKagerouStore.getState().updateSource('s1', { value: 'vless://nope' })
    expect(ok).toBe(false)
  })

  it('deleteSubscription refreshes state on success', async () => {
    const group: ProfileGroup = { id: 'sub', label: 'Work', kind: 'subscription', profileIds: ['p1'], open: true, sourceId: 's1' }
    useKagerouStore.setState({ profileGroups: [group], profiles: [profile({ groupId: 'sub', origin: 'imported' })] })
    api.deleteSubscription.mockResolvedValue(undefined)

    const ok = await useKagerouStore.getState().deleteSubscription('sub')

    expect(ok).toBe(true)
    expect(api.deleteSubscription).toHaveBeenCalledWith('sub')
    expect(useKagerouStore.getState().profileGroups).toEqual([])
    expect(useKagerouStore.getState().profiles).toEqual([])
  })

  it('deleteSubscription reports the backend reason and keeps the group', async () => {
    const group: ProfileGroup = { id: 'sub', label: 'Work', kind: 'subscription', profileIds: ['p1'], open: true, sourceId: 's1' }
    useKagerouStore.setState({ profileGroups: [group] })
    api.deleteSubscription.mockRejectedValue('switch to a VPN outside this subscription or disconnect before deleting it')

    const ok = await useKagerouStore.getState().deleteSubscription('sub')

    expect(ok).toBe(false)
    expect(toast.error).toHaveBeenCalledWith('switch to a VPN outside this subscription or disconnect before deleting it')
    expect(useKagerouStore.getState().profileGroups).toEqual([group])
  })
})
