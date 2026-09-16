import { beforeEach, describe, expect, it, vi } from 'vitest'

import { toast } from 'sonner'

vi.mock('@/lib/tauri-api', async () => ({ kagerouApi: (await import('../test-api')).kagerouApiMock }))
vi.mock('@/themes/runtime', () => ({ persistThemeId: vi.fn() }))
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }))

import { kagerouApiMock as api, resetApiMock } from '../test-api'
import { emptySnapshot, profile } from '../test-fixtures'
import en from '@/locales/en/common.json'
import type { AppSnapshot } from '@/lib/tauri-api'

const { useKagerouStore, __resetBackendEventSubscriptionForTests } =
  await import('../kagerou-store')

const initialState = useKagerouStore.getState()

beforeEach(() => {
  useKagerouStore.setState(initialState, true)
  __resetBackendEventSubscriptionForTests()
  resetApiMock()
  vi.mocked(toast.error).mockClear()
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
    api.selectProfile.mockRejectedValue({ code: 'notFound', detail: 'not found' })
    api.getAppState.mockResolvedValue({ ...emptySnapshot, activeProfileId: 'a', profiles: [profile({ id: 'a', selected: true }), profile({ id: 'b' })] })

    await useKagerouStore.getState().selectProfile('b')

    expect(toast.error).toHaveBeenCalledWith(en.errors.notFound)
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
