import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'

import { AppShell } from '@/components/layout/AppShell'
import { TooltipProvider } from '@/components/ui/tooltip'
import { Toaster } from '@/components/ui/sonner'
import { DashboardPage } from '@/pages/DashboardPage'
import { LogsPage } from '@/pages/LogsPage'
import { ProfilesPage } from '@/pages/ProfilesPage'
import { RoutingRulesPage } from '@/pages/RoutingRulesPage'
import { SettingsPage } from '@/pages/SettingsPage'
import { SourcesPage } from '@/pages/SourcesPage'

function App() {
  return (
    <BrowserRouter>
      <TooltipProvider delayDuration={150}>
        <Routes>
          <Route element={<AppShell />}>
            <Route index element={<Navigate replace to="/dashboard" />} />
            <Route path="dashboard" element={<DashboardPage />} />
            <Route path="profiles" element={<ProfilesPage />} />
            <Route path="sources" element={<SourcesPage />} />
            <Route path="subscriptions" element={<Navigate replace to="/sources" />} />
            <Route path="routing-rules" element={<RoutingRulesPage />} />
            <Route path="logs" element={<LogsPage />} />
            <Route path="settings" element={<SettingsPage />} />
            <Route path="*" element={<Navigate replace to="/dashboard" />} />
          </Route>
        </Routes>
        <Toaster position="bottom-right" />
      </TooltipProvider>
    </BrowserRouter>
  )
}

export default App
