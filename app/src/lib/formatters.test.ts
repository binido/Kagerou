import { describe, expect, it } from 'vitest'

import { regionToCountry } from './formatters'

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
