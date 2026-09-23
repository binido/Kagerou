import { useTranslation } from 'react-i18next'
import { X } from 'lucide-react'

import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { formatBytes, formatUptime } from '@/lib/formatters'
import { connectionStartMillis } from '@/store/slices/connections'
import type { ConnectionExit, LiveConnection } from '@/types/kagerou'

const headClassName =
  'px-3 py-3 text-[10px] font-semibold uppercase tracking-[0.15em] text-muted-copy'

const bytes = (count: number) => {
  const { value, unit } = formatBytes(count)
  return `${value} ${unit}`
}

interface ConnectionsTableProps {
  connections: LiveConnection[]
  now: number
  onClose: (id: string) => void
}

export function ConnectionsTable({ connections, now, onClose }: ConnectionsTableProps) {
  const { t } = useTranslation('connections')

  const exitLabel = (exit: ConnectionExit) => {
    switch (exit.kind) {
      case 'profile':
        return exit.name
      case 'direct':
        return t('table.direct')
      case 'block':
        return t('table.block')
      case 'other':
        return exit.tag || t('table.unknown')
    }
  }

  return (
    <div className="overflow-x-auto rounded-[10px] bg-surface ring-1 ring-inset ring-hairline/55">
      <Table className="min-w-[560px] table-fixed">
        <TableHeader>
          <TableRow className="border-b border-hairline/55 hover:bg-transparent">
            <TableHead className={`${headClassName} pl-5`}>{t('table.destination')}</TableHead>
            <TableHead className={`${headClassName} w-[88px]`}>{t('table.network')}</TableHead>
            <TableHead className={`${headClassName} w-[120px]`}>{t('table.exit')}</TableHead>
            <TableHead className={`${headClassName} w-[180px] max-[1100px]:hidden`}>
              {t('table.rule')}
            </TableHead>
            <TableHead className={`${headClassName} w-[150px] text-right`}>
              {t('table.traffic')}
            </TableHead>
            <TableHead className={`${headClassName} w-[72px] text-right`}>
              {t('table.duration')}
            </TableHead>
            <TableHead className="w-12 px-3 py-3">
              <span className="sr-only">{t('table.close', { host: '' })}</span>
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {connections.map((connection) => {
            const destination = `${connection.host}:${connection.port}`
            const exit = exitLabel(connection.exit)
            return (
              <TableRow
                className="group h-[42px] border-b border-hairline/55 text-[12px] text-body transition-colors hover:bg-row-hover"
                key={connection.id}
              >
                <TableCell className="truncate px-3 py-2 pl-5 font-mono" title={destination}>
                  {destination}
                </TableCell>
                <TableCell className="px-3 py-2 uppercase text-quiet">
                  {connection.network}
                </TableCell>
                <TableCell
                  className={
                    connection.exit.kind === 'block'
                      ? 'truncate px-3 py-2 text-bad'
                      : 'truncate px-3 py-2'
                  }
                  title={exit}
                >
                  {exit}
                </TableCell>
                <TableCell
                  className="truncate px-3 py-2 text-quiet max-[1100px]:hidden"
                  title={connection.rule}
                >
                  {connection.rule}
                </TableCell>
                <TableCell className="type-data whitespace-nowrap px-3 py-2 text-right">
                  {bytes(connection.download)} / {bytes(connection.upload)}
                </TableCell>
                <TableCell className="type-data px-3 py-2 text-right">
                  {formatUptime(now - connectionStartMillis(connection.start))}
                </TableCell>
                <TableCell className="px-3 py-2 text-right">
                  <Button
                    aria-label={t('table.close', { host: destination })}
                    className="size-7 text-quiet opacity-0 hover:bg-row-hover hover:text-bad focus-visible:opacity-100 group-hover:opacity-100"
                    onClick={() => onClose(connection.id)}
                    size="icon-xs"
                    type="button"
                    variant="ghost"
                  >
                    <X aria-hidden="true" className="size-3.5" strokeWidth={1.7} />
                  </Button>
                </TableCell>
              </TableRow>
            )
          })}
        </TableBody>
      </Table>
    </div>
  )
}
