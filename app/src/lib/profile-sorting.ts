import type { GroupSortMode, Profile, TestResult } from '@/types/kagerou'

const latencyPattern = /^(\d+)\s*ms$/i

export const getReachabilityAwarePing = (profile: Profile): TestResult => {
  if (profile.url.tone === 'good') return profile.tcp
  if (profile.url.tone === 'bad') return { value: 'Unavailable', tone: 'bad' }
  if (profile.url.tone === 'warn') return { value: 'Checking…', tone: 'warn' }
  return { value: 'Not tested', tone: 'muted' }
}

const pingValue = (profile: Profile) => {
  const result = getReachabilityAwarePing(profile)
  const match = result.value.match(latencyPattern)
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

export const groupSortLabels: Record<GroupSortMode, string> = {
  ping: 'Ping',
  name: 'Name',
  protocol: 'Protocol',
}
