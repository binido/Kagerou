import { Activity, ArrowDown, ArrowUp, type LucideIcon } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { ResultBadge } from '@/components/common/ResultBadge'
import { cn } from '@/lib/utils'
import { formatBytes, formatSpeedMbps } from '@/lib/formatters'
import type { SessionTraffic, TestResult } from '@/types/kagerou'

interface TrafficReadoutProps {
  icon: LucideIcon
  iconClassName: string
  label: string
  value: string
  unit: string
}

function TrafficReadout({ icon: Icon, iconClassName, label, value, unit }: TrafficReadoutProps) {
  return (
    <div aria-label={label} className="flex shrink-0 items-center gap-2 px-3 first:pl-0 max-[980px]:gap-1.5 max-[980px]:px-2 max-[760px]:px-1.5" role="group" title={label}>
      <Icon aria-hidden="true" className={cn('size-4', iconClassName)} strokeWidth={1.7} />
      <span className="type-data whitespace-nowrap text-body">
        {value} <span className="font-sans text-[10px] font-medium tracking-normal text-body">{unit}</span>
      </span>
    </div>
  )
}

interface ConnectionTrafficReadoutsProps {
  latestDownload: number
  latestUpload: number
  ping: TestResult
  sessionTraffic: SessionTraffic
}

export function ConnectionTrafficReadouts({ latestDownload, latestUpload, ping, sessionTraffic }: ConnectionTrafficReadoutsProps) {
  const { t } = useTranslation('dashboard')
  const sessionTotal = formatBytes(sessionTraffic.download + sessionTraffic.upload)

  return (
    <div aria-label={t('connection.traffic.ariaLabel')} className="flex min-w-0 shrink-0 items-center justify-end whitespace-nowrap text-[11px]" role="group">
      <TrafficReadout
        icon={ArrowDown}
        iconClassName="text-lavender"
        label={t('connection.traffic.download')}
        unit={t('connection.traffic.unit')}
        value={formatSpeedMbps(latestDownload)}
      />
      <span aria-hidden="true" className="h-5 w-px shrink-0 bg-hairline/80" />
      <TrafficReadout
        icon={ArrowUp}
        iconClassName="text-upload-line"
        label={t('connection.traffic.upload')}
        unit={t('connection.traffic.unit')}
        value={formatSpeedMbps(latestUpload)}
      />
      <span aria-hidden="true" className="h-5 w-px shrink-0 bg-hairline/80" />
      <TrafficReadout
        icon={Activity}
        iconClassName="text-body"
        label={t('connection.traffic.session')}
        unit={sessionTotal.unit}
        value={sessionTotal.value}
      />
      <span aria-hidden="true" className="h-5 w-px shrink-0 bg-hairline/80" />
      <div aria-label={`${t('connection.ping')}: ${ping.value}`} className="flex shrink-0 items-center gap-2 px-3 pr-0 max-[980px]:gap-1.5 max-[980px]:px-2 max-[760px]:px-1.5 max-[760px]:pr-0" role="status">
        <span className="type-eyebrow !text-[9px] !tracking-[0.14em]">{t('connection.ping')}</span>
        <ResultBadge tone={ping.tone} value={ping.value} />
      </div>
    </div>
  )
}
