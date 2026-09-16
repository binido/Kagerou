import { beforeEach, describe, expect, it, vi } from 'vitest'

import { toast } from 'sonner'

vi.mock('@/lib/tauri-api', async () => ({ kagerouApi: (await import('../test-api')).kagerouApiMock }))
vi.mock('@/themes/runtime', () => ({ persistThemeId: vi.fn() }))
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }))

import { kagerouApiMock as api, resetApiMock } from '../test-api'
import { emptySnapshot } from '../test-fixtures'
import type { RoutingRule } from '@/types/kagerou'

const { useKagerouStore, subscribeToBackendEvents, __resetBackendEventSubscriptionForTests } =
  await import('../kagerou-store')

const initialState = useKagerouStore.getState()

beforeEach(() => {
  useKagerouStore.setState(initialState, true)
  __resetBackendEventSubscriptionForTests()
  resetApiMock()
  vi.mocked(toast.error).mockClear()
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
    subscribeToBackendEvents()
    await useKagerouStore.getState().hydrate()
    useKagerouStore.setState({ rulesChangedSinceConnect: true })
    handler(true)
    expect(useKagerouStore.getState().rulesChangedSinceConnect).toBe(false)
  })
})
