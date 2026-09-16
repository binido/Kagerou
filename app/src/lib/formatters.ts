import type { ExitLocation } from '@/types/kagerou'

/** Bytes/sec (what the Clash API reports) -> Mbit/s with one decimal,
 * matching the dashboard readout's fixed "Mbps" unit label. */
export const formatSpeedMbps = (bytesPerSecond: number): string => ((bytesPerSecond * 8) / 1_000_000).toFixed(1)

/** Splits a byte count into a value + unit pair so the UI can style the
 * unit separately from the number. */
export const formatBytes = (bytes: number): { value: string; unit: string } => {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'] as const
  let value = bytes
  let unitIndex = 0
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024
    unitIndex += 1
  }
  return { value: unitIndex === 0 ? String(value) : value.toFixed(2), unit: units[unitIndex] }
}

/** An ISO 3166-1 alpha-2 region code (what the backend's region_from_name
 * emits) -> "🇦🇹 Austria"-style display string, localized via Intl.
 * Anything else ("", "Local profile", garbage) -> null. */
export const regionToFlag = (region: string): string | null =>
  /^[A-Z]{2}$/.test(region)
    ? String.fromCodePoint(...[...region].map((c) => 0x1f1e6 + c.charCodeAt(0) - 65))
    : null

export const regionToCountry = (region: string, language: string): string | null => {
  const flag = regionToFlag(region)
  if (!flag) return null
  return `${flag} ${new Intl.DisplayNames([language], { type: 'region' }).of(region)}`
}

/** "🇬🇧 London, United Kingdom" from a looked-up exit. The country name comes
 * from `Intl` so it follows the app's language; the city is whatever the
 * lookup service said, since it only speaks English. A lookup with no city
 * still located the exit, and reads as the country alone. */
export const formatExitLocation = (exit: ExitLocation, language: string): string => {
  const flag = regionToFlag(exit.countryCode)
  const country = flag
    ? (new Intl.DisplayNames([language], { type: 'region' }).of(exit.countryCode) ?? exit.country)
    : exit.country
  const place = exit.city ? `${exit.city}, ${country}` : country
  return flag ? `${flag} ${place}` : place
}

/** Elapsed milliseconds -> "h:mm:ss" past the first hour, "m:ss" before it.
 * Anything negative (a clock nudged backwards mid-session) reads as zero
 * rather than as a countdown. */
export const formatUptime = (elapsedMs: number): string => {
  const total = Math.max(0, Math.floor(elapsedMs / 1000))
  const seconds = String(total % 60).padStart(2, '0')
  const minutes = Math.floor(total / 60) % 60
  const hours = Math.floor(total / 3600)
  return hours > 0 ? `${hours}:${String(minutes).padStart(2, '0')}:${seconds}` : `${minutes}:${seconds}`
}

const logTime = new Intl.DateTimeFormat(undefined, { hour: '2-digit', hour12: false, minute: '2-digit', second: '2-digit' })

/** The ISO timestamp the store keeps to the `HH:MM:SS` the log column shows.
 * The entry keeps the ISO form: it is the sortable one, and it is what a copy
 * of the log should carry. */
export const formatLogTimestamp = (iso: string): string => logTime.format(new Date(iso))

/** Largest unit first; the first one the elapsed time reaches is the one shown. */
const RELATIVE_UNITS: readonly (readonly [Intl.RelativeTimeFormatUnit, number])[] = [
  ['year', 31_536_000_000],
  ['month', 2_592_000_000],
  ['day', 86_400_000],
  ['hour', 3_600_000],
  ['minute', 60_000],
]

/** The unix milliseconds the sources table stores to "5 minutes ago" in the
 * user's language, or empty for a source that has never been refreshed. A
 * stamp in the future - a clock nudged backwards between two refreshes - reads
 * as "now" rather than as a countdown, the same way `formatUptime` clamps. */
export const formatRelativeTime = (millis: string, language: string): string => {
  const stamp = Number(millis)
  if (!millis.trim() || !Number.isFinite(stamp)) return ''
  const elapsed = Math.max(0, Date.now() - stamp)
  const relative = new Intl.RelativeTimeFormat(language, { numeric: 'auto' })
  const unit = RELATIVE_UNITS.find(([, size]) => elapsed >= size)
  return unit ? relative.format(-Math.round(elapsed / unit[1]), unit[0]) : relative.format(0, 'second')
}
