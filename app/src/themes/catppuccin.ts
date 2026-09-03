import type { CatppuccinFlavor } from '@catppuccin/palette'

import type { Theme, ThemeTokens } from '@/themes/types'

const translucent = (color: string, amount: number) => `color-mix(in srgb, ${color} ${amount}%, transparent)`

const toTokens = (flavor: CatppuccinFlavor): ThemeTokens => {
  const { colors } = flavor

  return {
    canvas: colors.base.hex,
    sidebar: colors.mantle.hex,
    surface: colors.surface0.hex,
    surfaceElevated: colors.surface1.hex,
    surfaceSelected: colors.surface2.hex,
    surfaceHover: colors.surface1.hex,
    viewport: colors.crust.hex,
    overlay: flavor.dark ? colors.crust.hex : colors.text.hex,
    border: colors.overlay0.hex,
    text: colors.text.hex,
    textBody: colors.subtext1.hex,
    textMuted: colors.subtext0.hex,
    textQuiet: colors.overlay0.hex,
    accent: colors.mauve.hex,
    accentHover: colors.lavender.hex,
    accentForeground: flavor.dark ? colors.crust.hex : colors.base.hex,
    ink: flavor.dark ? colors.crust.hex : colors.base.hex,
    success: colors.green.hex,
    warning: colors.yellow.hex,
    error: colors.red.hex,
    info: colors.blue.hex,
    shadowColor: translucent(colors.crust.hex, 72),
  }
}

export const createCatppuccinTheme = (id: Theme['id'], flavor: CatppuccinFlavor): Theme => ({
  id,
  packId: 'catppuccin',
  packName: 'Catppuccin',
  name: flavor.name,
  isDark: flavor.dark,
  tokens: toTokens(flavor),
})
