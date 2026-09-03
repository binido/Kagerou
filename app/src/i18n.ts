import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'

import enCommon from '@/locales/en/common.json'
import enDashboard from '@/locales/en/dashboard.json'
import enProfiles from '@/locales/en/profiles.json'
import enSources from '@/locales/en/sources.json'
import enRouting from '@/locales/en/routing.json'
import enLogs from '@/locales/en/logs.json'
import enSettings from '@/locales/en/settings.json'
import ruCommon from '@/locales/ru/common.json'
import ruDashboard from '@/locales/ru/dashboard.json'
import ruProfiles from '@/locales/ru/profiles.json'
import ruSources from '@/locales/ru/sources.json'
import ruRouting from '@/locales/ru/routing.json'
import ruLogs from '@/locales/ru/logs.json'
import ruSettings from '@/locales/ru/settings.json'
import type { Language } from '@/types/kagerou'

export const resources = {
  en: {
    common: enCommon,
    dashboard: enDashboard,
    profiles: enProfiles,
    sources: enSources,
    routing: enRouting,
    logs: enLogs,
    settings: enSettings,
  },
  ru: {
    common: ruCommon,
    dashboard: ruDashboard,
    profiles: ruProfiles,
    sources: ruSources,
    routing: ruRouting,
    logs: ruLogs,
    settings: ruSettings,
  },
} as const

export const supportedLanguages = ['en', 'ru'] as const satisfies readonly Language[]
export const languageStorageKey = 'kagerou-language'

const isSupportedLanguage = (value: string | null): value is Language => value === 'en' || value === 'ru'

const getInitialLanguage = (): Language => {
  if (typeof window === 'undefined') return 'en'
  const stored = window.localStorage.getItem(languageStorageKey)
  if (isSupportedLanguage(stored)) return stored
  return 'en'
}

declare module 'i18next' {
  interface CustomTypeOptions {
    defaultNS: 'common'
    resources: typeof resources.en
    returnNull: false
    strictKeyChecks: true
  }
}

void i18n
  .use(initReactI18next)
  .init({
    resources,
    lng: getInitialLanguage(),
    fallbackLng: 'en',
    defaultNS: 'common',
    ns: ['common', 'dashboard', 'profiles', 'sources', 'routing', 'logs', 'settings'],
    interpolation: {
      escapeValue: false,
    },
    react: {
      useSuspense: false,
    },
  })

i18n.on('languageChanged', (language) => {
  if (typeof window !== 'undefined' && isSupportedLanguage(language)) {
    window.localStorage.setItem(languageStorageKey, language)
  }
})

export const changeAppLanguage = async (language: Language) => {
  await i18n.changeLanguage(language)
  if (typeof window !== 'undefined') window.localStorage.setItem(languageStorageKey, language)
}

export default i18n
