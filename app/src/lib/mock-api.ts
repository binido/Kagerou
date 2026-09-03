import type { ProfileProtocol, SourceType, TestMethod, TestResult } from '@/types/kagerou'

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

export const mockApi = {
  async runProfileTest(profileId: string, method: TestMethod): Promise<TestResult> {
    await wait(720)
    return testFixtures[profileId]?.[method] ??
      (method === 'tcp'
        ? { value: '74 ms', tone: 'warn' }
        : { value: '200 OK', tone: 'good' })
  },

  async refreshSource(sourceId: string) {
    await wait(850)
    return sourceId
  },

  validateSource(type: SourceType, value: string) {
    if (type === 'url') {
      return /^https?:\/\/[^\s]+$/i.test(value)
        ? null
        : 'Enter a valid http(s) source URL.'
    }

    return /^(vless|vmess|trojan|ss|hysteria2):\/\/[^\s]+$/i.test(value)
      ? null
      : 'Enter a valid profile key such as vless://…'
  },

  protocolFromKey,
}
