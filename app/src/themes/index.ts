import { catppuccinFrappe } from '@/themes/catppuccin-frappe'
import { catppuccinLatte } from '@/themes/catppuccin-latte'
import { catppuccinMacchiato } from '@/themes/catppuccin-macchiato'
import { catppuccinMocha } from '@/themes/catppuccin-mocha'
import type { Theme, ThemeId, ThemePack } from '@/themes/types'

export const DEFAULT_THEME_ID = 'catppuccin-mocha' as const
export const THEME_STORAGE_KEY = 'kagerou-theme' as const

export const themePacks: readonly ThemePack[] = [
  {
    id: 'catppuccin',
    name: 'Catppuccin',
    themes: [catppuccinLatte, catppuccinFrappe, catppuccinMacchiato, catppuccinMocha],
  },
]

export const themes: readonly Theme[] = themePacks.flatMap((pack) => pack.themes)
export const themesById = new Map<ThemeId, Theme>(themes.map((theme) => [theme.id, theme]))

export const getTheme = (themeId: string | null | undefined) => (themeId ? themesById.get(themeId) : undefined)
export const isThemeId = (themeId: string | null | undefined): themeId is ThemeId => Boolean(getTheme(themeId))
export const getInitialThemeId = (): ThemeId => {
  if (typeof window === 'undefined') return DEFAULT_THEME_ID
  const storedThemeId = window.localStorage.getItem(THEME_STORAGE_KEY)
  return isThemeId(storedThemeId) ? storedThemeId : DEFAULT_THEME_ID
}
