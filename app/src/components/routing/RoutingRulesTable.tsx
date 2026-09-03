import { useTranslation } from 'react-i18next'
import { ArrowRight, Pencil } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { cn } from '@/lib/utils'
import type { Outbound, RoutingRule } from '@/types/kagerou'

type OutboundTranslationKey = 'table.direct' | 'table.proxy' | 'table.block'

const outboundKeys: Record<Outbound, OutboundTranslationKey> = {
  Direct: 'table.direct',
  Proxy: 'table.proxy',
  Block: 'table.block',
}

interface RoutingRulesTableProps {
  rules: RoutingRule[]
  onSelect: (id: string) => void
  onEdit: (rule: RoutingRule) => void
}

export function RoutingRulesTable({ rules, onSelect, onEdit }: RoutingRulesTableProps) {
  const { t } = useTranslation('routing')

  return (
    <div className="overflow-x-auto rounded-[10px] bg-surface ring-1 ring-inset ring-hairline/55">
      <Table className="min-w-[620px]">
        <TableHeader><TableRow className="border-b border-hairline/55 hover:bg-transparent"><TableHead className="px-5 py-3 text-[10px] font-semibold uppercase tracking-[0.15em] text-muted-copy">{t('table.match')}</TableHead><TableHead className="w-[72px] px-2 text-center"><span className="sr-only">{t('table.direction')}</span></TableHead><TableHead className="w-[180px] px-3 py-3 text-[10px] font-semibold uppercase tracking-[0.15em] text-muted-copy">{t('table.outbound')}</TableHead><TableHead className="w-12 px-3 py-3"><span className="sr-only">{t('table.edit')}</span></TableHead></TableRow></TableHeader>
        <TableBody>
          {rules.map((rule) => (
            <TableRow aria-selected={rule.selected} className={cn('group h-[46px] cursor-pointer border-b border-hairline/55 text-[13px] text-body outline-none transition-colors hover:bg-row-hover focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-lavender', rule.selected && 'bg-selected text-primary hover:bg-selected')} key={rule.id} onClick={(event) => { if (!(event.target as HTMLElement).closest('button')) onSelect(rule.id) }} onKeyDown={(event) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); onSelect(rule.id) } }} tabIndex={0}>
              <TableCell className={cn('truncate px-5 py-2', rule.selected && 'font-medium')}>{rule.match}</TableCell>
              <TableCell className={cn('px-2 text-center', rule.selected ? 'text-lavender-hi' : 'text-quiet')}><ArrowRight aria-hidden="true" className="mx-auto size-4" /></TableCell>
              <TableCell className="px-3 py-2 text-body">{t(outboundKeys[rule.outbound])}</TableCell>
              <TableCell className="px-3 py-2 text-right"><Button aria-label={t('table.editRule', { match: rule.match })} className={cn('size-7 text-lavender-hi transition-opacity hover:bg-row-hover', rule.selected ? 'opacity-100' : 'opacity-0 group-hover:opacity-100 focus-visible:opacity-100')} onClick={() => onEdit(rule)} size="icon-xs" type="button" variant="ghost"><Pencil aria-hidden="true" className="size-3.5" strokeWidth={1.7} /></Button></TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}
