import { describe, expect, it } from 'vitest'

import { backendErrorMessage } from './errors'
import { formatExitLocation, formatUptime, regionToCountry } from './formatters'

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

describe('formatUptime', () => {
  it('drops the hour field until there is one', () => {
    expect(formatUptime(0)).toBe('0:00')
    expect(formatUptime(9_000)).toBe('0:09')
    expect(formatUptime(65_000)).toBe('1:05')
    expect(formatUptime(59 * 60_000 + 59_000)).toBe('59:59')
  })

  it('rolls into hours and restarts the minute field', () => {
    expect(formatUptime(3_600_000)).toBe('1:00:00')
    expect(formatUptime(3_600_000 + 61_000)).toBe('1:01:01')
    expect(formatUptime(26 * 3_600_000)).toBe('26:00:00')
  })

  it('reads a backwards clock as zero rather than as a countdown', () => {
    expect(formatUptime(-5_000)).toBe('0:00')
  })
})

describe('formatExitLocation', () => {
  const exit = { ip: '81.2.69.142', city: 'London', country: 'United Kingdom', countryCode: 'GB' }

  it('reads as flag, city and a country named in the app language', () => {
    expect(formatExitLocation(exit, 'en')).toBe('🇬🇧 London, United Kingdom')
    expect(formatExitLocation(exit, 'ru')).toBe('🇬🇧 London, Великобритания')
  })

  it('a lookup without a city still located the exit', () => {
    expect(formatExitLocation({ ...exit, city: '' }, 'en')).toBe('🇬🇧 United Kingdom')
  })

  it('falls back to the service’s own country name when the code is unusable', () => {
    expect(formatExitLocation({ ...exit, countryCode: '' }, 'en')).toBe('London, United Kingdom')
  })
})
