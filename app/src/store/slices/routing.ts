import { kagerouApi } from '@/lib/tauri-api'
import type { Outbound, RoutingPreset, RoutingRule } from '@/types/kagerou'

import { refresh, report, type Slice } from '../shared'

export interface RoutingSlice {
  routingPresets: RoutingPreset[]
  routingRules: RoutingRule[]
  /** Rules were edited after the current connection came up, so the running
   * core is still on the config generated at connect time. */
  rulesChangedSinceConnect: boolean
  setPreset: (id: string, enabled: boolean) => void
  selectRule: (id: string) => void
  updateRule: (id: string, patch: Partial<Pick<RoutingRule, 'match' | 'outbound'>>) => void
  addRule: (match: string, outbound: Outbound) => Promise<string | null>
  deleteRule: (id: string) => Promise<boolean>
}

/** Where traffic goes. Every change here sets `rulesChangedSinceConnect`
 * while connected: the core read its rules once, at connect time. */
export const createRoutingSlice: Slice<RoutingSlice> = (set, get) => ({
  routingPresets: [],
  routingRules: [],
  rulesChangedSinceConnect: false,

  setPreset: (id, enabled) => {
    set((state) => ({
      routingPresets: state.routingPresets.map((preset) =>
        preset.id === id ? { ...preset, enabled } : preset,
      ),
    }))
    void kagerouApi.setPreset(id, enabled).catch(async (error) => {
      report(error, 'common:feedback.presetSaveFailed')
      await refresh(set)
    })
  },

  selectRule: (id) => {
    set((state) => ({
      routingRules: state.routingRules.map((rule) => ({ ...rule, selected: rule.id === id })),
    }))
    void kagerouApi.selectRule(id).catch(async (error) => {
      report(error, 'common:feedback.ruleSelectFailed')
      await refresh(set)
    })
  },

  updateRule: (id, patch) => {
    const rulesChangedBefore = get().rulesChangedSinceConnect
    set((state) => ({
      routingRules: state.routingRules.map((rule) =>
        rule.id === id ? { ...rule, ...patch } : rule,
      ),
      rulesChangedSinceConnect: state.connected || state.rulesChangedSinceConnect,
    }))
    void kagerouApi.updateRule(id, patch).catch(async (error) => {
      report(error, 'common:feedback.ruleSaveFailed')
      // refresh() restores the rules but not this frontend-only flag; the
      // failed write changed nothing the core would need to reload.
      set({ rulesChangedSinceConnect: rulesChangedBefore })
      await refresh(set)
    })
  },

  // Waits for the backend because the new rule's id comes from there.
  addRule: async (match, outbound) => {
    try {
      const id = await kagerouApi.addRoutingRule(match, outbound)
      await refresh(set)
      set((state) => ({
        rulesChangedSinceConnect: state.connected || state.rulesChangedSinceConnect,
      }))
      return id
    } catch {
      return null
    }
  },

  deleteRule: async (id) => {
    try {
      await kagerouApi.deleteRoutingRule(id)
      set((state) => ({
        routingRules: state.routingRules.filter((rule) => rule.id !== id),
        rulesChangedSinceConnect: state.connected || state.rulesChangedSinceConnect,
      }))
      return true
    } catch {
      return false
    }
  },
})
