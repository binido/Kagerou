import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { LogToolbar } from '@/components/logs/LogToolbar'
import { LogViewer } from '@/components/logs/LogViewer'
import { PageHeader } from '@/components/layout/PageHeader'
import { useKagerouStore } from '@/store/kagerou-store'

export function LogsPage() {
  const { t } = useTranslation('logs')
  const logs = useKagerouStore((state) => state.logs)
  const [query, setQuery] = useState('')
  const normalizedQuery = query.trim().toLowerCase()
  const entries = useMemo(
    () => logs.filter((entry) => `${entry.timestamp} ${entry.level} ${entry.message}`.toLowerCase().includes(normalizedQuery)),
    [logs, normalizedQuery],
  )

  return (
    <div className="flex h-screen min-h-[680px] min-w-0 flex-col bg-canvas px-6 pb-5 pt-7 lg:px-8">
      <div className="mx-auto flex min-h-0 min-w-0 w-full max-w-[1280px] flex-1 flex-col">
        <PageHeader
          description={t('page.description')}
          status={<span className="flex items-center gap-2 text-[12px] text-muted-copy"><span aria-hidden="true" className="size-1.5 rounded-full bg-good" />{t('page.connected')}</span>}
          title={t('page.title')}
        />
        <div className="mt-6 shrink-0"><LogToolbar count={entries.length} onClear={() => setQuery('')} onQueryChange={setQuery} query={query} /></div>
        <LogViewer entries={entries} query={query} />
      </div>
    </div>
  )
}
