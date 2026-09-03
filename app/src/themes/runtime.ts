import { DEFAULT_THEME_ID, getInitialThemeId, getTheme, THEME_STORAGE_KEY } from '@/themes'
import type { Theme, ThemeId } from '@/themes/types'

const themeColorMetaName = 'theme-color'

const getCssVariables = (theme: Theme): Readonly<Record<string, string>> => ({
  '--canvas': theme.tokens.canvas,
  '--sidebar': theme.tokens.sidebar,
  '--surface': theme.tokens.surface,
  '--raised': theme.tokens.surfaceElevated,
  '--selected': theme.tokens.surfaceSelected,
  '--row-hover': theme.tokens.surfaceHover,
  '--viewport': theme.tokens.viewport,
  '--overlay': theme.tokens.overlay,
  '--hairline': theme.tokens.border,
  '--text-primary': theme.tokens.text,
  '--body-copy': theme.tokens.textBody,
  '--text-muted': theme.tokens.textMuted,
  '--text-quiet': theme.tokens.textQuiet,
  '--lavender': theme.tokens.accent,
  '--lavender-hi': theme.tokens.accentHover,
  '--ink': theme.tokens.ink,
  '--good': theme.tokens.success,
  '--warn': theme.tokens.warning,
  '--bad': theme.tokens.error,
  '--upload-line': theme.tokens.info,
  '--shadow-color': theme.tokens.shadowColor,
  '--background': theme.tokens.canvas,
  '--foreground': theme.tokens.text,
  '--card': theme.tokens.surface,
  '--card-foreground': theme.tokens.text,
  '--popover': theme.tokens.surfaceElevated,
  '--popover-foreground': theme.tokens.text,
  '--primary': theme.tokens.text,
  '--primary-foreground': theme.tokens.accentForeground,
  '--secondary': theme.tokens.surfaceElevated,
  '--secondary-foreground': theme.tokens.textBody,
  '--muted': theme.tokens.surfaceElevated,
  '--muted-foreground': theme.tokens.textMuted,
  '--accent': theme.tokens.surfaceSelected,
  '--accent-foreground': theme.tokens.text,
  '--destructive': theme.tokens.error,
  '--border': theme.tokens.border,
  '--input': theme.tokens.surface,
  '--ring': theme.tokens.accent,
  '--chart-1': theme.tokens.accent,
  '--chart-2': theme.tokens.info,
  '--chart-3': theme.tokens.success,
  '--chart-4': theme.tokens.warning,
  '--chart-5': theme.tokens.error,
  '--sidebar-foreground': theme.tokens.textBody,
  '--sidebar-primary': theme.tokens.accent,
  '--sidebar-primary-foreground': theme.tokens.accentForeground,
  '--sidebar-accent': theme.tokens.surfaceSelected,
  '--sidebar-accent-foreground': theme.tokens.text,
  '--sidebar-border': theme.tokens.border,
  '--sidebar-ring': theme.tokens.accent,
})

export const persistThemeId = (themeId: ThemeId) => {
  if (typeof window !== 'undefined') window.localStorage.setItem(THEME_STORAGE_KEY, themeId)
}

export const applyTheme = (theme: Theme) => {
  if (typeof document === 'undefined') return

  const root = document.documentElement
  root.dataset.theme = theme.id
  root.classList.toggle('dark', theme.isDark)
  root.style.colorScheme = theme.isDark ? 'dark' : 'light'

  Object.entries(getCssVariables(theme)).forEach(([property, value]) => {
    root.style.setProperty(property, value)
  })

  document.querySelector<HTMLMetaElement>(`meta[name="${themeColorMetaName}"]`)?.setAttribute('content', theme.tokens.canvas)
}

export const initializeTheme = () => {
  const theme = getTheme(getInitialThemeId()) ?? getTheme(DEFAULT_THEME_ID)
  if (theme) applyTheme(theme)
  return theme
}
