import { initialProfiles } from '@/lib/mock-data'
import type { Profile, ProfileDraft, ProfileProtocol, Source, SourceType, TestMethod, TestResult } from '@/types/kagerou'

const wait = (duration: number) =>
  new Promise<void>((resolve) => window.setTimeout(resolve, duration))

const protocolFromKey = (key: string): ProfileProtocol => {
  const scheme = key.trim().split('://')[0]?.toLowerCase()
  if (scheme === 'vmess') return 'VMess'
  if (scheme === 'trojan') return 'Trojan'
  if (scheme === 'ss') return 'Shadowsocks'
  if (scheme === 'hysteria2') return 'Hysteria2'
  return 'VLESS'
}

const profileToDraft = (profile: Profile): ProfileDraft => ({
  name: profile.name,
  region: profile.region,
  protocol: profile.protocol,
  origin: profile.origin,
  sourceId: profile.sourceId,
  tcp: profile.tcp,
  url: profile.url,
  key: profile.key,
})

const sourceSlugFromUrl = (value: string) => {
  try {
    const pathname = new URL(value).pathname.split('/').filter(Boolean)
    return pathname.at(-1) ?? ''
  } catch {
    return ''
  }
}

const genericProfile = (host: string, index: number): ProfileDraft => ({
  name: `${host} · Edge ${String(index).padStart(2, '0')}`,
  region: 'remote',
  protocol: index % 2 === 0 ? 'VLESS' : 'Trojan',
  origin: 'imported',
  tcp: { value: index === 1 ? '62 ms' : '78 ms', tone: index === 1 ? 'good' : 'warn' },
  url: { value: '200 OK', tone: 'good' },
  key: `vless://${host}-edge-${index}`,
})

const testFixtures: Record<string, Record<TestMethod, TestResult>> = {
  'p-seattle': {
    tcp: { value: '42 ms', tone: 'good' },
    url: { value: '200 OK', tone: 'good' },
  },
  'p-vancouver': {
    tcp: { value: '68 ms', tone: 'warn' },
    url: { value: '200 OK', tone: 'good' },
  },
  'p-new-york': {
    tcp: { value: '71 ms', tone: 'warn' },
    url: { value: '200 OK', tone: 'good' },
  },
  'p-chicago': {
    tcp: { value: 'No response', tone: 'bad' },
    url: { value: 'Timeout', tone: 'bad' },
  },
}

export interface ImportedSubscription {
  profiles: ProfileDraft[]
}

export const mockApi = {
  async runProfileTest(profileId: string, method: TestMethod): Promise<TestResult> {
    await wait(720)
    return testFixtures[profileId]?.[method] ??
      (method === 'tcp'
        ? { value: '74 ms', tone: 'warn' }
        : { value: '200 OK', tone: 'good' })
  },

  async importSubscription(url: string): Promise<ImportedSubscription> {
    await wait(850)
    const sourceSlug = sourceSlugFromUrl(url)
    const knownProfiles = initialProfiles.filter((profile) => profile.origin === 'imported' && profile.sourceId === sourceSlug)
    if (knownProfiles.length > 0) {
      return { profiles: knownProfiles.map(profileToDraft) }
    }

    let host = 'subscription'
    try {
      host = new URL(url).hostname.replace(/^www\./i, '') || host
    } catch {
      // Source URL validation happens before the import call.
    }

    return {
      profiles: [1, 2, 3].map((index) => genericProfile(host, index)),
    }
  },

  async refreshSource(source: Pick<Source, 'type' | 'value'>): Promise<ImportedSubscription> {
    if (source.type === 'url') return this.importSubscription(source.value)
    await wait(450)
    return { profiles: [] }
  },

  validateSource(type: SourceType, value: string) {
    if (type === 'url') {
      return /^https?:\/\/[^\s]+$/i.test(value)
        ? null
        : 'Enter a valid http(s) source URL.'
    }

    return /^(vless|vmess|trojan|ss|hysteria2):\/\/[^\s]+$/i.test(value)
      ? null
      : 'Enter a valid VPN key such as vless://…'
  },

  protocolFromKey,
}
