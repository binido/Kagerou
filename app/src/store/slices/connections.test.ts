import { beforeEach, describe, expect, it, vi } from 'vitest'

import { toast } from 'sonner'

vi.mock('@/lib/tauri-api', async () => ({
  kagerouApi: (await import('../test-api')).kagerouApiMock,
}))
vi.mock('@/themes/runtime', () => ({ persistThemeId: vi.fn() }))
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }))

import { kagerouApiMock as api, resetApiMock } from '../test-api'
import type { LiveConnection } from '@/types/kagerou'

import { connectionStartMillis } from './connections'

const { useKagerouStore, subscribeToBackendEvents, __resetBackendEventSubscriptionForTests } =
  await import('../kagerou-store')

const initialState = useKagerouStore.getState()

const connection = (id: string, start: string): LiveConnection => ({
  id,
  host: `${id}.example`,
  port: '443',
  network: 'tcp',
  rule: 'final',
  exit: { kind: 'direct' },
  upload: 0,
  download: 0,
  start,
})

const older = connection('a', '2026-09-23T10:00:00.5+03:00')
const newer = connection('b', '2026-09-23T10:00:01.123456789+03:00')

beforeEach(() => {
  useKagerouStore.setState(initialState, true)
  __resetBackendEventSubscriptionForTests()
  resetApiMock()
  vi.mocked(toast.error).mockClear()
})

describe('connectionStartMillis', () => {
  it('reads the nanosecond times Go writes', () => {
    expect(connectionStartMillis('2026-09-23T10:00:01.123456789+03:00')).toBe(
      Date.UTC(2026, 8, 23, 7, 0, 1, 123),
    )
  })

  it('reads times with a short fraction or none', () => {
    expect(connectionStartMillis('2026-09-23T10:00:00.5+03:00')).toBe(
      Date.UTC(2026, 8, 23, 7, 0, 0, 500),
    )
    expect(connectionStartMillis('2026-09-23T07:00:00Z')).toBe(Date.UTC(2026, 8, 23, 7, 0, 0))
  })
})

describe('connection list actions', () => {
  it('refreshConnections lists the newest connection first', async () => {
    api.listConnections.mockResolvedValue([older, newer])
    await useKagerouStore.getState().refreshConnections()
    expect(useKagerouStore.getState().liveConnections.map((c) => c.id)).toEqual(['b', 'a'])
  })

  it('refreshConnections keeps the previous list and stays quiet when the read fails', async () => {
    useKagerouStore.setState({ liveConnections: [older] })
    api.listConnections.mockRejectedValue({ code: 'network', detail: 'refused' })
    await useKagerouStore.getState().refreshConnections()
    expect(useKagerouStore.getState().liveConnections).toEqual([older])
    expect(toast.error).not.toHaveBeenCalled()
  })

  it('closeConnection drops the row before the backend answers', async () => {
    useKagerouStore.setState({ liveConnections: [newer, older] })
    let resolve = () => {}
    api.closeConnection.mockReturnValue(new Promise<void>((r) => (resolve = r)))
    const closing = useKagerouStore.getState().closeConnection('a')
    expect(useKagerouStore.getState().liveConnections.map((c) => c.id)).toEqual(['b'])
    resolve()
    await closing
    expect(api.closeConnection).toHaveBeenCalledWith('a')
  })

  it('closeConnection reports a failure and re-reads the real list', async () => {
    useKagerouStore.setState({ liveConnections: [newer, older] })
    api.closeConnection.mockRejectedValue({ code: 'network', detail: 'refused' })
    api.listConnections.mockResolvedValue([older, newer])
    await useKagerouStore.getState().closeConnection('a')
    expect(toast.error).toHaveBeenCalledOnce()
    expect(useKagerouStore.getState().liveConnections.map((c) => c.id)).toEqual(['b', 'a'])
  })

  it('closeAllConnections reports a failure and re-reads the real list', async () => {
    useKagerouStore.setState({ liveConnections: [newer] })
    api.closeAllConnections.mockRejectedValue({ code: 'network', detail: 'refused' })
    api.listConnections.mockResolvedValue([newer])
    await useKagerouStore.getState().closeAllConnections()
    expect(toast.error).toHaveBeenCalledOnce()
    expect(useKagerouStore.getState().liveConnections).toEqual([newer])
  })

  it('a disconnect clears the list', async () => {
    let onConnectionChanged: (connected: boolean) => void = () => {}
    api.onConnectionChanged.mockImplementation(async (handler) => {
      onConnectionChanged = handler
      return () => {}
    })
    subscribeToBackendEvents()
    useKagerouStore.setState({ connected: true, liveConnections: [older] })
    onConnectionChanged(false)
    expect(useKagerouStore.getState().liveConnections).toEqual([])
  })
})
