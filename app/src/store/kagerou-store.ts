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
import {
  DEFAULT_PROFILE_GROUP_ID,
  canMoveProfileToGroup,
  normalizeGroupName,
} from '@/lib/profile-groups'
import { getInitialThemeId, getTheme } from '@/themes'
import { persistThemeId } from '@/themes/runtime'
import type {
  AddLocalProfileInput,
  AddSourceInput,
  KagerouStore,
  Profile,
  ProfileDraft,
  ProfileGroup,
  Source,
} from '@/types/kagerou'

let localProfileSequence = 1

const createId = (prefix: string) => `${prefix}-${Date.now()}-${localProfileSequence++}`

const protocolFromKey = (key: string): Profile['protocol'] => {
  const scheme = key.trim().split('://')[0]?.toLowerCase()
  if (scheme === 'vmess') return 'VMess'
  if (scheme === 'trojan') return 'Trojan'
  if (scheme === 'ss') return 'Shadowsocks'
  if (scheme === 'hysteria2') return 'Hysteria2'
  return 'VLESS'
}

const createLocalProfile = ({ name, key, groupId, sourceId }: AddLocalProfileInput, id = createId('local')): Profile => ({
  id,
  name: name.trim(),
  region: 'Local profile',
  protocol: protocolFromKey(key),
  origin: 'local',
  groupId: groupId ?? DEFAULT_PROFILE_GROUP_ID,
  sourceId,
  selected: false,
  tcp: { value: 'Not tested', tone: 'muted' },
  url: { value: 'Not tested', tone: 'muted' },
  key: key.trim(),
})

const createImportedProfile = (draft: ProfileDraft, groupId: string, sourceId: string, id = createId('imported')): Profile => ({
  ...draft,
  id,
  groupId,
  sourceId,
  selected: false,
})

const groupForProfile = (profileId: string, groups: ProfileGroup[]) =>
  groups.find((group) => group.profileIds.includes(profileId))

const sourceGroupForSource = (sourceId: string, groups: ProfileGroup[]) =>
  groups.find((group) => group.kind === 'subscription' && group.sourceId === sourceId)

const sourceNameForKey = (value: string, fallbackNumber: number) => {
  const scheme = value.split('://')[0]?.toUpperCase() || 'VPN'
  return `${scheme} key ${String(fallbackNumber).padStart(2, '0')}`
}

const replaceGroupProfiles = (
  state: Pick<KagerouStore, 'profiles' | 'profileGroups' | 'activeProfileId'>,
  sourceId: string,
  drafts: ProfileDraft[],
) => {
  const group = sourceGroupForSource(sourceId, state.profileGroups)
  if (!group) return null

  const existing = state.profiles.filter((profile) => profile.groupId === group.id)
  const existingByKey = new Map(existing.map((profile) => [profile.key, profile]))
  const activeProfile = state.profiles.find((profile) => profile.id === state.activeProfileId)
  const nextProfiles = drafts.map((draft) => {
    const matching = existingByKey.get(draft.key)
    return createImportedProfile(draft, group.id, sourceId, matching?.id)
  })
  const nextIds = new Set(nextProfiles.map((profile) => profile.id))
  const nextActiveId = activeProfile && activeProfile.groupId === group.id
    ? nextProfiles.find((profile) => profile.key === activeProfile.key)?.id ?? nextProfiles[0]?.id ?? state.activeProfileId
    : state.activeProfileId

  return {
    profileGroups: state.profileGroups.map((candidate) =>
      candidate.id === group.id ? { ...candidate, profileIds: nextProfiles.map((profile) => profile.id) } : candidate,
    ),
    profiles: [
      ...state.profiles.filter((profile) => profile.groupId !== group.id && !nextIds.has(profile.id)),
      ...nextProfiles.map((profile) => ({ ...profile, selected: profile.id === nextActiveId })),
    ],
    activeProfileId: nextActiveId,
  }
}

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
  settings: { ...initialSettings, theme: getInitialThemeId() },

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
  addProfileGroup: (label) => {
    const trimmed = label.trim().replace(/\s+/g, ' ')
    if (!trimmed || get().profileGroups.some((group) => normalizeGroupName(group.label) === normalizeGroupName(trimmed))) return null

    const id = createId('group')
    set((state) => ({
      profileGroups: [
        ...state.profileGroups,
        { id, label: trimmed, kind: 'custom', profileIds: [], open: true },
      ],
    }))
    return id
  },
  renameProfileGroup: (id, label) => {
    const trimmed = label.trim().replace(/\s+/g, ' ')
    const current = get().profileGroups.find((group) => group.id === id)
    if (
      !trimmed ||
      !current ||
      current.kind === 'default' ||
      get().profileGroups.some((group) => group.id !== id && normalizeGroupName(group.label) === normalizeGroupName(trimmed))
    ) return false

    set((state) => ({
      profileGroups: state.profileGroups.map((group) => (group.id === id ? { ...group, label: trimmed } : group)),
      sources: current.sourceId
        ? state.sources.map((source) => (source.id === current.sourceId ? { ...source, name: trimmed } : source))
        : state.sources,
    }))
    return true
  },
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
  renameProfile: (id, name) => {
    const trimmed = name.trim().replace(/\s+/g, ' ')
    const profile = get().profiles.find((candidate) => candidate.id === id)
    if (!profile || profile.origin !== 'local' || !trimmed) return false
    set((state) => ({
      profiles: state.profiles.map((candidate) => candidate.id === id ? { ...candidate, name: trimmed } : candidate),
    }))
    return true
  },
  addLocalProfile: (input) => {
    const targetGroupId = input.groupId ?? DEFAULT_PROFILE_GROUP_ID
    const targetGroup = get().profileGroups.find((group) => group.id === targetGroupId)
    if (!targetGroup || targetGroup.kind === 'subscription') return null

    const profile = createLocalProfile({ ...input, groupId: targetGroupId })
    set((state) => ({
      profiles: [...state.profiles, profile],
      profileGroups: state.profileGroups.map((group) =>
        group.id === targetGroupId ? { ...group, profileIds: [...group.profileIds, profile.id], open: true } : group,
      ),
    }))
    return profile.id
  },
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
  moveProfileToGroup: (profileId, targetGroupId) => {
    const profile = get().profiles.find((candidate) => candidate.id === profileId)
    const currentGroup = profile ? groupForProfile(profileId, get().profileGroups) : undefined
    const targetGroup = get().profileGroups.find((group) => group.id === targetGroupId)
    if (!profile || !canMoveProfileToGroup(profile, currentGroup, targetGroup)) return false

    set((state) => ({
      profiles: state.profiles.map((candidate) => candidate.id === profileId ? { ...candidate, groupId: targetGroupId } : candidate),
      profileGroups: state.profileGroups.map((group) => {
        if (group.id === currentGroup?.id) return { ...group, profileIds: group.profileIds.filter((id) => id !== profileId) }
        if (group.id === targetGroupId) return { ...group, profileIds: [...group.profileIds, profileId], open: true }
        return group
      }),
    }))
    return true
  },
  moveProfile: (id, direction) => {
    const profile = get().profiles.find((candidate) => candidate.id === id)
    const group = profile ? groupForProfile(id, get().profileGroups) : undefined
    if (!profile || !group) return false
    const index = group.profileIds.indexOf(id)
    const nextIndex = direction === 'up' ? index - 1 : index + 1
    if (index < 0 || nextIndex < 0 || nextIndex >= group.profileIds.length) return false
    const profileIds = [...group.profileIds]
    ;[profileIds[index], profileIds[nextIndex]] = [profileIds[nextIndex], profileIds[index]]
    set((state) => ({
      profileGroups: state.profileGroups.map((candidate) =>
        candidate.id === group.id ? { ...candidate, profileIds } : candidate,
      ),
    }))
    return true
  },
  reorderProfiles: (fromId, toId) => {
    if (fromId === toId) return false
    const fromProfile = get().profiles.find((profile) => profile.id === fromId)
    const toProfile = get().profiles.find((profile) => profile.id === toId)
    if (!fromProfile || !toProfile || fromProfile.groupId !== toProfile.groupId) return false
    const group = get().profileGroups.find((candidate) => candidate.id === fromProfile.groupId)
    if (!group) return false
    const profileIds = [...group.profileIds]
    const fromIndex = profileIds.indexOf(fromId)
    const toIndex = profileIds.indexOf(toId)
    if (fromIndex < 0 || toIndex < 0) return false
    profileIds.splice(fromIndex, 1)
    profileIds.splice(toIndex, 0, fromId)
    set((state) => ({
      profileGroups: state.profileGroups.map((candidate) =>
        candidate.id === group.id ? { ...candidate, profileIds } : candidate,
      ),
    }))
    return true
  },
  setTestResult: (id, method, result) =>
    set((state) => ({
      profiles: state.profiles.map((profile) =>
        profile.id === id ? { ...profile, [method]: result } : profile,
      ),
    })),
  replaceSubscriptionProfiles: (sourceId, drafts) => {
    const replacement = replaceGroupProfiles(get(), sourceId, drafts)
    if (!replacement) return false
    set(replacement)
    return true
  },
  addSource: (input: AddSourceInput, importedProfiles = []) => {
    const value = input.value.trim()
    const sourceId = createId('source')
    const name = input.name?.trim() || (input.type === 'key' ? sourceNameForKey(value, get().sources.length + 1) : `Subscription ${String(get().sources.length + 1).padStart(2, '0')}`)
    const source: Source = {
      id: sourceId,
      name,
      type: input.type,
      value,
      status: input.type === 'url' ? 'up-to-date' : 'ready',
      lastRefresh: input.type === 'url' ? 'Updated just now' : 'Added just now',
      originLabel: input.type === 'url' ? 'Remote URL' : 'Local key',
    }

    if (input.type === 'key') {
      const profile = createLocalProfile({ name, key: value, sourceId })
      set((state) => ({
        sources: [...state.sources, source],
        profiles: [...state.profiles, profile],
        profileGroups: state.profileGroups.map((group) => group.id === DEFAULT_PROFILE_GROUP_ID
          ? { ...group, profileIds: [...group.profileIds, profile.id], open: true }
          : group),
      }))
      return sourceId
    }

    const groupId = `subscription-${sourceId}`
    const profiles = importedProfiles.map((profile) => createImportedProfile(profile, groupId, sourceId))
    set((state) => ({
      sources: [...state.sources, source],
      profiles: [...state.profiles, ...profiles],
      profileGroups: [
        ...state.profileGroups,
        { id: groupId, label: name, kind: 'subscription', sourceId, profileIds: profiles.map((profile) => profile.id), open: true },
      ],
    }))
    return sourceId
  },
  updateSource: (id, patch) => {
    const source = get().sources.find((candidate) => candidate.id === id)
    if (!source) return false
    const nextName = patch.name?.trim()
    if (patch.name !== undefined && !nextName) return false

    set((state) => ({
      sources: state.sources.map((candidate) => candidate.id === id ? { ...candidate, ...patch, ...(nextName ? { name: nextName } : {}) } : candidate),
      profileGroups: source.type === 'url' && nextName
        ? state.profileGroups.map((group) => group.sourceId === id ? { ...group, label: nextName } : group)
        : state.profileGroups,
    }))
    return true
  },
  removeSource: (id) => {
    const source = get().sources.find((candidate) => candidate.id === id)
    if (!source) return false

    set((state) => ({
      sources: state.sources.filter((candidate) => candidate.id !== id),
      profiles: state.profiles.map((profile) => profile.sourceId === id ? { ...profile, sourceId: undefined, origin: source.type === 'url' ? 'local' : profile.origin } : profile),
      profileGroups: state.profileGroups.map((group) => group.sourceId === id
        ? { ...group, kind: 'custom', sourceId: undefined }
        : group),
    }))
    return true
  },
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
  setTheme: (themeId) => {
    const theme = getTheme(themeId)
    if (!theme) return
    set((state) => ({ settings: { ...state.settings, theme: theme.id } }))
    persistThemeId(theme.id)
  },
  updateSettings: (patch) => set((state) => ({ settings: { ...state.settings, ...patch } })),
}))
