import { Activity } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Card } from '@/components/ui/card'
import { TelemetryChart } from '@/components/dashboard/TelemetryChart'
import { formatBytes, formatSpeedMbps } from '@/lib/formatters'
import type { SessionTraffic, TelemetryPoint } from '@/types/kagerou'

export function TelemetryPanel({ data, sessionTraffic }: { data: TelemetryPoint[]; sessionTraffic: SessionTraffic }) {
  const { t } = useTranslation('dashboard')
  const latest = data.at(-1)
  const sessionTotal = formatBytes(sessionTraffic.download + sessionTraffic.upload)

  return (
    <Card className="flex min-h-[584px] flex-col rounded-[10px] border border-hairline bg-surface p-0 shadow-none">
      <div className="border-b border-hairline px-6 py-6">
        <div className="flex items-center justify-between">
          <h2 className="type-eyebrow">{t('telemetry.title')}</h2>
          <span className="type-data text-muted-copy">{t('telemetry.window')}</span>
        </div>
        <div className="mt-7 grid grid-cols-2 gap-4">
          <div>
            <p className="text-[12px] text-muted-copy">{t('telemetry.download')}</p>
            <p className="type-display mt-1 whitespace-nowrap text-[29px] leading-none tracking-[-0.02em] text-primary">
              {formatSpeedMbps(latest?.download ?? 0)} <span className="font-sans text-[12px] font-medium tracking-normal text-body">{t('telemetry.unit')}</span>
            </p>
          </div>
          <div>
            <p className="text-[12px] text-muted-copy">{t('telemetry.upload')}</p>
            <p className="type-display mt-1 whitespace-nowrap text-[29px] leading-none tracking-[-0.02em] text-primary">
              {formatSpeedMbps(latest?.upload ?? 0)} <span className="font-sans text-[12px] font-medium tracking-normal text-body">{t('telemetry.unit')}</span>
            </p>
          </div>
        </div>
      </div>
      <div className="min-h-0 flex-1 px-6 py-6">
        <div className="flex items-center justify-between">
          <p className="text-[12px] text-body">{t('telemetry.lastSeconds')}</p>
          <span className="type-data text-muted-copy">{t('telemetry.unit')}</span>
        </div>
        <div className="mt-5 h-[202px] min-w-0">
          <TelemetryChart data={data} />
        </div>
      </div>
      <div className="flex items-end justify-between border-t border-hairline px-6 py-6">
        <div>
          <p className="text-[12px] text-muted-copy">{t('telemetry.sessionTraffic')}</p>
          <p className="type-display mt-2 text-[27px] leading-none tracking-[-0.02em] text-primary">
            {sessionTotal.value} <span className="font-sans text-[12px] font-medium tracking-normal text-body">{sessionTotal.unit}</span>
          </p>
        </div>
        <Activity aria-hidden="true" className="size-5 text-lavender" strokeWidth={1.7} />
      </div>
    </Card>
  )
}
