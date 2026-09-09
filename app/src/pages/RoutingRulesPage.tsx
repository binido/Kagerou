import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyTitle } from '@/components/ui/empty'
import { DeleteRuleDialog } from '@/components/routing/DeleteRuleDialog'
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
  const connected = useKagerouStore((state) => state.connected)
  const rulesChanged = useKagerouStore((state) => state.rulesChangedSinceConnect)
  const setPreset = useKagerouStore((state) => state.setPreset)
  const selectRule = useKagerouStore((state) => state.selectRule)
  const updateRule = useKagerouStore((state) => state.updateRule)
  const addRule = useKagerouStore((state) => state.addRule)
  const deleteRule = useKagerouStore((state) => state.deleteRule)
  const [editingRule, setEditingRule] = useState<RoutingRule | 'new' | null>(null)
  const [deletingRule, setDeletingRule] = useState<RoutingRule | null>(null)

  const addButton = (
    <Button className="h-8 gap-1.5 bg-lavender px-3 text-[12px] text-ink hover:bg-lavender-hi" onClick={() => setEditingRule('new')} type="button">
      <Plus aria-hidden="true" className="size-3.5" strokeWidth={2} />
      {t('page.addRule')}
    </Button>
  )

  return (
    <PageContainer>
        <PageHeader description={t('page.description')} eyebrow={t('page.eyebrow')} status={<span className="flex items-center gap-2 text-[11px] text-quiet"><span aria-hidden="true" className="size-1.5 rounded-full bg-good" />{t('page.connectionActive')}</span>} title={t('page.title')} />
        <section aria-labelledby="presets-heading" className="mt-9"><div className="mb-3 flex items-end justify-between gap-6"><div><p className="type-eyebrow mb-1">{t('page.presets')}</p><h2 className="text-[16px] font-semibold tracking-[-0.015em] text-primary" id="presets-heading">{t('page.commonShortcuts')}</h2></div><p className="text-[11px] text-quiet">{t('page.appliedBeforeCustom')}</p></div><Card className="gap-0 overflow-hidden rounded-[10px] border-0 bg-surface p-0 shadow-none ring-1 ring-inset ring-hairline">{presets.map((preset) => <PresetSwitchRow key={preset.id} onChange={(enabled) => setPreset(preset.id, enabled)} preset={preset} />)}</Card></section>
      <section aria-labelledby="rules-heading" className="mt-10">
        <div className="mb-3 flex items-end justify-between gap-6"><div><p className="type-eyebrow mb-1">{t('page.customRules')}</p><h2 className="text-[16px] font-semibold tracking-[-0.015em] text-primary" id="rules-heading">{t('page.trafficMatches')}</h2></div><div className="flex items-center gap-4"><p className="text-[11px] text-quiet">{t('page.firstMatchWins')} <span className="px-1 text-quiet">·</span> {t('page.rulesCount', { count: rules.length })}</p>{addButton}</div></div>
        {connected && rulesChanged ? <p className="mb-3 rounded-lg bg-surface px-3 py-2 text-[11px] leading-4 text-warn ring-1 ring-inset ring-hairline" role="status">{t('page.notLiveYet')}</p> : null}
        {rules.length === 0
          ? <Empty className="rounded-[10px] bg-surface py-10 ring-1 ring-inset ring-hairline/55"><EmptyHeader><EmptyTitle className="text-[14px] text-primary">{t('page.emptyTitle')}</EmptyTitle><EmptyDescription className="text-[12px] text-quiet">{t('page.emptyDescription')}</EmptyDescription></EmptyHeader><EmptyContent>{addButton}</EmptyContent></Empty>
          : <RoutingRulesTable onDelete={setDeletingRule} onEdit={setEditingRule} onSelect={selectRule} rules={rules} />}
      </section>
      {rules.length > 0 ? <p className="mt-3 text-[11px] text-quiet">{t('page.evaluatedTopBottom')}</p> : null}
      <EditRuleDialog
        key={editingRule === 'new' ? 'new' : (editingRule?.id ?? 'closed')}
        onOpenChange={(open) => { if (!open) setEditingRule(null) }}
        onSave={(patch) => {
          if (editingRule === 'new') void addRule(patch.match, patch.outbound)
          else if (editingRule) updateRule(editingRule.id, patch)
          setEditingRule(null)
        }}
        rule={editingRule}
      />
      <DeleteRuleDialog
        onConfirm={() => { if (deletingRule) void deleteRule(deletingRule.id); setDeletingRule(null) }}
        onOpenChange={(open) => { if (!open) setDeletingRule(null) }}
        rule={deletingRule}
      />
    </PageContainer>
  )
}
