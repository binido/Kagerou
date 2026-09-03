import { useState } from 'react'

import { Card } from '@/components/ui/card'
import { EditRuleDialog } from '@/components/routing/EditRuleDialog'
import { PresetSwitchRow } from '@/components/routing/PresetSwitchRow'
import { RoutingRulesTable } from '@/components/routing/RoutingRulesTable'
import { PageHeader } from '@/components/layout/PageHeader'
import { useKagerouStore } from '@/store/kagerou-store'
import type { RoutingRule } from '@/types/kagerou'

export function RoutingRulesPage() {
  const presets = useKagerouStore((state) => state.routingPresets)
  const rules = useKagerouStore((state) => state.routingRules)
  const setPreset = useKagerouStore((state) => state.setPreset)
  const selectRule = useKagerouStore((state) => state.selectRule)
  const updateRule = useKagerouStore((state) => state.updateRule)
  const [editingRule, setEditingRule] = useState<RoutingRule | null>(null)

  return (
    <div className="min-h-screen min-w-0 bg-canvas px-6 py-8 lg:px-12 lg:py-10">
      <div className="mx-auto w-full max-w-[1000px]">
        <PageHeader description="Decide which traffic goes direct, through your proxy, or is blocked." eyebrow="Network" status={<span className="flex items-center gap-2 text-[11px] text-quiet"><span aria-hidden="true" className="size-1.5 rounded-full bg-good" />Connection active</span>} title="Routing rules" />
        <section aria-labelledby="presets-heading" className="mt-9"><div className="mb-3 flex items-end justify-between gap-6"><div><p className="type-eyebrow mb-1">Presets</p><h2 className="text-[16px] font-semibold tracking-[-0.015em] text-[#e8e5ee]" id="presets-heading">Common traffic shortcuts</h2></div><p className="text-[11px] text-quiet">Applied before custom rules</p></div><Card className="overflow-hidden rounded-[10px] border-0 bg-surface p-0 shadow-none ring-1 ring-inset ring-white/[0.055]">{presets.map((preset) => <PresetSwitchRow key={preset.id} onChange={(enabled) => setPreset(preset.id, enabled)} preset={preset} />)}</Card></section>
        <section aria-labelledby="rules-heading" className="mt-10"><div className="mb-3 flex items-end justify-between gap-6"><div><p className="type-eyebrow mb-1">Custom rules</p><h2 className="text-[16px] font-semibold tracking-[-0.015em] text-[#e8e5ee]" id="rules-heading">Traffic matches</h2></div><p className="text-[11px] text-quiet">First match wins <span className="px-1 text-quiet">·</span> {rules.length} rules</p></div><RoutingRulesTable onEdit={setEditingRule} onSelect={selectRule} rules={rules} /></section>
        <p className="mt-3 text-[11px] text-quiet">Rules are evaluated from top to bottom.</p>
      </div>
      <EditRuleDialog key={editingRule?.id ?? 'closed'} onOpenChange={(open) => { if (!open) setEditingRule(null) }} onSave={(patch) => { if (editingRule) updateRule(editingRule.id, patch); setEditingRule(null) }} rule={editingRule} />
    </div>
  )
}
