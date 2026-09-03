import { KeyRound, Link2, MoreHorizontal, Pencil, RefreshCw, Rss, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { cn } from '@/lib/utils'
import type { Source } from '@/types/kagerou'

type SourceStatusKey = 'status.upToDate' | 'status.ready' | 'status.refreshDue' | 'status.updating'
type SourceTimestampKey = 'card.updatedJustNow' | 'card.addedJustNow' | 'card.checkedJustNow' | 'card.updatedMinutesAgo' | 'card.updatedDaysAgo'
type LocalizedTimestamp = SourceTimestampKey | { key: 'card.updatedMinutesAgo' | 'card.updatedDaysAgo'; count: number }

interface SourceCardProps {
  source: Source
  profileCount: number
  refreshing: boolean
  onRefresh: () => void
  onEdit: () => void
  onRemove: () => void
}

const sourceStatusKey = (status: Source['status']): SourceStatusKey => {
  switch (status) {
    case 'up-to-date': return 'status.upToDate'
    case 'ready': return 'status.ready'
    case 'refresh-due': return 'status.refreshDue'
    case 'updating': return 'status.updating'
  }
}

const sourceTimestampKey = (value: string): LocalizedTimestamp | null => {
  if (value === 'Updated just now') return 'card.updatedJustNow'
  if (value === 'Added just now') return 'card.addedJustNow'
  if (value === 'Checked just now') return 'card.checkedJustNow'
  const minuteMatch = value.match(/^Updated (\d+) min ago$/)
  if (minuteMatch) return { key: 'card.updatedMinutesAgo', count: Number(minuteMatch[1]) }
  const dayMatch = value.match(/^Updated (\d+) days ago$/)
  if (dayMatch) return { key: 'card.updatedDaysAgo', count: Number(dayMatch[1]) }
  return null
}

export function SourceCard({ source, profileCount, refreshing, onRefresh, onEdit, onRemove }: SourceCardProps) {
  const { t } = useTranslation('sources')
  const { t: tc } = useTranslation('common')
  const isKey = source.type === 'key'
  const statusTone = source.status === 'refresh-due' || source.status === 'updating' ? 'warn' : source.status === 'ready' ? 'neutral' : 'good'
  const Icon = isKey ? KeyRound : Rss
  const displayedValue = isKey ? `${source.value.split('://')[0]}://••••••••••••••••` : source.value
  const originLabel = source.originLabel === 'Remote URL' ? t('card.remoteUrl') : t('card.localKey')
  const sourceTypeLabel = isKey ? t('card.singleKey') : t('card.subscriptionUrl')
  const localizedTimestamp = sourceTimestampKey(source.lastRefresh)
  const lastRefresh = localizedTimestamp
    ? typeof localizedTimestamp === 'string' ? t(localizedTimestamp) : t(localizedTimestamp.key, { count: localizedTimestamp.count })
    : source.lastRefresh

  return (
    <Card className={cn('gap-0 rounded-[10px] border border-hairline bg-surface p-5 shadow-none transition-colors duration-150 hover:border-[#3a3942] hover:bg-row-hover', refreshing && 'border-warn/40')}>
      <div className="flex items-start justify-between gap-5">
        <div className="flex min-w-0 items-start gap-3.5">
          <span className={cn('flex size-10 shrink-0 items-center justify-center rounded-[9px] bg-raised text-lavender-hi', isKey && 'text-good')}><Icon aria-hidden="true" className="size-[19px]" strokeWidth={1.7} /></span>
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2.5">
              <h2 className="type-display text-[18px] tracking-[-0.02em] text-primary">{source.name}</h2>
              <Badge className={cn('rounded-full border-0 px-2 py-1 text-[10px] font-medium', statusTone === 'good' && 'bg-good/12 text-good', statusTone === 'warn' && 'bg-warn/12 text-warn', statusTone === 'neutral' && 'bg-lavender/12 text-lavender-hi')} variant="outline">{tc(sourceStatusKey(source.status))}</Badge>
            </div>
            <p className="mt-1 text-[11px] text-muted-copy">{sourceTypeLabel} <span className="px-1.5 text-quiet">·</span> {originLabel}</p>
          </div>
        </div>
        <div className="flex shrink-0 items-center gap-1">
          <Button aria-label={t(refreshing ? 'menu.refreshingAria' : 'menu.refreshAria', { name: source.name })} className="h-9 gap-2 border-hairline px-3 text-[11px] text-body hover:bg-raised hover:text-primary" disabled={refreshing} onClick={onRefresh} type="button" variant="outline"><RefreshCw aria-hidden="true" className={cn('size-[15px]', refreshing && 'animate-spin')} /> <span>{refreshing ? t('menu.refreshing') : t('menu.refresh')}</span></Button>
          <DropdownMenu>
            <DropdownMenuTrigger asChild><Button aria-label={t('menu.more', { name: source.name })} className="size-9 border-hairline text-body hover:bg-raised hover:text-primary" size="icon" type="button" variant="outline"><MoreHorizontal aria-hidden="true" className="size-[17px]" strokeWidth={1.7} /></Button></DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-44 border-hairline bg-raised text-[12px]">
              <DropdownMenuItem onSelect={onEdit}><Pencil aria-hidden="true" className="size-3.5 text-muted-copy" />{t('menu.edit')}</DropdownMenuItem>
              <DropdownMenuItem onSelect={onRefresh}><RefreshCw aria-hidden="true" className="size-3.5 text-muted-copy" />{t('menu.refreshNow')}</DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem className="text-bad focus:bg-bad/10 focus:text-bad" onSelect={onRemove}><Trash2 aria-hidden="true" className="size-3.5" />{t('menu.remove')}</DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
      <div className="mt-4 flex items-center gap-2 rounded-lg border border-hairline bg-canvas px-3 py-2.5">
        <Link2 aria-hidden="true" className="size-[15px] shrink-0 text-muted-copy" strokeWidth={1.7} />
        <code className="min-w-0 truncate font-mono text-[11px] text-body">{displayedValue}</code>
      </div>
      <div className="mt-4 grid grid-cols-3 gap-5 border-t border-hairline pt-3.5 max-[640px]:grid-cols-1 max-[640px]:gap-3">
        <div><p className="type-eyebrow !text-[10px]">{t('card.vpnLabel')}</p><p className="mt-1 text-[13px] font-medium text-primary">{tc(profileCount === 1 ? 'counts.vpnOne' : 'counts.vpnMany', { count: profileCount })}</p></div>
        <div><p className="type-eyebrow !text-[10px]">{isKey ? t('card.lastUpdate') : t('card.lastRefresh')}</p><p className="mt-1 text-[12px] text-body">{refreshing ? t('card.refreshingNow') : lastRefresh}</p></div>
        <div><p className="type-eyebrow !text-[10px]">{t('card.origin')}</p><p className="mt-1 text-[12px] text-body">{originLabel}</p></div>
      </div>
      <p className="mt-3 flex items-center gap-1.5 text-[11px] text-muted-copy"><span className="text-lavender">→</span>{isKey ? t('card.thisKeyStarts') : t('card.vpnStayGrouped')} <span className="text-body">{isKey ? t('card.default') : t('card.groups')}</span></p>
    </Card>
  )
}
