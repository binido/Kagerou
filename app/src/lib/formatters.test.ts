import { describe, expect, it } from 'vitest'

import { formatExitLocation, formatLogTimestamp, formatRelativeTime, formatUptime, regionToCountry } from './formatters'

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

describe('formatLogTimestamp', () => {
  it('reduces the stored ISO timestamp to the clock time the column has room for', () => {
    // Asserted by shape, not by value: the format follows the machine's own
    // time zone, and a fixed expectation would only pass where it was written.
    expect(formatLogTimestamp('2026-09-08T17:52:46.522Z')).toMatch(/^\d{2}:\d{2}:\d{2}$/)
  })

  it('pads a single-digit hour so the column never jumps by a character', () => {
    expect(formatLogTimestamp('2026-09-08T04:05:06.000Z')).toHaveLength(8)
  })
})

describe('formatRelativeTime', () => {
  const ago = (millis: number) => String(Date.now() - millis)

  it('reads as now for a source refreshed seconds ago', () => {
    expect(formatRelativeTime(ago(5_000), 'en')).toBe('now')
    expect(formatRelativeTime(ago(5_000), 'ru')).toBe('сейчас')
  })

  it('picks the largest unit the elapsed time reaches', () => {
    expect(formatRelativeTime(ago(5 * 60_000), 'en')).toBe('5 minutes ago')
    expect(formatRelativeTime(ago(3 * 3_600_000), 'en')).toBe('3 hours ago')
    expect(formatRelativeTime(ago(8 * 86_400_000), 'en')).toBe('8 days ago')
  })

  it('translates, which the stored English phrase could never do', () => {
    expect(formatRelativeTime(ago(7 * 86_400_000), 'ru')).toBe('7 дней назад')
  })

  it('is empty for a source that has never been refreshed', () => {
    expect(formatRelativeTime('', 'en')).toBe('')
    expect(formatRelativeTime('   ', 'en')).toBe('')
  })

  it('is empty for the prose the column used to hold', () => {
    expect(formatRelativeTime('Updated just now', 'en')).toBe('')
  })

  it('reads as now rather than as a countdown when the clock went backwards', () => {
    expect(formatRelativeTime(String(Date.now() + 60_000), 'en')).toBe('now')
  })
})
