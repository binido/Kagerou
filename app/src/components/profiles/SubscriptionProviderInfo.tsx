import { useState, type ReactNode } from 'react'
import { CalendarClock, ExternalLink, Gauge, LifeBuoy, Megaphone } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { formatBytes } from '@/lib/formatters'
import { daysUntil, expiryTone, usageTone, type ProviderTone } from '@/lib/provider-info'
import { cn } from '@/lib/utils'
import type { ProviderInfo } from '@/types/kagerou'

const textTone: Record<ProviderTone, string> = {
  normal: 'text-body',
  warn: 'text-warn',
  bad: 'text-bad',
}

const barTone: Record<ProviderTone, string> = {
  normal: 'bg-lavender',
  warn: 'bg-warn',
  bad: 'bg-bad',
}

const bytes = (count: number) => {
  const { value, unit } = formatBytes(count)
  return `${value} ${unit}`
}

function Fact({
  icon: Icon,
  label,
  children,
}: {
  icon: typeof Gauge
  label: string
  children: ReactNode
}) {
  return (
    <span className="flex items-center gap-2">
      <Icon aria-hidden="true" className="size-3.5 shrink-0 text-muted-copy" strokeWidth={1.8} />
      <span className="sr-only">{label}: </span>
      {children}
    </span>
  )
}

interface SubscriptionProviderInfoProps {
  info: ProviderInfo
  groupLabel: string
  onOpenSupport: () => void
}

/** Sits under the group's title, indented to line up with it. */
export function SubscriptionProviderInfo({
  info,
  groupLabel,
  onOpenSupport,
}: SubscriptionProviderInfoProps) {
  const { i18n, t } = useTranslation('profiles')
  const language = i18n.resolvedLanguage ?? 'en'
  // Days are the finest unit shown, so the time the card mounted is close enough.
  const [now] = useState(() => Date.now())

  const { trafficUsed: used, trafficTotal: total, expiresAt } = info
  const hasFacts = used !== null || expiresAt !== null || info.supportUrl !== null

  return (
    <div className="space-y-3 pb-4 pl-[60px] pr-5 max-[640px]:pl-5">
      {hasFacts ? (
        <div className="flex flex-wrap items-center gap-x-6 gap-y-2 text-[12px] text-body">
          {used !== null ? (
            <Fact icon={Gauge} label={t('provider.traffic')}>
              {total !== null ? (
                <>
                  <span
                    aria-hidden="true"
                    className="h-1.5 w-24 overflow-hidden rounded-full bg-hairline"
                  >
                    <span
                      className={cn('block h-full rounded-full', barTone[usageTone(used, total)])}
                      style={{ width: `${Math.min(100, (used / total) * 100)}%` }}
                    />
                  </span>
                  <span className={cn('type-data', textTone[usageTone(used, total)])}>
                    {t('provider.used', { used: bytes(used), total: bytes(total) })}
                  </span>
                </>
              ) : (
                <span className="type-data">
                  {t('provider.usedUnlimited', { used: bytes(used) })}
                </span>
              )}
            </Fact>
          ) : null}
          {expiresAt !== null ? (
            <Fact icon={CalendarClock} label={t('provider.expiry')}>
              <span className={textTone[expiryTone(expiresAt, now)]}>
                {t(expiresAt <= now ? 'provider.expired' : 'provider.expires', {
                  date: new Intl.DateTimeFormat(language, { dateStyle: 'medium' }).format(
                    expiresAt,
                  ),
                  relative: new Intl.RelativeTimeFormat(language, { numeric: 'auto' }).format(
                    daysUntil(expiresAt, now),
                    'day',
                  ),
                })}
              </span>
            </Fact>
          ) : null}
          {info.supportUrl ? (
            <button
              aria-label={t('provider.supportAria', { name: groupLabel })}
              className="flex items-center gap-2 rounded-sm text-lavender-hi outline-none hover:underline focus-visible:ring-2 focus-visible:ring-lavender"
              onClick={onOpenSupport}
              type="button"
            >
              <LifeBuoy aria-hidden="true" className="size-3.5" strokeWidth={1.8} />
              {t('provider.support')}
              <ExternalLink aria-hidden="true" className="size-3" />
            </button>
          ) : null}
        </div>
      ) : null}
      {info.announce ? (
        <div
          className="flex max-w-[720px] items-start gap-2.5 rounded-lg bg-raised px-3 py-2.5 text-[12px] leading-5 text-body ring-1 ring-inset ring-hairline"
          data-selectable
        >
          <Megaphone aria-hidden="true" className="mt-0.5 size-3.5 shrink-0 text-lavender" />
          <p className="whitespace-pre-line">
            <span className="sr-only">{t('provider.announce')}: </span>
            {info.announce}
          </p>
        </div>
      ) : null}
    </div>
  )
}
