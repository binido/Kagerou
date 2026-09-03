import { ExternalLink } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { APP_VERSION, KAGEROU_REPOSITORY_URL } from '@/config/app-meta'

interface SettingsFooterProps {
  version?: string
}

export function SettingsFooter({ version = APP_VERSION }: SettingsFooterProps) {
  const { t } = useTranslation('settings')

  return (
    <footer className="pt-5 text-[11px] leading-4 text-muted-copy">
      <div className="flex flex-wrap items-center justify-between gap-x-6 gap-y-2">
        <a
          aria-label={t('footer.openRepository')}
          className="inline-flex min-w-0 items-center gap-1.5 text-muted-copy transition-colors hover:text-lavender-hi focus-visible:focus-ring"
          href={KAGEROU_REPOSITORY_URL}
          rel="noreferrer"
          target="_blank"
        >
          <span className="truncate">{t('footer.repository')}</span>
          <ExternalLink aria-hidden="true" className="size-3 shrink-0" strokeWidth={1.8} />
        </a>
        <span className="text-quiet">{t('footer.version', { version })}</span>
      </div>
    </footer>
  )
}
