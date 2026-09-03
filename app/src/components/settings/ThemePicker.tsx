import { animate } from 'motion'
import { ChevronDown } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { SemanticSwatchStrip } from '@/components/settings/SemanticSwatchStrip'
import { ThemeFlavorRow } from '@/components/settings/ThemeFlavorRow'
import { DEFAULT_THEME_ID, getTheme, themePacks, themes } from '@/themes'
import type { ThemeId } from '@/themes/types'

interface ThemePickerProps {
  value: ThemeId
  onChange: (themeId: ThemeId) => void
}

const getReducedMotionPreference = () => typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches

export function ThemePicker({ value, onChange }: ThemePickerProps) {
  const { t } = useTranslation('settings')
  const [open, setOpen] = useState(false)
  const contentRef = useRef<HTMLDivElement>(null)
  const chevronRef = useRef<SVGSVGElement>(null)
  const rowRefs = useRef<Record<string, HTMLButtonElement | null>>({})
  const activeTheme = getTheme(value) ?? getTheme(DEFAULT_THEME_ID) ?? themes[0]
  const allThemes = themes

  useEffect(() => {
    const controls = chevronRef.current
      ? animate(chevronRef.current, { rotate: open ? 180 : 0 }, { duration: getReducedMotionPreference() ? 0 : 0.12, ease: 'easeOut' })
      : null
    return () => controls?.stop()
  }, [open])

  useEffect(() => {
    if (!open || !contentRef.current) return
    const controls = animate(contentRef.current, { opacity: [0, 1], y: [-4, 0] }, { duration: getReducedMotionPreference() ? 0 : 0.12, ease: 'easeOut' })
    return () => controls.stop()
  }, [open])

  useEffect(() => {
    if (!open) return
    const frame = window.requestAnimationFrame(() => {
      rowRefs.current[activeTheme.id]?.focus()
    })
    return () => window.cancelAnimationFrame(frame)
  }, [open, activeTheme.id])

  const focusTheme = (index: number) => {
    const theme = allThemes[index]
    if (theme) rowRefs.current[theme.id]?.focus()
  }

  const handleRowKeyDown = (event: React.KeyboardEvent<HTMLButtonElement>, index: number) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault()
      focusTheme((index + 1) % allThemes.length)
    } else if (event.key === 'ArrowUp') {
      event.preventDefault()
      focusTheme((index - 1 + allThemes.length) % allThemes.length)
    } else if (event.key === 'Home') {
      event.preventDefault()
      focusTheme(0)
    } else if (event.key === 'End') {
      event.preventDefault()
      focusTheme(allThemes.length - 1)
    } else if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault()
      const theme = allThemes[index]
      if (theme) onChange(theme.id)
    }
  }

  return (
    <Popover onOpenChange={setOpen} open={open}>
      <PopoverTrigger asChild>
        <Button
          aria-expanded={open}
          aria-haspopup="dialog"
          aria-label={`${t('theme.ariaLabel')}: ${activeTheme.packName} / ${activeTheme.name}, ${activeTheme.isDark ? t('theme.dark') : t('theme.light')}`}
          className="h-10 w-[332px] max-w-full justify-start gap-3 rounded-md border-0 bg-surface px-3 text-body shadow-none hover:bg-row-hover hover:text-primary focus-visible:focus-ring max-[639px]:w-full"
          type="button"
          variant="outline"
        >
          <SemanticSwatchStrip theme={activeTheme} />
          <span className="min-w-0 flex-1 truncate text-left text-[13px] font-medium text-primary">
            <span className="mr-1.5 text-[11px] font-normal text-muted-copy">{activeTheme.packName} /</span>
            {activeTheme.name}
          </span>
          <span className="w-10 shrink-0 font-mono text-[10px] text-muted-copy">{activeTheme.isDark ? t('theme.dark') : t('theme.light')}</span>
          <ChevronDown ref={chevronRef} aria-hidden="true" className="size-4 shrink-0 text-muted-copy" strokeWidth={1.8} />
        </Button>
      </PopoverTrigger>
      <PopoverContent
        align="end"
        className="w-[432px] max-w-[calc(100vw-24px)] gap-0 rounded-md border-0 bg-popover p-0 text-popover-foreground shadow-none ring-0 theme-shadow"
        onOpenAutoFocus={(event) => event.preventDefault()}
        sideOffset={8}
      >
        <div ref={contentRef}>
          {themePacks.map((pack) => (
            <section aria-labelledby={`${pack.id}-theme-pack`} key={pack.id}>
              <div className="flex h-[50px] items-center justify-between border-b border-hairline px-4">
                <div className="flex items-center gap-2.5"><span className="size-2 rounded-full bg-lavender" /><p className="type-display text-[14px] text-primary" id={`${pack.id}-theme-pack`}>{pack.name}</p></div>
                <span className="font-mono text-[10px] text-muted-copy">{t('theme.installed', { installed: pack.themes.length, total: pack.themes.length })}</span>
              </div>
              <div aria-label={pack.name} className="p-2" role="radiogroup">
                {pack.themes.map((theme, packIndex) => {
                  const themeIndex = allThemes.findIndex((candidate) => candidate.id === theme.id)
                  return (
                    <ThemeFlavorRow
                      active={theme.id === activeTheme.id}
                      isFirst={packIndex === 0}
                      isLast={packIndex === pack.themes.length - 1}
                      key={theme.id}
                      onKeyDown={(event) => handleRowKeyDown(event, themeIndex)}
                      onSelect={onChange}
                      ref={(element) => { rowRefs.current[theme.id] = element }}
                      theme={theme}
                    />
                  )
                })}
              </div>
            </section>
          ))}
          <div className="flex h-7 items-center border-t border-hairline px-4 font-mono text-[9px] text-quiet max-[640px]:hidden">{t('theme.keyboard')}</div>
        </div>
      </PopoverContent>
    </Popover>
  )
}
