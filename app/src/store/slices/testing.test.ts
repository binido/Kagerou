import { beforeEach, describe, expect, it, vi } from 'vitest'

import { toast } from 'sonner'

vi.mock('@/lib/tauri-api', async () => ({
  kagerouApi: (await import('../test-api')).kagerouApiMock,
}))
vi.mock('@/themes/runtime', () => ({ persistThemeId: vi.fn() }))
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }))

import { kagerouApiMock as api, resetApiMock } from '../test-api'
import { emptySnapshot, profile } from '../test-fixtures'
import en from '@/locales/en/common.json'
import type { TestResult } from '@/types/kagerou'

const { useKagerouStore, subscribeToBackendEvents, __resetBackendEventSubscriptionForTests } =
  await import('../kagerou-store')

const initialState = useKagerouStore.getState()

beforeEach(() => {
  useKagerouStore.setState(initialState, true)
  __resetBackendEventSubscriptionForTests()
  resetApiMock()
  vi.mocked(toast.error).mockClear()
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
    api.startGroupTest.mockRejectedValue({ code: 'testRunInProgress', detail: 'already running' })
    await useKagerouStore.getState().startGroupTest('g1')
    expect(useKagerouStore.getState().testRun).toBeNull()
    expect(toast.error).toHaveBeenCalledWith(en.errors.testRunInProgress)
  })

  it('a progress event advances the count and applies the profile result', async () => {
    let handler: (event: {
      profileId: string
      result: TestResult
      done: number
      total: number
    }) => void = () => {}
    api.onTestProgress.mockImplementation((h: typeof handler) => {
      handler = h
      return Promise.resolve(() => {})
    })
    api.startGroupTest.mockResolvedValue(2)

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    useKagerouStore.setState({ profiles: [profile({ id: 'p1' })] })
    await useKagerouStore.getState().startGroupTest('g1')
    handler({
      profileId: 'p1',
      result: { kind: 'latency', millis: 42, tone: 'good' },
      done: 1,
      total: 2,
    })

    expect(useKagerouStore.getState().testRun).toEqual({ groupId: 'g1', done: 1, total: 2 })
    expect(useKagerouStore.getState().profiles[0].url).toEqual({
      kind: 'latency',
      millis: 42,
      tone: 'good',
    })
  })

  it('a progress event for an unknown profile does not invent one', async () => {
    let handler: (event: {
      profileId: string
      result: TestResult
      done: number
      total: number
    }) => void = () => {}
    api.onTestProgress.mockImplementation((h: typeof handler) => {
      handler = h
      return Promise.resolve(() => {})
    })

    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    useKagerouStore.setState({ profiles: [profile({ id: 'p1' })] })
    handler({
      profileId: 'ghost',
      result: { kind: 'latency', millis: 9, tone: 'good' },
      done: 1,
      total: 1,
    })

    expect(useKagerouStore.getState().profiles).toHaveLength(1)
    expect(useKagerouStore.getState().profiles[0].url).toEqual({ kind: 'notTested', tone: 'muted' })
  })

  it('the finished event ends the run, cancelled or not', async () => {
    let finish: () => void = () => {}
    api.onTestFinished.mockImplementation((h: () => void) => {
      finish = h
      return Promise.resolve(() => {})
    })
    api.startGroupTest.mockResolvedValue(3)

    subscribeToBackendEvents()
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
    api.runProfileTest.mockResolvedValue({ kind: 'latency', millis: 42, tone: 'good' })

    const result = await useKagerouStore.getState().runProfileTest('p1')

    expect(result).toEqual({ kind: 'latency', millis: 42, tone: 'good' })
    expect(useKagerouStore.getState().profiles[0].url).toEqual({
      kind: 'latency',
      millis: 42,
      tone: 'good',
    })
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
    api.clearGroupTestResults.mockRejectedValue({ code: 'storage', detail: 'boom' })

    await useKagerouStore.getState().clearGroupTestResults('g1')

    expect(useKagerouStore.getState().profiles).toEqual(original)
    expect(toast.error).toHaveBeenCalledWith(en.errors.storage)
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
    api.deleteUnavailableProfiles.mockRejectedValue({ code: 'storage', detail: 'boom' })

    const deleted = await useKagerouStore.getState().deleteUnavailableProfiles('g1')

    expect(deleted).toBe(0)
    expect(useKagerouStore.getState().profiles).toEqual(original)
    expect(toast.error).toHaveBeenCalledWith(en.errors.storage)
  })
})
