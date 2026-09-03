import { Power } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

interface ConnectionDialProps {
  connected: boolean
  onToggle: () => void
}

export function ConnectionDial({ connected, onToggle }: ConnectionDialProps) {
  const { t } = useTranslation('dashboard')
  const stateLabel = connected ? t('connection.connected') : t('connection.disconnected')

  return (
    <Button
      aria-label={t('connection.control', { state: stateLabel })}
      aria-pressed={connected}
      className={cn(
        'group/dial size-[190px] rounded-full border-[3px] bg-canvas p-2 shadow-[0_20px_64px_rgba(9,8,13,.2)] hover:bg-selected active:translate-y-px',
        connected ? 'border-lavender' : 'border-hairline',
      )}
      onClick={onToggle}
      size="icon-lg"
      type="button"
      variant="ghost"
    >
      <span className={cn('flex size-full flex-col items-center justify-center rounded-full border border-hairline', connected ? 'bg-raised' : 'bg-surface')}>
        <Power aria-hidden="true" className={cn('size-6', connected ? 'text-lavender-hi' : 'text-muted-copy')} strokeWidth={1.7} />
        <span className="type-display mt-3 text-[18px] tracking-[-0.02em] text-primary">
          {stateLabel}
        </span>
      </span>
    </Button>
  )
}
