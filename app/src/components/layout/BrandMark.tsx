import { useTranslation } from 'react-i18next'

export function BrandMark() {
  const { t } = useTranslation('common')

  return (
    <span aria-label={`${t('brand.name')} · ${t('brand.tagline')}`} className="relative flex size-9 shrink-0 items-center justify-center" role="img">
      <span aria-hidden="true" className="absolute left-0 top-0 size-8 rounded-full bg-lavender" />
      <span aria-hidden="true" className="absolute right-0 top-0.5 size-7 translate-x-1 rounded-full bg-sidebar" />
    </span>
  )
}
