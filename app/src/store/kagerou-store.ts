import { create } from 'zustand'

import {
  initialLogs,
  initialProfileGroups,
  initialProfiles,
  initialRoutingPresets,
  initialRoutingRules,
  initialSettings,
  initialSources,
  initialTelemetry,
} from '@/lib/mock-data'
import type { KagerouStore, Profile } from '@/types/kagerou'

let localProfileSequence = 1

const createLocalProfile = (name: string, key: string): Profile => ({
  id: `local-${Date.now()}-${localProfileSequence++}`,
  name,
  region: 'Local profile',
  protocol: key.toLowerCase().startsWith('vmess://')
    ? 'VMess'
    : key.toLowerCase().startsWith('trojan://')
      ? 'Trojan'
      : key.toLowerCase().startsWith('ss://')
        ? 'Shadowsocks'
        : key.toLowerCase().startsWith('hysteria2://')
          ? 'Hysteria2'
          : 'VLESS',
  origin: 'local',
  selected: false,
  tcp: { value: 'Not tested', tone: 'muted' },
  url: { value: 'Not tested', tone: 'muted' },
  key,
})

export const useKagerouStore = create<KagerouStore>((set, get) => ({
  sidebarCollapsed: false,
  connected: true,
  tunMode: true,
  systemProxy: false,
  activeProfileId: 'p-seattle',
  profiles: initialProfiles,
  profileGroups: initialProfileGroups,
  sources: initialSources,
  routingPresets: initialRoutingPresets,
  routingRules: initialRoutingRules,
  logs: initialLogs,
  telemetry: initialTelemetry,
  settings: initialSettings,

  toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
  toggleConnection: () => set((state) => ({ connected: !state.connected })),
  toggleMode: (mode) =>
    set((state) => (mode === 'tun' ? { tunMode: !state.tunMode } : { systemProxy: !state.systemProxy })),
  setProfileGroupOpen: (id, open) =>
    set((state) => ({
      profileGroups: state.profileGroups.map((group) =>
        group.id === id ? { ...group, open } : group,
      ),
    })),
  selectProfile: (id) => {
    if (!get().profiles.some((profile) => profile.id === id)) return
    set((state) => ({
      activeProfileId: id,
      profiles: state.profiles.map((profile) => ({
        ...profile,
        selected: profile.id === id,
      })),
    }))
  },
  addProfile: ({ name, key }) => {
    const profile = createLocalProfile(name, key)
    set((state) => ({
      profiles: [...state.profiles, profile],
      profileGroups: state.profileGroups.map((group, index) =>
        index === 0 ? { ...group, profileIds: [...group.profileIds, profile.id], open: true } : group,
      ),
    }))
  },
  renameProfile: (id, name) =>
    set((state) => ({
      profiles: state.profiles.map((profile) =>
        profile.id === id && profile.origin === 'local' ? { ...profile, name } : profile,
      ),
    })),
  deleteProfile: (id) => {
    const profile = get().profiles.find((candidate) => candidate.id === id)
    if (!profile || profile.origin !== 'local') return
    const remaining = get().profiles.filter((candidate) => candidate.id !== id)
    const nextActive = get().activeProfileId === id ? remaining[0]?.id ?? '' : get().activeProfileId
    set((state) => ({
      activeProfileId: nextActive,
      profiles: state.profiles
        .filter((candidate) => candidate.id !== id)
        .map((candidate) => ({ ...candidate, selected: candidate.id === nextActive })),
      profileGroups: state.profileGroups.map((group) => ({
        ...group,
        profileIds: group.profileIds.filter((profileId) => profileId !== id),
      })),
    }))
  },
  moveProfile: (id, direction) => {
    const group = get().profileGroups.find((candidate) => candidate.profileIds.includes(id))
    if (!group) return
    const index = group.profileIds.indexOf(id)
    const nextIndex = direction === 'up' ? index - 1 : index + 1
    if (index < 0 || nextIndex < 0 || nextIndex >= group.profileIds.length) return
    const profileIds = [...group.profileIds]
    ;[profileIds[index], profileIds[nextIndex]] = [profileIds[nextIndex], profileIds[index]]
    set((state) => ({
      profileGroups: state.profileGroups.map((candidate) =>
        candidate.id === group.id ? { ...candidate, profileIds } : candidate,
      ),
    }))
  },
  reorderProfiles: (fromId, toId) => {
    if (fromId === toId) return
    set((state) => ({
      profileGroups: state.profileGroups.map((group) => {
        if (!group.profileIds.includes(fromId) || !group.profileIds.includes(toId)) return group
        const profileIds = [...group.profileIds]
        const fromIndex = profileIds.indexOf(fromId)
        const toIndex = profileIds.indexOf(toId)
        profileIds.splice(fromIndex, 1)
        profileIds.splice(toIndex, 0, fromId)
        return { ...group, profileIds }
      }),
    }))
  },
  setTestResult: (id, method, result) =>
    set((state) => ({
      profiles: state.profiles.map((profile) =>
        profile.id === id ? { ...profile, [method]: result } : profile,
      ),
    })),
  addSource: (source) => set((state) => ({ sources: [...state.sources, source] })),
  updateSource: (id, patch) =>
    set((state) => ({
      sources: state.sources.map((source) => (source.id === id ? { ...source, ...patch } : source)),
    })),
  removeSource: (id) => set((state) => ({ sources: state.sources.filter((source) => source.id !== id) })),
  setPreset: (id, enabled) =>
    set((state) => ({
      routingPresets: state.routingPresets.map((preset) =>
        preset.id === id ? { ...preset, enabled } : preset,
      ),
    })),
  selectRule: (id) =>
    set((state) => ({
      routingRules: state.routingRules.map((rule) => ({ ...rule, selected: rule.id === id })),
    })),
  updateRule: (id, patch) =>
    set((state) => ({
      routingRules: state.routingRules.map((rule) => (rule.id === id ? { ...rule, ...patch } : rule)),
    })),
  updateSettings: (patch) => set((state) => ({ settings: { ...state.settings, ...patch } })),
}))
