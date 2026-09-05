import { describe, expect, it } from 'vitest'

import { backendErrorMessage } from './errors'
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

describe('backendErrorMessage', () => {
  it('keeps the string Tauri rejects with — that is where the reason lives', () => {
    expect(backendErrorMessage('HTTP status server error (502 Bad Gateway)', 'fallback'))
      .toBe('HTTP status server error (502 Bad Gateway)')
  })

  it('still reads a real Error', () => {
    expect(backendErrorMessage(new Error('boom'), 'fallback')).toBe('boom')
  })

  it('falls back on anything empty or unrecognised', () => {
    expect(backendErrorMessage('   ', 'fallback')).toBe('fallback')
    expect(backendErrorMessage(new Error(''), 'fallback')).toBe('fallback')
    expect(backendErrorMessage(undefined, 'fallback')).toBe('fallback')
    expect(backendErrorMessage({ message: 'nope' }, 'fallback')).toBe('fallback')
  })
})
