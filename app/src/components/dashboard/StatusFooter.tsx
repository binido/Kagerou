import { useTranslation } from 'react-i18next'

import { cn } from '@/lib/utils'
import type { RoutingPreset, RoutingRule, Source } from '@/types/kagerou'

interface StatusFooterProps {
  presets: RoutingPreset[]
  rules: RoutingRule[]
  sources: Source[]
  className?: string
}

/** Reference information, not a reason to open the app: one line, no card.
 * Deliberately counts rather than refresh timestamps — the backend stores
 * those as English prose, and re-displaying them here would spread a known
 * localisation hole to a second screen. */
export function StatusFooter({ presets, rules, sources, className }: StatusFooterProps) {
  const { t } = useTranslation('dashboard')
  const enabled = presets.filter((preset) => preset.enabled)
  const stale = sources.filter((source) => source.status === 'refresh-due').length

  const items = [
    enabled.length > 0 ? t('footer.presets', { names: enabled.map((preset) => preset.label).join(', ') }) : t('footer.noPresets'),
    t('footer.rules', { count: rules.length }),
    stale > 0 ? t('footer.sourcesStale', { count: stale, total: sources.length }) : t('footer.sources', { count: sources.length }),
  ]

  return (
    <p className={cn('type-meta flex flex-wrap items-center gap-x-2 gap-y-1 px-1', className)}>
      {items.map((item, index) => (
        <span key={item} className="flex items-center gap-2">
          {index > 0 ? <span aria-hidden="true" className="text-hairline">·</span> : null}
          {item}
        </span>
      ))}
    </p>
  )
}
