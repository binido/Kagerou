import { useTranslation } from 'react-i18next'

import { cn } from '@/lib/utils'
import { localizeResultValue } from '@/lib/result-copy'
import type { TestTone } from '@/types/kagerou'

const toneClasses: Record<TestTone, string> = {
  good: 'text-body before:bg-good',
  warn: 'text-warn before:bg-warn',
  bad: 'text-bad before:bg-bad',
  muted: 'text-muted-copy before:bg-quiet',
}

export function ResultBadge({ value, tone }: { value: string; tone: TestTone }) {
  const { t } = useTranslation('common')

  return (
    <span className={cn('inline-flex min-w-0 items-center gap-2 whitespace-nowrap font-mono text-[11px] tabular-nums before:size-1.5 before:shrink-0 before:rounded-full', toneClasses[tone])}>
      {localizeResultValue(value, t)}
    </span>
  )
}
