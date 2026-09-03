import { useTranslation } from 'react-i18next'

import { SettingSelectRow } from '@/components/settings/SettingSelectRow'
import { changeAppLanguage } from '@/i18n'
import { useKagerouStore } from '@/store/kagerou-store'
import type { Language } from '@/types/kagerou'

const isLanguage = (value: string): value is Language => value === 'en' || value === 'ru'

export function LanguageSwitcher() {
  const { i18n, t } = useTranslation('settings')
  const updateSettings = useKagerouStore((state) => state.updateSettings)
  const resolvedLanguage = i18n.resolvedLanguage ?? i18n.language
  const currentLanguage: Language = isLanguage(resolvedLanguage) ? resolvedLanguage : 'en'

  const handleChange = (value: string) => {
    if (!isLanguage(value)) return
    updateSettings({ language: value })
    void changeAppLanguage(value)
  }

  return (
    <SettingSelectRow
      id="language"
      label={t('fields.language')}
      onChange={handleChange}
      options={[
        { value: 'en', label: t('language.options.en') },
        { value: 'ru', label: t('language.options.ru') },
      ]}
      value={currentLanguage}
    />
  )
}
