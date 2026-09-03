export type ThemeId = string

export interface ThemeTokens {
  canvas: string
  sidebar: string
  surface: string
  surfaceElevated: string
  surfaceSelected: string
  surfaceHover: string
  viewport: string
  overlay: string
  border: string
  text: string
  textBody: string
  textMuted: string
  textQuiet: string
  accent: string
  accentHover: string
  accentForeground: string
  ink: string
  success: string
  warning: string
  error: string
  info: string
  shadowColor: string
}

export interface Theme {
  id: ThemeId
  packId: string
  packName: string
  name: string
  isDark: boolean
  tokens: ThemeTokens
}

export interface ThemePack {
  id: string
  name: string
  themes: readonly Theme[]
}
