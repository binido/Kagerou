import { describe, expect, it } from 'vitest'

import { sortProfiles } from './profile-sorting'
import type { Profile, TestResult } from '@/types/kagerou'

const untested: TestResult = { value: 'Not tested', tone: 'muted' }
const ms = (value: number): TestResult => ({ value: `${value} ms`, tone: 'good' })

const profile = (name: string, url: TestResult): Profile => ({
  id: name, name, region: '', protocol: 'VLESS', origin: 'imported',
  groupId: 'g', selected: false, url, key: 'vless://x@h:443',
})

describe('sortProfiles by ping', () => {
  it('orders on the measured latency', () => {
    const order = sortProfiles([profile('slow', ms(400)), profile('fast', ms(50))], 'ping')
      .map((p) => p.name)

    expect(order).toEqual(['fast', 'slow'])
  })

  it('sinks untested and failed profiles below every measured one', () => {
    const order = sortProfiles([
      profile('untested', untested),
      profile('failed', { value: 'Timeout', tone: 'bad' }),
      profile('measured', ms(300)),
    ], 'ping').map((p) => p.name)

    expect(order[0]).toBe('measured')
  })
})
