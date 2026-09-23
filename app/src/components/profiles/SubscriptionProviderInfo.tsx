import { useState } from 'react'
import { ExternalLink, Megaphone } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { formatBytes } from '@/lib/formatters'
import { daysUntil, expiryTone, usageTone, type ProviderTone } from '@/lib/provider-info'
import { cn } from '@/lib/utils'
import type { ProviderInfo } from '@/types/kagerou'

const toneClass: Record<ProviderTone, string> = {
  normal: 'text-muted-copy',
  warn: 'text-warn',
  bad: 'text-bad',
}

const bytes = (count: number) => {
  const { value, unit } = formatBytes(count)
  return `${value} ${unit}`
}

interface SubscriptionProviderInfoProps {
  info: ProviderInfo
  groupLabel: string
  onOpenSupport: () => void
  className?: string
}

export function SubscriptionProviderInfo({
  info,
  groupLabel,
  onOpenSupport,
  className,
}: SubscriptionProviderInfoProps) {
  const { i18n, t } = useTranslation('profiles')
  const language = i18n.resolvedLanguage ?? 'en'
  // Days are the finest unit shown, so the time the card mounted is close enough.
  const [now] = useState(() => Date.now())

  const usage =
    info.trafficUsed === null ? null : info.trafficTotal === null ? (
      <span>{t('provider.usedUnlimited', { used: bytes(info.trafficUsed) })}</span>
    ) : (
      <span className={toneClass[usageTone(info.trafficUsed, info.trafficTotal)]}>
        {t('provider.used', { used: bytes(info.trafficUsed), total: bytes(info.trafficTotal) })}
      </span>
    )

  const expiry =
    info.expiresAt === null ? null : (
      <span className={toneClass[expiryTone(info.expiresAt, now)]}>
        {t(info.expiresAt <= now ? 'provider.expired' : 'provider.expires', {
          date: new Intl.DateTimeFormat(language, { dateStyle: 'medium' }).format(info.expiresAt),
          relative: new Intl.RelativeTimeFormat(language, { numeric: 'auto' }).format(
            daysUntil(info.expiresAt, now),
            'day',
          ),
        })}
      </span>
    )

  const parts = [usage, expiry].filter((part) => part !== null)

  return (
    <div className={cn('space-y-1.5 px-5 py-3 text-[11px] text-muted-copy', className)}>
      {parts.length > 0 || info.supportUrl ? (
        <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
          {parts.map((part, index) => (
            <span className="flex items-center gap-2" key={index}>
              {index > 0 ? <span aria-hidden="true">·</span> : null}
              {part}
            </span>
          ))}
          {info.supportUrl ? (
            <>
              {parts.length > 0 ? <span aria-hidden="true">·</span> : null}
              <button
                aria-label={t('provider.supportAria', { name: groupLabel })}
                className="inline-flex items-center gap-1 rounded-sm text-lavender-hi underline-offset-2 outline-none hover:underline focus-visible:ring-2 focus-visible:ring-lavender"
                onClick={onOpenSupport}
                type="button"
              >
                {t('provider.support')}
                <ExternalLink aria-hidden="true" className="size-3" />
              </button>
            </>
          ) : null}
        </div>
      ) : null}
      {info.announce ? (
        <p className="flex items-start gap-1.5 whitespace-pre-line text-body" data-selectable>
          <Megaphone aria-hidden="true" className="mt-px size-3.5 shrink-0 text-lavender" />
          <span>
            <span className="sr-only">{t('provider.announce')}: </span>
            {info.announce}
          </span>
        </p>
      ) : null}
    </div>
  )
}
