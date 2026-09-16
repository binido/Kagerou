import { beforeEach, describe, expect, it, vi } from 'vitest'

import { toast } from 'sonner'

vi.mock('@/lib/tauri-api', async () => ({ kagerouApi: (await import('../test-api')).kagerouApiMock }))
vi.mock('@/themes/runtime', () => ({ persistThemeId: vi.fn() }))
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }))

import { kagerouApiMock as api, resetApiMock } from '../test-api'
import en from '@/locales/en/common.json'

const { useKagerouStore, __resetBackendEventSubscriptionForTests } =
  await import('../kagerou-store')

const initialState = useKagerouStore.getState()

beforeEach(() => {
  useKagerouStore.setState(initialState, true)
  __resetBackendEventSubscriptionForTests()
  resetApiMock()
  vi.mocked(toast.error).mockClear()
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
    api.connect.mockRejectedValue({ code: 'coreFailed', detail: 'sing-box not running' })
    await expect(useKagerouStore.getState().toggleConnection()).resolves.toBeUndefined()
    expect(toast.error).toHaveBeenCalledWith(en.errors.coreFailed)
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

  it('turns the other mode off when one is turned on, as the backend does', () => {
    api.updateSettings.mockResolvedValue(undefined)
    useKagerouStore.getState().updateSettings({ tunMode: true })
    useKagerouStore.getState().updateSettings({ systemProxy: true })

    expect(useKagerouStore.getState().settings).toMatchObject({ tunMode: false, systemProxy: true })
    expect(api.updateSettings).toHaveBeenLastCalledWith({ systemProxy: true })

    useKagerouStore.getState().updateSettings({ tunMode: true })
    expect(useKagerouStore.getState().settings).toMatchObject({ tunMode: true, systemProxy: false })
  })
})
