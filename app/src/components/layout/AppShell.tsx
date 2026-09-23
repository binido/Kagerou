import { useTranslation } from 'react-i18next'
import { Outlet } from 'react-router-dom'

import { Sidebar } from '@/components/layout/Sidebar'

export function AppShell() {
  const { t } = useTranslation('common')

  return (
    <div className="flex min-h-screen w-full overflow-x-clip bg-canvas text-primary">
      <a
        className="sr-only focus:not-sr-only focus:absolute focus:top-3 focus:left-3 focus:z-50 focus:rounded-md focus:bg-raised focus:px-3 focus:py-2 focus:focus-ring"
        href="#main"
      >
        {t('a11y.skipToContent')}
      </a>
      <Sidebar />
      {/* tabIndex -1 lets the skip link move focus here, not only scroll. */}
      <main aria-label={t('a11y.mainContent')} className="min-w-0 flex-1" id="main" tabIndex={-1}>
        <Outlet />
      </main>
    </div>
  )
}
