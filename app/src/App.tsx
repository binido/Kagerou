import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'

import { AppShell } from '@/components/layout/AppShell'
import { TooltipProvider } from '@/components/ui/tooltip'
import { Toaster } from '@/components/ui/sonner'
import { DashboardPage } from '@/pages/DashboardPage'
import { LogsPage } from '@/pages/LogsPage'
import { GroupsPage } from '@/pages/GroupsPage'
import { RoutingRulesPage } from '@/pages/RoutingRulesPage'
import { SettingsPage } from '@/pages/SettingsPage'
import { useKagerouStore } from '@/store/kagerou-store'
import { ThemeProvider } from '@/themes/ThemeProvider'

function App() {
  const { t } = useTranslation('common')
  const hydrated = useKagerouStore((state) => state.hydrated)
  const hydrateError = useKagerouStore((state) => state.hydrateError)
  const hydrate = useKagerouStore((state) => state.hydrate)

  useEffect(() => {
    void hydrate()
  }, [hydrate])

  if (!hydrated) return null

  if (hydrateError) {
    return (
      <main className="flex min-h-dvh items-center justify-center p-8">
        <div className="max-w-md space-y-3" data-selectable role="alert">
          <h1 className="type-display text-[20px] leading-none text-primary">{t('startup.title')}</h1>
          <p className="text-[13px] leading-5 text-bad">{hydrateError.message}</p>
          <p className="type-meta">{hydrateError.dataDir ? t('startup.recover', { dir: hydrateError.dataDir }) : t('startup.recoverNoDir')}</p>
        </div>
      </main>
    )
  }

  return (
    <ThemeProvider>
      <BrowserRouter>
        <TooltipProvider delayDuration={150}>
        <Routes>
          <Route element={<AppShell />}>
            <Route index element={<Navigate replace to="/dashboard" />} />
            <Route path="dashboard" element={<DashboardPage />} />
            <Route path="groups" element={<GroupsPage />} />
            <Route path="profiles" element={<Navigate replace to="/groups" />} />
            <Route path="sources" element={<Navigate replace to="/groups" />} />
            <Route path="subscriptions" element={<Navigate replace to="/groups" />} />
            <Route path="routing-rules" element={<RoutingRulesPage />} />
            <Route path="logs" element={<LogsPage />} />
            <Route path="settings" element={<SettingsPage />} />
            <Route path="*" element={<Navigate replace to="/dashboard" />} />
          </Route>
        </Routes>
          <Toaster position="bottom-right" />
        </TooltipProvider>
      </BrowserRouter>
    </ThemeProvider>
  )
}

export default App
