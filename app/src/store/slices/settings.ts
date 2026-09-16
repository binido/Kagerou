import { kagerouApi } from '@/lib/tauri-api'
import { getInitialThemeId, getTheme } from '@/themes'
import { persistThemeId } from '@/themes/runtime'
import type { ThemeId } from '@/themes/types'
import type { SettingsState } from '@/types/kagerou'

import { refresh, report, type Slice } from '../shared'

export interface SettingsSlice {
  settings: SettingsState
  setTheme: (themeId: ThemeId) => void
  updateSettings: (patch: Partial<SettingsState>) => void
}

/** What the user has chosen. The defaults here only stand until `hydrate`
 * replaces them with what is stored. */
export const createSettingsSlice: Slice<SettingsSlice> = (set, get) => ({
  settings: {
    theme: getInitialThemeId(),
    language: 'en',
    startup: false,
    geoLookup: true,
    tunMode: false,
    systemProxy: false,
    autoConnect: false,
    tunInterface: 'utun / tun0',
    autoUpdateSubscriptions: false,
    subscriptionUpdateInterval: '30',
    customSubscriptionUpdateMinutes: 60,
    groupSort: 'ping',
    logLevel: 'info',
    testUrl: 'http://www.gstatic.com/generate_204',
  },

  setTheme: (themeId) => {
    const theme = getTheme(themeId)
    if (!theme) return
    const previousThemeId = get().settings.theme
    set((state) => ({ settings: { ...state.settings, theme: theme.id } }))
    persistThemeId(theme.id)
    void kagerouApi.setTheme(theme.id).catch(async (error) => {
      report(error, 'common:feedback.themeSaveFailed')
      // refresh() restores the stored theme but not the localStorage copy.
      persistThemeId(previousThemeId)
      await refresh(set)
    })
  },

  updateSettings: (patch) => {
    // Mirrors the backend: the connection modes are exclusive.
    const exclusive = patch.tunMode ? { systemProxy: false } : patch.systemProxy ? { tunMode: false } : {}
    set((state) => ({ settings: { ...state.settings, ...exclusive, ...patch } }))
    void kagerouApi.updateSettings(patch).catch(async (error) => {
      report(error, 'common:feedback.settingsSaveFailed')
      await refresh(set)
    })
  },
})
