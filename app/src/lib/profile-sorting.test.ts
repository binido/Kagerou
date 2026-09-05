import { describe, expect, it } from 'vitest'

import { sortProfiles } from './profile-sorting'
import type { Profile, TestResult } from '@/types/kagerou'

const untested: TestResult = { value: 'Not tested', tone: 'muted' }
const ms = (value: number): TestResult => ({ value: `${value} ms`, tone: 'good' })

const profile = (name: string, tcp: TestResult, url: TestResult): Profile => ({
  id: name, name, region: '', protocol: 'VLESS', origin: 'imported',
  groupId: 'g', selected: false, tcp, url, key: 'vless://x@h:443',
})

describe('sortProfiles by ping', () => {
  it('orders on the latency through the proxy, not the reachability ping', () => {
    const order = sortProfiles([
      profile('slow-path', ms(20), ms(400)),
      profile('fast-path', ms(200), ms(50)),
    ], 'ping').map((p) => p.name)

    expect(order).toEqual(['fast-path', 'slow-path'])
  })

  it('falls back to the ping for profiles the URL test has not reached', () => {
    const order = sortProfiles([
      profile('no-url-slow', ms(300), untested),
      profile('no-url-fast', ms(30), untested),
    ], 'ping').map((p) => p.name)

    expect(order).toEqual(['no-url-fast', 'no-url-slow'])
  })

  it('sinks profiles with no usable measurement to the bottom', () => {
    const order = sortProfiles([
      profile('dead', { value: 'No response', tone: 'bad' }, untested),
      profile('alive', untested, ms(100)),
    ], 'ping').map((p) => p.name)

    expect(order).toEqual(['alive', 'dead'])
  })
})
