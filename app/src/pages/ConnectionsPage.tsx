import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { XCircle } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '@/components/ui/empty'
import { ConnectionsTable } from '@/components/connections/ConnectionsTable'
import { PageContainer } from '@/components/layout/PageContainer'
import { PageHeader } from '@/components/layout/PageHeader'
import { useKagerouStore } from '@/store/kagerou-store'

const POLL_INTERVAL_MS = 1000

export function ConnectionsPage() {
  const { t } = useTranslation('connections')
  const connected = useKagerouStore((state) => state.connected)
  const connections = useKagerouStore((state) => state.liveConnections)
  const refreshConnections = useKagerouStore((state) => state.refreshConnections)
  const closeConnection = useKagerouStore((state) => state.closeConnection)
  const closeAllConnections = useKagerouStore((state) => state.closeAllConnections)
  const [now, setNow] = useState(() => Date.now())

  useEffect(() => {
    if (!connected) return
    const tick = () => {
      setNow(Date.now())
      void refreshConnections()
    }
    tick()
    const timer = window.setInterval(tick, POLL_INTERVAL_MS)
    return () => window.clearInterval(timer)
  }, [connected, refreshConnections])

  const empty = !connected || connections.length === 0

  return (
    <PageContainer>
      <PageHeader
        actions={
          <Button
            className="h-8 gap-1.5 px-3 text-[12px]"
            disabled={empty}
            onClick={() => void closeAllConnections()}
            type="button"
            variant="outline"
          >
            <XCircle aria-hidden="true" className="size-3.5" strokeWidth={1.8} />
            {t('page.closeAll')}
          </Button>
        }
        description={t('page.description')}
        eyebrow={t('page.eyebrow')}
        status={
          connected ? (
            <span className="text-[11px] text-quiet" role="status">
              {t('page.count', { count: connections.length })}
            </span>
          ) : null
        }
        title={t('page.title')}
      />
      <section className="mt-9">
        {empty ? (
          <Empty className="rounded-[10px] bg-surface py-10 ring-1 ring-inset ring-hairline/55">
            <EmptyHeader>
              <EmptyTitle className="text-[14px] text-primary">
                {connected ? t('page.emptyTitle') : t('page.disconnectedTitle')}
              </EmptyTitle>
              <EmptyDescription className="text-[12px] text-quiet">
                {connected ? t('page.emptyDescription') : t('page.disconnectedDescription')}
              </EmptyDescription>
            </EmptyHeader>
          </Empty>
        ) : (
          <ConnectionsTable
            connections={connections}
            now={now}
            onClose={(id) => void closeConnection(id)}
          />
        )}
      </section>
    </PageContainer>
  )
}
