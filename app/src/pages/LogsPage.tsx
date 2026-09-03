import { useMemo, useState } from 'react'

import { LogToolbar } from '@/components/logs/LogToolbar'
import { LogViewer } from '@/components/logs/LogViewer'
import { PageHeader } from '@/components/layout/PageHeader'
import { formatLogCount } from '@/lib/formatters'
import { useKagerouStore } from '@/store/kagerou-store'

export function LogsPage() {
  const logs = useKagerouStore((state) => state.logs)
  const [query, setQuery] = useState('')
  const normalizedQuery = query.trim().toLowerCase()
  const entries = useMemo(
    () => logs.filter((entry) => `${entry.timestamp} ${entry.level} ${entry.message}`.toLowerCase().includes(normalizedQuery)),
    [logs, normalizedQuery],
  )

  return (
    <div className="flex h-screen min-h-[680px] min-w-0 flex-col bg-canvas px-6 pb-5 pt-7 lg:px-8">
      <div className="flex min-h-0 min-w-0 flex-1 flex-col">
        <PageHeader
          description="Connection events and service output"
          status={<span className="flex items-center gap-2 text-[12px] text-muted-copy"><span aria-hidden="true" className="size-1.5 rounded-full bg-good" />Connected</span>}
          title="Logs"
        />
        <div className="mt-6 shrink-0"><LogToolbar countLabel={formatLogCount(entries.length, query)} onClear={() => setQuery('')} onQueryChange={setQuery} query={query} /></div>
        <LogViewer entries={entries} query={query} />
      </div>
    </div>
  )
}
