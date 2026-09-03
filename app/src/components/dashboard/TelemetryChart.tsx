import { useTranslation } from 'react-i18next'
import { CartesianGrid, Line, LineChart, XAxis, YAxis } from 'recharts'

import { ChartContainer, ChartTooltip, ChartTooltipContent, type ChartConfig } from '@/components/ui/chart'
import type { TelemetryPoint } from '@/types/kagerou'

export function TelemetryChart({ data }: { data: TelemetryPoint[] }) {
  const { t } = useTranslation('dashboard')
  const chartConfig = {
    download: { label: t('telemetry.download'), color: 'var(--lavender)' },
    upload: { label: t('telemetry.upload'), color: 'var(--upload-line)' },
  } satisfies ChartConfig

  return (
    <ChartContainer config={chartConfig} className="h-full w-full min-w-0">
      <LineChart accessibilityLayer data={data} margin={{ top: 8, right: 4, left: -28, bottom: 0 }}>
        <CartesianGrid stroke="var(--hairline)" strokeDasharray="2 6" vertical={false} />
        <XAxis axisLine={{ stroke: 'var(--hairline)' }} dataKey="label" tickLine={{ stroke: 'var(--text-muted)' }} tick={{ fill: 'var(--text-muted)', fontFamily: 'IBM Plex Mono, monospace', fontSize: 9 }} tickMargin={10} />
        <YAxis domain={[0, 100]} hide />
        <ChartTooltip content={<ChartTooltipContent />} cursor={{ stroke: 'var(--hairline)' }} />
        <Line activeDot={{ r: 3, fill: 'var(--lavender)', stroke: 'var(--canvas)' }} dataKey="download" dot={false} isAnimationActive={false} stroke="var(--lavender)" strokeLinecap="round" strokeWidth={2} />
        <Line activeDot={{ r: 3, fill: 'var(--upload-line)', stroke: 'var(--canvas)' }} dataKey="upload" dot={false} isAnimationActive={false} stroke="var(--upload-line)" strokeLinecap="round" strokeWidth={2} />
      </LineChart>
    </ChartContainer>
  )
}
