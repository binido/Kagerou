import { useLayoutEffect, type ReactNode } from 'react'

import { useKagerouStore } from '@/store/kagerou-store'
import { DEFAULT_THEME_ID, getTheme } from '@/themes'
import { applyTheme } from '@/themes/runtime'

export function ThemeProvider({ children }: { children: ReactNode }) {
  const themeId = useKagerouStore((state) => state.settings.theme)

  useLayoutEffect(() => {
    const theme = getTheme(themeId) ?? getTheme(DEFAULT_THEME_ID)
    if (theme) applyTheme(theme)
  }, [themeId])

  return children
}
