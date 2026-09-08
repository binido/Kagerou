import { useTranslation } from 'react-i18next'

import { cn } from '@/lib/utils'
import type { TrafficSample } from '@/types/kagerou'

const VIEW_WIDTH = 600
const VIEW_HEIGHT = 100

/** Maps samples onto the viewBox, newest on the right. The scale is the
 * window's own maximum: the previous chart pinned its axis to [0, 100] while
 * the points carried bytes per second, so anything real clipped instantly. */
const toPath = (values: number[], peak: number) => {
  if (values.length < 2) return ''
  const step = VIEW_WIDTH / (values.length - 1)
  return values
    .map((value, index) => {
      const x = index * step
      const y = VIEW_HEIGHT - (value / peak) * VIEW_HEIGHT
      return `${index === 0 ? 'M' : 'L'}${x.toFixed(1)} ${y.toFixed(1)}`
    })
    .join(' ')
}

interface SpeedSparklineProps {
  history: TrafficSample[]
  className?: string
}

export function SpeedSparkline({ history, className }: SpeedSparklineProps) {
  const { t } = useTranslation('dashboard')
  // A flat line at zero would read as "no traffic" when the truth is "not
  // connected", so an empty window says so in words and keeps its height.
  const hasShape = history.length >= 2
  const peak = Math.max(1, ...history.map((point) => Math.max(point.download, point.upload)))

  return (
    <div className={cn('relative min-h-[56px] overflow-hidden rounded-md border border-hairline bg-canvas', className)}>
      {hasShape ? (
        <svg
          aria-label={t('sparkline.ariaLabel')}
          className="size-full"
          preserveAspectRatio="none"
          role="img"
          viewBox={`0 0 ${VIEW_WIDTH} ${VIEW_HEIGHT}`}
        >
          <path
            className="text-lavender"
            d={toPath(history.map((point) => point.download), peak)}
            fill="none"
            stroke="currentColor"
            strokeLinejoin="round"
            strokeWidth={2}
            vectorEffect="non-scaling-stroke"
          />
          <path
            className="text-upload-line"
            d={toPath(history.map((point) => point.upload), peak)}
            fill="none"
            stroke="currentColor"
            strokeLinejoin="round"
            strokeWidth={2}
            vectorEffect="non-scaling-stroke"
          />
        </svg>
      ) : (
        <p className="type-meta flex size-full items-center justify-center">{t('sparkline.empty')}</p>
      )}
    </div>
  )
}
