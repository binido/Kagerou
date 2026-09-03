import { Check } from 'lucide-react'
import { forwardRef } from 'react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { SemanticSwatchStrip } from '@/components/settings/SemanticSwatchStrip'
import { cn } from '@/lib/utils'
import type { Theme } from '@/themes/types'

interface ThemeFlavorRowProps {
  theme: Theme
  active: boolean
  isFirst: boolean
  isLast: boolean
  onSelect: (themeId: Theme['id']) => void
  onKeyDown: (event: React.KeyboardEvent<HTMLButtonElement>) => void
}

export const ThemeFlavorRow = forwardRef<HTMLButtonElement, ThemeFlavorRowProps>(function ThemeFlavorRow(
  { theme, active, isFirst, isLast, onSelect, onKeyDown },
  ref,
) {
  const { t } = useTranslation('settings')
  const modeLabel = theme.isDark ? t('theme.dark') : t('theme.light')

  return (
    <Button
      ref={ref}
      aria-checked={active}
      className={cn(
        'relative h-11 w-full justify-start gap-3 rounded-md border-0 bg-transparent px-3 text-left text-body shadow-none hover:bg-row-hover hover:text-primary focus-visible:focus-ring',
        active && 'bg-selected text-primary hover:bg-selected',
      )}
      onClick={() => onSelect(theme.id)}
      onKeyDown={onKeyDown}
      role="radio"
      type="button"
      variant="ghost"
    >
      <span aria-hidden="true" className="relative flex h-11 w-3 shrink-0 items-center justify-center">
        <span className={cn('absolute left-1/2 top-0 h-1/2 -translate-x-1/2 border-l border-hairline', isFirst && 'top-1/2 h-1/2')} />
        <span className={cn('absolute bottom-0 left-1/2 h-1/2 -translate-x-1/2 border-l border-hairline', isLast && 'h-1/2')} />
        <span className="absolute left-1/2 top-1/2 w-3 -translate-y-1/2 border-t border-hairline" />
      </span>
      <SemanticSwatchStrip theme={theme} />
      <span className="min-w-0 flex-1 truncate text-[13px] font-medium">{theme.name}</span>
      <span className="w-12 shrink-0 font-mono text-[10px] text-muted-copy">{modeLabel}</span>
      <span className={cn('flex w-[92px] shrink-0 items-center justify-end gap-1.5 text-[11px]', active ? 'font-medium text-lavender-hi' : 'text-muted-copy')}>
        {active ? <Check aria-hidden="true" className="size-3.5" strokeWidth={2.2} /> : null}
        {active ? t('theme.active') : t('theme.use')}
      </span>
    </Button>
  )
})
