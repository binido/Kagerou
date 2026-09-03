import type { Profile, ProfileGroup } from '@/types/kagerou'

export const DEFAULT_PROFILE_GROUP_ID = 'default'

export const normalizeGroupName = (value: string) => value.trim().replace(/\s+/g, ' ').toLocaleLowerCase()

export const isSubscriptionGroup = (group?: ProfileGroup) => group?.kind === 'subscription'

export const getProfileGroup = (profile: Profile, groups: ProfileGroup[]) =>
  groups.find((group) => group.id === profile.groupId)

export const canMoveProfileToGroup = (
  profile: Profile,
  currentGroup: ProfileGroup | undefined,
  targetGroup: ProfileGroup | undefined,
) => Boolean(
  profile.origin === 'local' &&
  currentGroup &&
  targetGroup &&
  profile.groupId !== targetGroup.id &&
  currentGroup.kind !== 'subscription' &&
  targetGroup.kind !== 'subscription',
)
