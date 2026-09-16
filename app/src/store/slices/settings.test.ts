import { beforeEach, describe, expect, it, vi } from 'vitest'

import { toast } from 'sonner'

vi.mock('@/lib/tauri-api', async () => ({
  kagerouApi: (await import('../test-api')).kagerouApiMock,
}))
vi.mock('@/themes/runtime', () => ({ persistThemeId: vi.fn() }))
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }))

import { kagerouApiMock as api, resetApiMock } from '../test-api'

const { useKagerouStore, __resetBackendEventSubscriptionForTests } =
  await import('../kagerou-store')

const initialState = useKagerouStore.getState()

beforeEach(() => {
  useKagerouStore.setState(initialState, true)
  __resetBackendEventSubscriptionForTests()
  resetApiMock()
  vi.mocked(toast.error).mockClear()
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
