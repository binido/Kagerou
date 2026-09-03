import { useTranslation } from 'react-i18next'

import { LogoIcon } from '@/components/layout/LogoIcon'

export function BrandMark() {
  const { t } = useTranslation('common')

  return (
    <span aria-label={`${t('brand.name')} · ${t('brand.tagline')}`} className="flex size-9 shrink-0 items-center justify-center text-lavender" role="img">
      <LogoIcon aria-hidden="true" className="size-9" height={36} width={36} />
    </span>
  )
}

export { LogoIcon }
