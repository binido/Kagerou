import { create } from 'zustand'
import { toast } from 'sonner'

import i18n from '@/i18n'
import { backendErrorMessage } from '@/lib/errors'
import { kagerouApi, type AppSnapshot } from '@/lib/tauri-api'
import { getInitialThemeId, getTheme } from '@/themes'
import { persistThemeId } from '@/themes/runtime'
import type {
  AddLocalProfileInput,
  AddSourceInput,
  KagerouStore,
  LogEntry,
  LogLevel,
  Source,
} from '@/types/kagerou'
import { TRAFFIC_HISTORY_LIMIT } from '@/types/kagerou'

const DEFAULT_PROFILE_GROUP_ID = 'default'
const MAX_LOG_ENTRIES = 500

let logSequence = 0
let backendEventsSubscribed = false

/** Test-only: lets each test re-arm event subscription instead of being
 * stuck with whichever mock handler happened to be installed first. */
export const __resetBackendEventSubscriptionForTests = () => {
  backendEventsSubscribed = false
}

const applySnapshot = (snapshot: AppSnapshot) => ({
  connected: snapshot.connected,
  connectedSince: snapshot.connectedSince,
  activeProfileId: snapshot.activeProfileId,
  profiles: snapshot.profiles,
  profileGroups: snapshot.profileGroups,
  sources: snapshot.sources,
  routingPresets: snapshot.routingPresets,
  routingRules: snapshot.routingRules,
  settings: snapshot.settings,
})

const detectLogLevel = (line: string): LogLevel => {
  if (/\berror\b/i.test(line)) return 'ERROR'
  if (/\bwarn(ing)?\b/i.test(line)) return 'WARN'
  return 'INFO'
}

const toLogEntry = (line: string): LogEntry => ({
  id: `log-${Date.now()}-${logSequence++}`,
  timestamp: new Date().toISOString(),
  level: detectLogLevel(line),
  message: line,
})

export const useKagerouStore = create<KagerouStore>((set, get) => {
  const refresh = async () => {
    const snapshot = await kagerouApi.getAppState()
    set(applySnapshot(snapshot))
  }

  const subscribeToBackendEvents = () => {
    if (backendEventsSubscribed) return
    backendEventsSubscribed = true

    // The event fires in the same breath as the backend's own stamp, so
    // `Date.now()` here and `connectedSince` in the snapshot agree to within
    // a millisecond — and the snapshot is what a reload reads, which is the
    // case the frontend clock cannot serve on its own. Going down clears the
    // history so the sparkline stops drawing the previous session's shape.
    void kagerouApi.onConnectionChanged((connected) => {
      set(connected
        ? { connected, connectedSince: Date.now(), rulesChangedSinceConnect: false }
        : { connected, connectedSince: null, trafficHistory: [], activeConnections: null })
      // The exit only exists while the core does, so this is one of the two
      // moments worth asking — the other is a profile switch.
      if (connected) void get().refreshExitLocation()
    })

    void kagerouApi.onTraffic((event) => {
      if (event.kind !== 'sample') return
      const sample = { download: event.down, upload: event.up }
      set((state) => ({
        trafficSample: sample,
        trafficHistory: [...state.trafficHistory.slice(-(TRAFFIC_HISTORY_LIMIT - 1)), sample],
        // Same rule as the totals below: a failed `/connections` fetch keeps
        // the previous count rather than blanking the readout.
        activeConnections: event.activeConnections ?? state.activeConnections,
        // A null total means the backend's `/connections` fetch failed for
        // this sample — keep the previous value rather than blanking it.
        sessionTraffic:
          event.downloadTotal !== null && event.uploadTotal !== null
            ? { download: event.downloadTotal, upload: event.uploadTotal }
            : state.sessionTraffic,
      }))
    })

    void kagerouApi.onTestProgress((event) => {
      set((state) => ({
        testRun: state.testRun ? { ...state.testRun, done: event.done, total: event.total } : state.testRun,
        profiles: state.profiles.map((profile) =>
          profile.id === event.profileId ? { ...profile, url: event.result } : profile),
      }))
    })

    void kagerouApi.onTestFinished(() => set({ testRun: null }))

    // The tray asks for the same two actions the UI offers, and goes through
    // the same code: a second path would be a second set of bugs.
    void kagerouApi.onTrayToggleConnection(() => {
      void get().toggleConnection()
    })

    void kagerouApi.onTraySelectProfile((profileId) => {
      void get().selectProfile(profileId)
    })

    void kagerouApi.onLog((line) => {
      set((state) => ({ logs: [...state.logs.slice(-(MAX_LOG_ENTRIES - 1)), toLogEntry(line)] }))
    })

    void kagerouApi.onCrashed(() =>
      set({ connected: false, connectedSince: null, trafficHistory: [], activeConnections: null }))
  }

  return {
    hydrated: false,
    sidebarCollapsed: false,
    connected: false,
    activeProfileId: '',
    profiles: [],
    profileGroups: [],
    sources: [],
    routingPresets: [],
    routingRules: [],
    rulesChangedSinceConnect: false,
    logs: [],
    trafficSample: { download: 0, upload: 0 },
    trafficHistory: [],
    activeConnections: null,
    connectedSince: null,
    exitLocation: null,
    exitLocationPending: false,
    sessionTraffic: { download: 0, upload: 0 },
    updateAvailable: null,
    testRun: null,
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

    hydrate: async () => {
      subscribeToBackendEvents()
      const snapshot = await kagerouApi.getAppState()
      set({ ...applySnapshot(snapshot), hydrated: true })
      // Deliberately not awaited: a slow or unreachable GitHub must not hold
      // up the first paint, and the command never rejects.
      void kagerouApi.checkForUpdate().then((updateAvailable) => set({ updateAvailable }))
      // A reload mid-session lands here with the tunnel already up, and the
      // connection-changed event that would have asked has long since fired.
      if (snapshot.connected) void get().refreshExitLocation()
    },

    toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),

    toggleConnection: async () => {
      const { connected } = get()
      try {
        if (connected) await kagerouApi.disconnect()
        else await kagerouApi.connect()
      } catch (error) {
        toast.error(backendErrorMessage(
          error,
          i18n.t(connected ? 'common:feedback.disconnectFailed' : 'common:feedback.connectFailed'),
        ))
      }
    },

    // Rollback is a full refresh rather than a captured value: the snapshot
    // is what the database actually accepted and cannot drift from it.
    setProfileGroupOpen: (id, open) => {
      set((state) => ({
        profileGroups: state.profileGroups.map((group) => (group.id === id ? { ...group, open } : group)),
      }))
      void kagerouApi.setProfileGroupOpen(id, open).catch(async (error) => {
        toast.error(backendErrorMessage(error, i18n.t('common:feedback.groupOpenSaveFailed')))
        await refresh()
      })
    },

    addProfileGroup: async (label) => {
      try {
        const id = await kagerouApi.addProfileGroup(label)
        await refresh()
        return id
      } catch {
        return null
      }
    },

    renameProfileGroup: async (id, label) => {
      try {
        await kagerouApi.renameProfileGroup(id, label)
        await refresh()
        return true
      } catch {
        return false
      }
    },

    selectProfile: async (id) => {
      if (!get().profiles.some((profile) => profile.id === id)) return
      set((state) => ({
        activeProfileId: id,
        profiles: state.profiles.map((profile) => ({ ...profile, selected: profile.id === id })),
      }))
      try {
        await kagerouApi.selectProfile(id)
        // A hot switch changes the exit without touching the connection, so
        // nothing else would invalidate the location.
        void get().refreshExitLocation()
      } catch (error) {
        toast.error(backendErrorMessage(error, i18n.t('common:feedback.profileSelectFailed')))
        await refresh()
      }
    },

    addLocalProfile: async (input: AddLocalProfileInput) => {
      try {
        const id = await kagerouApi.addLocalProfile({ ...input, groupId: input.groupId ?? DEFAULT_PROFILE_GROUP_ID })
        await refresh()
        return id
      } catch {
        return null
      }
    },

    renameProfile: async (id, name) => {
      try {
        await kagerouApi.renameProfile(id, name)
        await refresh()
        return true
      } catch {
        return false
      }
    },

    deleteProfile: async (id) => {
      try {
        await kagerouApi.deleteProfile(id)
        await refresh()
      } catch (error) {
        toast.error(backendErrorMessage(error, i18n.t('common:feedback.profileDeleteFailed')))
      }
    },

    moveProfileToGroup: async (profileId, targetGroupId) => {
      try {
        await kagerouApi.moveProfileToGroup(profileId, targetGroupId)
        await refresh()
        return true
      } catch {
        return false
      }
    },

    moveProfile: async (id, direction) => {
      try {
        await kagerouApi.moveProfile(id, direction)
        await refresh()
        return true
      } catch {
        return false
      }
    },

    reorderProfiles: async (fromId, toId) => {
      try {
        await kagerouApi.reorderProfiles(fromId, toId)
        await refresh()
        return true
      } catch {
        return false
      }
    },

    runProfileTest: async (id) => {
      try {
        const result = await kagerouApi.runProfileTest(id)
        set((state) => ({
          profiles: state.profiles.map((profile) => (profile.id === id ? { ...profile, url: result } : profile)),
        }))
        return result
      } catch (error) {
        console.error('runProfileTest failed', error)
        return null
      }
    },

    startGroupTest: async (groupId) => {
      // Optimistic so the progress bar appears on the click rather than after
      // the first profile has been measured, which can be seconds later.
      set({ testRun: { groupId, done: 0, total: 0 } })
      try {
        const total = await kagerouApi.startGroupTest(groupId)
        if (total === 0) set({ testRun: null })
      } catch (error) {
        toast.error(backendErrorMessage(error, i18n.t('common:feedback.groupTestStartFailed')))
        set({ testRun: null })
      }
    },

    cancelGroupTest: async () => {
      try {
        await kagerouApi.cancelGroupTest()
      } catch (error) {
        toast.error(backendErrorMessage(error, i18n.t('common:feedback.groupTestCancelFailed')))
      }
    },

    clearGroupTestResults: async (groupId) => {
      try {
        await kagerouApi.clearGroupTestResults(groupId)
        await refresh()
      } catch (error) {
        toast.error(backendErrorMessage(error, i18n.t('common:feedback.groupTestClearFailed')))
      }
    },

    deleteUnavailableProfiles: async (groupId) => {
      try {
        const deleted = await kagerouApi.deleteUnavailableProfiles(groupId)
        await refresh()
        return deleted
      } catch (error) {
        toast.error(backendErrorMessage(error, i18n.t('common:feedback.deleteUnavailableFailed')))
        return 0
      }
    },

    addSource: async (input: AddSourceInput) => {
      try {
        const id = await kagerouApi.addSource(input)
        await refresh()
        return id
      } catch {
        return null
      }
    },

    updateSource: async (id, patch: Partial<Pick<Source, 'name' | 'value'>>) => {
      try {
        await kagerouApi.updateSource(id, patch)
        await refresh()
        return true
      } catch {
        return false
      }
    },

    refreshSource: async (id) => {
      await kagerouApi.refreshSource(id)
      await refresh()
    },

    removeSource: async (id) => {
      try {
        await kagerouApi.removeSource(id)
        await refresh()
        return true
      } catch {
        return false
      }
    },

    setPreset: (id, enabled) => {
      set((state) => ({
        routingPresets: state.routingPresets.map((preset) => (preset.id === id ? { ...preset, enabled } : preset)),
      }))
      void kagerouApi.setPreset(id, enabled).catch(async (error) => {
        toast.error(backendErrorMessage(error, i18n.t('common:feedback.presetSaveFailed')))
        await refresh()
      })
    },

    selectRule: (id) => {
      set((state) => ({
        routingRules: state.routingRules.map((rule) => ({ ...rule, selected: rule.id === id })),
      }))
      void kagerouApi.selectRule(id).catch(async (error) => {
        toast.error(backendErrorMessage(error, i18n.t('common:feedback.ruleSelectFailed')))
        await refresh()
      })
    },

    updateRule: (id, patch) => {
      const rulesChangedBefore = get().rulesChangedSinceConnect
      set((state) => ({
        routingRules: state.routingRules.map((rule) => (rule.id === id ? { ...rule, ...patch } : rule)),
        rulesChangedSinceConnect: state.connected || state.rulesChangedSinceConnect,
      }))
      void kagerouApi.updateRule(id, patch).catch(async (error) => {
        toast.error(backendErrorMessage(error, i18n.t('common:feedback.ruleSaveFailed')))
        // refresh() restores the rules but not this frontend-only flag; the
        // failed write changed nothing the core would need to reload.
        set({ rulesChangedSinceConnect: rulesChangedBefore })
        await refresh()
      })
    },

    // Waits for the backend because the new rule's id comes from there.
    addRule: async (match, outbound) => {
      try {
        const id = await kagerouApi.addRoutingRule(match, outbound)
        await refresh()
        set((state) => ({ rulesChangedSinceConnect: state.connected || state.rulesChangedSinceConnect }))
        return id
      } catch {
        return null
      }
    },

    deleteRule: async (id) => {
      try {
        await kagerouApi.deleteRoutingRule(id)
        set((state) => ({
          routingRules: state.routingRules.filter((rule) => rule.id !== id),
          rulesChangedSinceConnect: state.connected || state.rulesChangedSinceConnect,
        }))
        return true
      } catch {
        return false
      }
    },

    setTheme: (themeId) => {
      const theme = getTheme(themeId)
      if (!theme) return
      const previousThemeId = get().settings.theme
      set((state) => ({ settings: { ...state.settings, theme: theme.id } }))
      persistThemeId(theme.id)
      void kagerouApi.setTheme(theme.id).catch(async (error) => {
        toast.error(backendErrorMessage(error, i18n.t('common:feedback.themeSaveFailed')))
        // refresh() restores the stored theme but not the localStorage copy.
        persistThemeId(previousThemeId)
        await refresh()
      })
    },

    /** Failure is not worth surfacing: a null location is the signal, and the
     * dashboard falls back to the profile's own flag. */
    refreshExitLocation: async () => {
      set({ exitLocationPending: true })
      try {
        set({ exitLocation: await kagerouApi.lookupExitLocation() })
      } catch (error) {
        console.error('lookupExitLocation failed', error)
        set({ exitLocation: null })
      } finally {
        set({ exitLocationPending: false })
      }
    },

    updateSettings: (patch) => {
      set((state) => ({ settings: { ...state.settings, ...patch } }))
      void kagerouApi.updateSettings(patch).catch(async (error) => {
        toast.error(backendErrorMessage(error, i18n.t('common:feedback.settingsSaveFailed')))
        await refresh()
      })
    },
  }
})
