import { useTranslation } from 'react-i18next'
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty'
import { ScrollArea } from '@/components/ui/scroll-area'
import { SearchX } from 'lucide-react'

import { LogRow } from '@/components/logs/LogRow'
import type { LogEntry } from '@/types/kagerou'

export function LogViewer({ entries, query }: { entries: LogEntry[]; query: string }) {
  const { t } = useTranslation('logs')

  return (
    <section aria-label={t('viewer.ariaLabel')} className="mt-4 flex min-h-0 flex-1 flex-col">
      <ScrollArea aria-label={t('viewer.scrollableAriaLabel')} className="min-h-0 flex-1 overflow-hidden rounded-[10px] bg-viewport ring-1 ring-inset ring-hairline" id="log-viewport" tabIndex={0}>
        <div className="min-w-0 font-mono">
          {entries.length ? (
            <div className="space-y-0 px-4 py-4" id="log-list">
              {entries.map((entry) => <LogRow entry={entry} key={entry.id} query={query} />)}
            </div>
          ) : (
            <Empty className="min-h-[240px] border-0 p-12 text-body">
              <EmptyHeader>
                <EmptyMedia variant="default"><SearchX aria-hidden="true" className="size-6 text-quiet" strokeWidth={1.5} /></EmptyMedia>
                <EmptyTitle className="text-[13px] font-normal text-body">{t('viewer.noMatch')}</EmptyTitle>
                <EmptyDescription className="text-[12px] text-muted-copy">{t('viewer.tryDifferent')}</EmptyDescription>
              </EmptyHeader>
            </Empty>
          )}
        </div>
      </ScrollArea>
      <footer className="mt-3 flex h-8 shrink-0 items-center border-t border-hairline pt-2 text-[12px] text-muted-copy">
        <span className="flex items-center gap-2"><span aria-hidden="true" className="size-1.5 rounded-full bg-good" />{t('footer.live')}</span>
        <span className="ml-auto max-[760px]:hidden">{t('footer.scrollHint')}</span>
      </footer>
    </section>
  )
}
