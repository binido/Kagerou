import { beforeEach, describe, expect, it, vi } from 'vitest'

import { toast } from 'sonner'

vi.mock('@/lib/tauri-api', async () => ({ kagerouApi: (await import('../test-api')).kagerouApiMock }))
vi.mock('@/themes/runtime', () => ({ persistThemeId: vi.fn() }))
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }))

import { kagerouApiMock as api, resetApiMock } from '../test-api'
import { emptySnapshot, profile } from '../test-fixtures'
import en from '@/locales/en/common.json'
import type { ImportOutcome, ProfileGroup } from '@/types/kagerou'

const { useKagerouStore, __resetBackendEventSubscriptionForTests } =
  await import('../kagerou-store')

const initialState = useKagerouStore.getState()

beforeEach(() => {
  useKagerouStore.setState(initialState, true)
  __resetBackendEventSubscriptionForTests()
  resetApiMock()
  vi.mocked(toast.error).mockClear()
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
    api.importFromText.mockRejectedValue({ code: 'subscriptionInvalid', detail: 'unrecognized format' })

    const attempt = await useKagerouStore.getState().importText('garbage')

    expect(attempt).toEqual({ status: 'failed', text: 'garbage', error: en.errors.subscriptionInvalid })
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
    api.deleteSubscription.mockRejectedValue({ code: 'activeProfileInUse', detail: 'switch away first' })

    const ok = await useKagerouStore.getState().deleteSubscription('sub')

    expect(ok).toBe(false)
    expect(toast.error).toHaveBeenCalledWith(en.errors.activeProfileInUse)
    expect(useKagerouStore.getState().profileGroups).toEqual([group])
  })
})
