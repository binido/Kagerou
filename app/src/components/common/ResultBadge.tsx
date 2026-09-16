import { useTranslation } from 'react-i18next'

import { cn } from '@/lib/utils'
import { resultLabel } from '@/lib/result-copy'
import type { TestResult, TestTone } from '@/types/kagerou'

const toneClasses: Record<TestTone, string> = {
  good: 'text-body before:bg-good',
  warn: 'text-warn before:bg-warn',
  bad: 'text-bad before:bg-bad',
  muted: 'text-muted-copy before:bg-quiet',
}

/** `running` is a state of this screen rather than a measurement, so it is a
 * flag here instead of a sixth outcome the backend would never send. */
export function ResultBadge({ result, running = false }: { result: TestResult; running?: boolean }) {
  const { t } = useTranslation('common')

  return (
    <span className={cn('inline-flex min-w-0 items-center gap-2 whitespace-nowrap font-mono text-[11px] tabular-nums before:size-1.5 before:shrink-0 before:rounded-full', toneClasses[running ? 'warn' : result.tone])}>
      {running ? t('status.running') : resultLabel(result, t)}
    </span>
  )
}
