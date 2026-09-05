import { Loader2, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import type { TestRun } from '@/types/kagerou'

interface TestRunBarProps {
  run: TestRun
  onCancel: () => void
}

export function TestRunBar({ run, onCancel }: TestRunBarProps) {
  const { t } = useTranslation('profiles')
  // Before the first result lands the total is unknown; show the bar as
  // indeterminate rather than as a confident zero percent.
  const known = run.total > 0
  const percent = known ? Math.round((run.done / run.total) * 100) : 0

  return (
    <div className="mt-4 flex items-center gap-3 rounded-lg border border-hairline bg-surface px-3.5 py-2.5">
      <Loader2 aria-hidden="true" className="size-4 shrink-0 animate-spin text-lavender" strokeWidth={1.8} />
      <div className="min-w-0 flex-1">
        <div className="flex items-baseline justify-between gap-3">
          <span className="truncate text-[12px] text-body">{t('test.runningLabel')}</span>
          <span className="type-data shrink-0 text-[11px] text-muted-copy">
            {known ? t('test.progressCount', { done: run.done, total: run.total }) : t('test.starting')}
          </span>
        </div>
        <div
          aria-label={t('test.runningLabel')}
          aria-valuemax={100}
          aria-valuemin={0}
          aria-valuenow={known ? percent : undefined}
          className="mt-2 h-1 w-full overflow-hidden rounded-full bg-raised"
          role="progressbar"
        >
          <div
            className={known ? 'h-full rounded-full bg-lavender transition-[width] duration-200' : 'h-full w-1/3 animate-pulse rounded-full bg-lavender'}
            style={known ? { width: `${percent}%` } : undefined}
          />
        </div>
      </div>
      <Button aria-label={t('test.stop')} className="h-8 shrink-0 gap-1.5 border-hairline px-2.5 text-[11px] text-body hover:bg-raised hover:text-primary" onClick={onCancel} type="button" variant="outline">
        <X aria-hidden="true" className="size-3.5" />
        <span>{t('test.stop')}</span>
      </Button>
    </div>
  )
}
