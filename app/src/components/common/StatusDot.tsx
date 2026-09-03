import { cn } from '@/lib/utils'

type StatusTone = 'good' | 'warn' | 'bad' | 'muted'

const toneClasses: Record<StatusTone, string> = {
  good: 'bg-good',
  warn: 'bg-warn',
  bad: 'bg-bad',
  muted: 'bg-quiet',
}

export function StatusDot({ tone = 'good', className }: { tone?: StatusTone; className?: string }) {
  return <span aria-hidden="true" className={cn('size-1.5 shrink-0 rounded-full', toneClasses[tone], className)} />
}
