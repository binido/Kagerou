import { useTranslation } from 'react-i18next'

import { ConnectionStage } from '@/components/dashboard/ConnectionStage'
import { ModeSwitches } from '@/components/dashboard/ModeSwitches'
import { QuickProfiles } from '@/components/dashboard/QuickProfiles'
import { StatusFooter } from '@/components/dashboard/StatusFooter'
import { PageContainer } from '@/components/layout/PageContainer'
import { PageHeader } from '@/components/layout/PageHeader'
import { latencyOf, sortProfiles } from '@/lib/profile-sorting'
import { useKagerouStore } from '@/store/kagerou-store'

const QUICK_PROFILE_COUNT = 4

export function DashboardPage() {
  const { t } = useTranslation('dashboard')
  const { t: tp } = useTranslation('profiles')
  const connected = useKagerouStore((state) => state.connected)
  const connectedSince = useKagerouStore((state) => state.connectedSince)
  const exitLocation = useKagerouStore((state) => state.exitLocation)
  const exitLocationPending = useKagerouStore((state) => state.exitLocationPending)
  const profiles = useKagerouStore((state) => state.profiles)
  const profileGroups = useKagerouStore((state) => state.profileGroups)
  const activeProfileId = useKagerouStore((state) => state.activeProfileId)
  const trafficSample = useKagerouStore((state) => state.trafficSample)
  const trafficHistory = useKagerouStore((state) => state.trafficHistory)
  const activeConnections = useKagerouStore((state) => state.activeConnections)
  const sessionTraffic = useKagerouStore((state) => state.sessionTraffic)
  const routingPresets = useKagerouStore((state) => state.routingPresets)
  const routingRules = useKagerouStore((state) => state.routingRules)
  const sources = useKagerouStore((state) => state.sources)
  const settings = useKagerouStore((state) => state.settings)
  const testRun = useKagerouStore((state) => state.testRun)
  const toggleConnection = useKagerouStore((state) => state.toggleConnection)
  const selectProfile = useKagerouStore((state) => state.selectProfile)
  const startGroupTest = useKagerouStore((state) => state.startGroupTest)
  const updateSettings = useKagerouStore((state) => state.updateSettings)
  const refreshExitLocation = useKagerouStore((state) => state.refreshExitLocation)

  const activeProfile = profiles.find((profile) => profile.id === activeProfileId) ?? profiles[0]
  const group = activeProfile ? profileGroups.find((g) => g.id === activeProfile.groupId) : undefined
  const groupLabel = group?.kind === 'default' ? tp('group.defaultName') : group?.label
  const profileName = activeProfile
    ? groupLabel
      ? t('connection.profile', { group: groupLabel, name: activeProfile.name })
      : activeProfile.name
    : t('connection.fallbackProfile')
  const ping = activeProfile ? activeProfile.url : { value: 'Not tested', tone: 'muted' as const }

  // The quick list ranks the active profile's own group. Without a single
  // measured latency the ranking is meaningless — every profile ties at
  // infinity — so the card offers the test instead of an arbitrary slice.
  const groupProfiles = group ? profiles.filter((profile) => profile.groupId === group.id) : []
  const ranked = groupProfiles.some((profile) => latencyOf(profile) !== null)

  return (
    <PageContainer className="flex h-dvh min-h-0 flex-col overflow-hidden" contentClassName="flex h-full min-h-0 flex-col">
      <PageHeader eyebrow={t('page.eyebrow')} title={t('page.title')} />
      <section aria-labelledby="connection-stage-title" className="mt-5 flex min-h-0 max-h-[700px] flex-1 flex-col gap-4">
        <ConnectionStage
          activeConnections={activeConnections}
          connected={connected}
          connectedSince={connectedSince}
          exitLocation={exitLocation}
          exitLocationPending={exitLocationPending}
          latestDownload={trafficSample.download}
          latestUpload={trafficSample.upload}
          onRefreshLocation={() => void refreshExitLocation()}
          onToggleConnection={toggleConnection}
          ping={ping}
          profileName={profileName}
          sessionTraffic={sessionTraffic}
          trafficHistory={trafficHistory}
        />
        <div className="grid shrink-0 grid-cols-2 gap-4 max-[720px]:grid-cols-1">
          <ModeSwitches
            onToggle={(mode) => updateSettings(mode === 'tun' ? { tunMode: !settings.tunMode } : { systemProxy: !settings.systemProxy })}
            systemProxy={settings.systemProxy}
            tunMode={settings.tunMode}
          />
          <QuickProfiles
            activeProfileId={activeProfile?.id ?? ''}
            onSelect={(id) => void selectProfile(id)}
            onTestGroup={() => void startGroupTest(group?.id ?? null)}
            profiles={sortProfiles(groupProfiles, 'ping').slice(0, QUICK_PROFILE_COUNT)}
            ranked={ranked}
            testRunning={testRun !== null}
          />
        </div>
        <StatusFooter
          className="shrink-0 [@media(max-height:700px)]:hidden"
          presets={routingPresets}
          rules={routingRules}
          sources={sources}
        />
      </section>
    </PageContainer>
  )
}
