import { describe, expect, it } from 'vitest'

import { maskSubscriptionUrl, regionToCountry } from './formatters'

describe('regionToCountry', () => {
  it('maps a valid ISO code to a localized country with its flag', () => {
    expect(regionToCountry('AT', 'en')).toBe('🇦🇹 Austria')
    expect(regionToCountry('AT', 'ru')).toBe('🇦🇹 Австрия')
  })

  it('rejects non-codes', () => {
    expect(regionToCountry('', 'en')).toBeNull()
    expect(regionToCountry('Local profile', 'en')).toBeNull()
    expect(regionToCountry('at', 'en')).toBeNull()
    expect(regionToCountry('A', 'en')).toBeNull()
    expect(regionToCountry('ATL', 'en')).toBeNull()
  })
})

describe('maskSubscriptionUrl', () => {
  it('keeps the host and hides the path, query and fragment', () => {
    expect(maskSubscriptionUrl('https://example.com/sub/abc123?token=secret')).toBe('https://example.com/••••')
    expect(maskSubscriptionUrl('https://example.com/?token=secret')).toBe('https://example.com/••••')
    expect(maskSubscriptionUrl('https://example.com/#secret')).toBe('https://example.com/••••')
  })

  it('leaves a bare host alone — there is nothing secret to hide', () => {
    expect(maskSubscriptionUrl('https://example.com')).toBe('https://example.com')
    expect(maskSubscriptionUrl('https://example.com/')).toBe('https://example.com')
  })

  it('keeps a non-default port, which distinguishes two hosts', () => {
    expect(maskSubscriptionUrl('https://example.com:8443/sub')).toBe('https://example.com:8443/••••')
  })

  it('masks anything it cannot parse rather than leaking it', () => {
    expect(maskSubscriptionUrl('not a url')).toBe('••••••••••••••••')
  })
})
