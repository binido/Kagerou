import { create } from 'zustand'

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

    void kagerouApi.onConnectionChanged((connected) => set({ connected }))

    void kagerouApi.onTraffic((event) => {
      if (event.kind !== 'sample') return
      set((state) => ({
        trafficSample: { download: event.down, upload: event.up },
        // A null total means the backend's `/connections` fetch failed for
        // this sample — keep the previous value rather than blanking it.
        sessionTraffic:
          event.downloadTotal !== null && event.uploadTotal !== null
            ? { download: event.downloadTotal, upload: event.uploadTotal }
            : state.sessionTraffic,
      }))
    })

    void kagerouApi.onLog((line) => {
      set((state) => ({ logs: [...state.logs.slice(-(MAX_LOG_ENTRIES - 1)), toLogEntry(line)] }))
    })

    void kagerouApi.onCrashed(() => set({ connected: false }))
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
    logs: [],
    trafficSample: { download: 0, upload: 0 },
    sessionTraffic: { download: 0, upload: 0 },
    updateAvailable: null,
    settings: {
      theme: getInitialThemeId(),
      language: 'en',
      startup: true,
      tunMode: false,
      systemProxy: false,
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
    },

    toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),

    toggleConnection: async () => {
      const { connected } = get()
      try {
        if (connected) await kagerouApi.disconnect()
        else await kagerouApi.connect()
      } catch (error) {
        console.error('toggleConnection failed', error)
      }
    },

    setProfileGroupOpen: (id, open) => {
      set((state) => ({
        profileGroups: state.profileGroups.map((group) => (group.id === id ? { ...group, open } : group)),
      }))
      void kagerouApi.setProfileGroupOpen(id, open).catch((error) => console.error('setProfileGroupOpen failed', error))
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
      } catch (error) {
        console.error('selectProfile failed', error)
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
        console.error('deleteProfile failed', error)
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

    runProfileTest: async (id, method) => {
      try {
        const result = await kagerouApi.runProfileTest(id, method)
        set((state) => ({
          profiles: state.profiles.map((profile) => (profile.id === id ? { ...profile, [method]: result } : profile)),
        }))
        return result
      } catch (error) {
        console.error('runProfileTest failed', error)
        return null
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
      void kagerouApi.setPreset(id, enabled).catch((error) => console.error('setPreset failed', error))
    },

    selectRule: (id) => {
      set((state) => ({
        routingRules: state.routingRules.map((rule) => ({ ...rule, selected: rule.id === id })),
      }))
      void kagerouApi.selectRule(id).catch((error) => console.error('selectRule failed', error))
    },

    updateRule: (id, patch) => {
      set((state) => ({
        routingRules: state.routingRules.map((rule) => (rule.id === id ? { ...rule, ...patch } : rule)),
      }))
      void kagerouApi.updateRule(id, patch).catch((error) => console.error('updateRule failed', error))
    },

    setTheme: (themeId) => {
      const theme = getTheme(themeId)
      if (!theme) return
      set((state) => ({ settings: { ...state.settings, theme: theme.id } }))
      persistThemeId(theme.id)
      void kagerouApi.setTheme(theme.id).catch((error) => console.error('setTheme failed', error))
    },

    updateSettings: (patch) => {
      set((state) => ({ settings: { ...state.settings, ...patch } }))
      void kagerouApi.updateSettings(patch).catch((error) => console.error('updateSettings failed', error))
    },
  }
})
