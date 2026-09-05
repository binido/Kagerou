import { Search, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'

interface LogToolbarProps {
  query: string
  count: number
  onQueryChange: (query: string) => void
  onClear: () => void
}

export function LogToolbar({ query, count, onQueryChange, onClear }: LogToolbarProps) {
  const { t } = useTranslation('logs')
  const countLabel = query.trim()
    ? t(count === 1 ? 'toolbar.matchingOne' : 'toolbar.matchingMany', { count })
    : t(count === 1 ? 'toolbar.entriesOne' : 'toolbar.entriesMany', { count })

  return (
    <div className="flex min-w-0 items-center gap-3">
      <label className="log-toolbar-control log-toolbar-search flex min-w-0 flex-1 items-center gap-3 rounded-lg bg-surface ring-1 ring-inset ring-hairline transition-colors focus-within:ring-2 focus-within:ring-lavender/70">
        <Search aria-hidden="true" className="size-[17px] shrink-0 text-muted-copy" strokeWidth={1.7} />
        <span className="shrink-0 text-[13px] font-medium text-body">{t('toolbar.search')}</span>
        <span aria-hidden="true" className="h-4 w-px shrink-0 bg-white/10" />
        <Input
          aria-controls="log-list"
          aria-label={t('toolbar.search')}
          autoComplete="off"
          className="h-auto min-w-0 flex-1 border-0 bg-transparent px-0 py-0 text-[13px] text-primary shadow-none ring-0 placeholder:text-quiet focus-visible:ring-0"
          onChange={(event) => onQueryChange(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Escape') onClear()
          }}
          placeholder={t('toolbar.placeholder')}
          type="search"
          value={query}
        />
      </label>
      <Button className="log-toolbar-control shrink-0 gap-2 bg-surface text-[12px] text-body ring-1 ring-inset ring-hairline hover:bg-row-hover hover:text-primary" onClick={onClear} type="button" variant="ghost">
        <X aria-hidden="true" className="size-4" strokeWidth={1.7} />
        <span>{t('toolbar.clear')}</span>
      </Button>
      <span aria-live="polite" className={cn('ml-auto shrink-0 text-[12px] text-muted-copy', query && 'text-lavender-hi')} role="status">
        {countLabel}
      </span>
    </div>
  )
}
