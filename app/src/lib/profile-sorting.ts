import type { GroupSortMode, Profile, TestResult } from '@/types/kagerou'

const latencyPattern = /^(\d+)\s*ms$/i

const latencyOf = (result: TestResult) => {
  const match = result.value.match(latencyPattern)
  return match ? Number(match[1]) : Number.POSITIVE_INFINITY
}

/** Sorts on the latency through the proxy, since that is what the connection
 * will actually feel like, and falls back to the reachability ping for
 * profiles the URL test has not reached yet. */
const pingValue = (profile: Profile) => {
  const url = latencyOf(profile.url)
  return Number.isFinite(url) ? url : latencyOf(profile.tcp)
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
