import { describe, expect, it } from 'vitest'

import { daysUntil, expiryTone, hasProviderInfo, usageTone } from './provider-info'

const DAY = 86_400_000
const now = Date.UTC(2026, 8, 23)

const nothing = {
  trafficUsed: null,
  trafficTotal: null,
  expiresAt: null,
  announce: null,
  supportUrl: null,
}

describe('provider info', () => {
  it('counts any single field as something to show', () => {
    expect(hasProviderInfo(nothing)).toBe(false)
    expect(hasProviderInfo({ ...nothing, trafficUsed: 0 })).toBe(true)
    expect(hasProviderInfo({ ...nothing, supportUrl: 'https://x.example' })).toBe(true)
  })

  it('warns from 90% of the limit and turns bad at the limit', () => {
    expect(usageTone(89, 100)).toBe('normal')
    expect(usageTone(90, 100)).toBe('warn')
    expect(usageTone(100, 100)).toBe('bad')
    expect(usageTone(150, 100)).toBe('bad')
  })

  it('warns in the last three days and turns bad once expired', () => {
    expect(expiryTone(now + 4 * DAY, now)).toBe('normal')
    expect(expiryTone(now + 2 * DAY, now)).toBe('warn')
    expect(expiryTone(now, now)).toBe('bad')
    expect(expiryTone(now - DAY, now)).toBe('bad')
  })

  it('counts whole days towards now', () => {
    expect(daysUntil(now + 2.9 * DAY, now)).toBe(2)
    expect(daysUntil(now + 0.5 * DAY, now)).toBe(0)
    expect(daysUntil(now - 1.5 * DAY, now)).toBe(-1)
  })
})
