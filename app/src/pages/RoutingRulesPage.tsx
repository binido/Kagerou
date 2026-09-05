import { useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Card } from '@/components/ui/card'
import { EditRuleDialog } from '@/components/routing/EditRuleDialog'
import { PresetSwitchRow } from '@/components/routing/PresetSwitchRow'
import { RoutingRulesTable } from '@/components/routing/RoutingRulesTable'
import { PageContainer } from '@/components/layout/PageContainer'
import { PageHeader } from '@/components/layout/PageHeader'
import { useKagerouStore } from '@/store/kagerou-store'
import type { RoutingRule } from '@/types/kagerou'

export function RoutingRulesPage() {
  const { t } = useTranslation('routing')
  const presets = useKagerouStore((state) => state.routingPresets)
  const rules = useKagerouStore((state) => state.routingRules)
  const setPreset = useKagerouStore((state) => state.setPreset)
  const selectRule = useKagerouStore((state) => state.selectRule)
  const updateRule = useKagerouStore((state) => state.updateRule)
  const [editingRule, setEditingRule] = useState<RoutingRule | null>(null)

  return (
    <PageContainer>
        <PageHeader description={t('page.description')} eyebrow={t('page.eyebrow')} status={<span className="flex items-center gap-2 text-[11px] text-quiet"><span aria-hidden="true" className="size-1.5 rounded-full bg-good" />{t('page.connectionActive')}</span>} title={t('page.title')} />
        <section aria-labelledby="presets-heading" className="mt-9"><div className="mb-3 flex items-end justify-between gap-6"><div><p className="type-eyebrow mb-1">{t('page.presets')}</p><h2 className="text-[16px] font-semibold tracking-[-0.015em] text-primary" id="presets-heading">{t('page.commonShortcuts')}</h2></div><p className="text-[11px] text-quiet">{t('page.appliedBeforeCustom')}</p></div><Card className="gap-0 overflow-hidden rounded-[10px] border-0 bg-surface p-0 shadow-none ring-1 ring-inset ring-hairline">{presets.map((preset) => <PresetSwitchRow key={preset.id} onChange={(enabled) => setPreset(preset.id, enabled)} preset={preset} />)}</Card></section>
      <section aria-labelledby="rules-heading" className="mt-10"><div className="mb-3 flex items-end justify-between gap-6"><div><p className="type-eyebrow mb-1">{t('page.customRules')}</p><h2 className="text-[16px] font-semibold tracking-[-0.015em] text-primary" id="rules-heading">{t('page.trafficMatches')}</h2></div><p className="text-[11px] text-quiet">{t('page.firstMatchWins')} <span className="px-1 text-quiet">·</span> {t('page.rulesCount', { count: rules.length })}</p></div><RoutingRulesTable onEdit={setEditingRule} onSelect={selectRule} rules={rules} /></section>
      <p className="mt-3 text-[11px] text-quiet">{t('page.evaluatedTopBottom')}</p>
      <EditRuleDialog key={editingRule?.id ?? 'closed'} onOpenChange={(open) => { if (!open) setEditingRule(null) }} onSave={(patch) => { if (editingRule) updateRule(editingRule.id, patch); setEditingRule(null) }} rule={editingRule} />
    </PageContainer>
  )
}
