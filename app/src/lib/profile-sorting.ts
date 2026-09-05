import type { GroupSortMode, Profile } from '@/types/kagerou'

const latencyPattern = /^(\d+)\s*ms$/i

/** Untested and failed profiles sort last: neither yields a latency. */
const pingValue = (profile: Profile) => {
  const match = profile.url.value.match(latencyPattern)
  return match ? Number(match[1]) : Number.POSITIVE_INFINITY
}

export const sortProfiles = (profiles: Profile[], mode: GroupSortMode) => profiles
  .map((profile, index) => ({ profile, index }))
  .sort((left, right) => {
    if (mode === 'name') {
      return left.profile.name.localeCompare(right.profile.name, undefined, { sensitivity: 'base' }) || left.index - right.index
    }
    if (mode === 'protocol') {
      return left.profile.protocol.localeCompare(right.profile.protocol, undefined, { sensitivity: 'base' }) || left.profile.name.localeCompare(right.profile.name, undefined, { sensitivity: 'base' }) || left.index - right.index
    }
    return pingValue(left.profile) - pingValue(right.profile) || left.profile.name.localeCompare(right.profile.name, undefined, { sensitivity: 'base' }) || left.index - right.index
  })
  .map(({ profile }) => profile)
