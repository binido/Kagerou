import { kagerouApi } from '@/lib/tauri-api'
import type { Profile, ProfileGroup } from '@/types/kagerou'

import { refresh, report, type Slice } from '../shared'

export interface ProfilesSlice {
  profiles: Profile[]
  profileGroups: ProfileGroup[]
  activeProfileId: string
  setProfileGroupOpen: (id: string, open: boolean) => void
  addProfileGroup: (label: string) => Promise<string | null>
  renameProfileGroup: (id: string, label: string) => Promise<boolean>
  selectProfile: (id: string) => Promise<void>
  renameProfile: (id: string, name: string) => Promise<boolean>
  deleteProfile: (id: string) => Promise<void>
  moveProfileToGroup: (profileId: string, targetGroupId: string) => Promise<boolean>
  moveProfile: (id: string, direction: 'up' | 'down') => Promise<boolean>
  reorderProfiles: (fromId: string, toId: string) => Promise<boolean>
}

/** The VPNs themselves and the groups they sit in.
 *
 * Anything that needs an id or a count back from the backend waits for it
 * and then re-reads everything; the rest updates on the spot and saves in
 * the background. */
export const createProfilesSlice: Slice<ProfilesSlice> = (set, get) => ({
  profiles: [],
  profileGroups: [],
  activeProfileId: '',

  setProfileGroupOpen: (id, open) => {
    set((state) => ({
      profileGroups: state.profileGroups.map((group) =>
        group.id === id ? { ...group, open } : group,
      ),
    }))
    void kagerouApi.setProfileGroupOpen(id, open).catch(async (error) => {
      report(error, 'common:feedback.groupOpenSaveFailed')
      await refresh(set)
    })
  },

  addProfileGroup: async (label) => {
    try {
      const id = await kagerouApi.addProfileGroup(label)
      await refresh(set)
      return id
    } catch {
      return null
    }
  },

  renameProfileGroup: async (id, label) => {
    try {
      await kagerouApi.renameProfileGroup(id, label)
      await refresh(set)
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
      report(error, 'common:feedback.profileSelectFailed')
      await refresh(set)
    }
  },

  renameProfile: async (id, name) => {
    try {
      await kagerouApi.renameProfile(id, name)
      await refresh(set)
      return true
    } catch {
      return false
    }
  },

  deleteProfile: async (id) => {
    try {
      await kagerouApi.deleteProfile(id)
      await refresh(set)
    } catch (error) {
      report(error, 'common:feedback.profileDeleteFailed')
    }
  },

  moveProfileToGroup: async (profileId, targetGroupId) => {
    try {
      await kagerouApi.moveProfileToGroup(profileId, targetGroupId)
      await refresh(set)
      return true
    } catch {
      return false
    }
  },

  moveProfile: async (id, direction) => {
    try {
      await kagerouApi.moveProfile(id, direction)
      await refresh(set)
      return true
    } catch {
      return false
    }
  },

  reorderProfiles: async (fromId, toId) => {
    try {
      await kagerouApi.reorderProfiles(fromId, toId)
      await refresh(set)
      return true
    } catch {
      return false
    }
  },
})
