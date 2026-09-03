import { ArrowRight, Pencil } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { cn } from '@/lib/utils'
import type { RoutingRule } from '@/types/kagerou'

interface RoutingRulesTableProps {
  rules: RoutingRule[]
  onSelect: (id: string) => void
  onEdit: (rule: RoutingRule) => void
}

export function RoutingRulesTable({ rules, onSelect, onEdit }: RoutingRulesTableProps) {
  return (
    <div className="overflow-x-auto rounded-[10px] bg-surface ring-1 ring-inset ring-white/[0.055]">
      <Table className="min-w-[620px]">
        <TableHeader><TableRow className="border-b border-white/[0.055] hover:bg-transparent"><TableHead className="px-5 py-3 text-[10px] font-semibold uppercase tracking-[0.15em] text-muted-copy">Match</TableHead><TableHead className="w-[72px] px-2 text-center"><span className="sr-only">Direction</span></TableHead><TableHead className="w-[180px] px-3 py-3 text-[10px] font-semibold uppercase tracking-[0.15em] text-muted-copy">Outbound</TableHead><TableHead className="w-12 px-3 py-3"><span className="sr-only">Edit</span></TableHead></TableRow></TableHeader>
        <TableBody>
          {rules.map((rule) => (
            <TableRow aria-selected={rule.selected} className={cn('group h-[46px] cursor-pointer border-b border-white/[0.055] text-[13px] text-body outline-none transition-colors hover:bg-row-hover focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-lavender', rule.selected && 'bg-selected text-primary hover:bg-selected')} key={rule.id} onClick={(event) => { if (!(event.target as HTMLElement).closest('button')) onSelect(rule.id) }} onKeyDown={(event) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); onSelect(rule.id) } }} tabIndex={0}>
              <TableCell className={cn('truncate px-5 py-2', rule.selected && 'font-medium')}>{rule.match}</TableCell>
              <TableCell className={cn('px-2 text-center', rule.selected ? 'text-lavender-hi' : 'text-quiet')}><ArrowRight aria-hidden="true" className="mx-auto size-4" /></TableCell>
              <TableCell className="px-3 py-2 text-[#d9d5e1]">{rule.outbound}</TableCell>
              <TableCell className="px-3 py-2 text-right"><Button aria-label={`Edit ${rule.match}`} className={cn('size-7 text-lavender-hi transition-opacity hover:bg-white/6', rule.selected ? 'opacity-100' : 'opacity-0 group-hover:opacity-100 focus-visible:opacity-100')} onClick={() => onEdit(rule)} size="icon-xs" type="button" variant="ghost"><Pencil aria-hidden="true" className="size-3.5" strokeWidth={1.7} /></Button></TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}
