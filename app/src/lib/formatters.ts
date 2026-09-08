import type { ExitLocation } from '@/types/kagerou'

export const formatSourceTimestamp = (value: string) => value

/** Bytes/sec (what the Clash API reports) → Mbit/s with one decimal,
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

export const deriveSubscriptionName = (value: string, fallbackNumber: number, fallbackLabel: string) => {
  try {
    const host = new URL(value).hostname.replace(/^www\./i, '')
    return host || `${fallbackLabel} ${String(fallbackNumber).padStart(2, '0')}`
  } catch {
    return `${fallbackLabel} ${String(fallbackNumber).padStart(2, '0')}`
  }
}

/** An ISO 3166-1 alpha-2 region code (what the backend's region_from_name
 * emits) → "🇦🇹 Austria"-style display string, localized via Intl.
 * Anything else ("", "Local profile", garbage) → null. */
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

const MASK = '••••'

/** Hides the secret half of a subscription URL while keeping the host, so
 * two sources stay distinguishable at a glance. The token lives in the path
 * or query, never in the hostname. Anything unparseable is masked whole. */
export const maskSubscriptionUrl = (value: string): string => {
  try {
    const url = new URL(value)
    const hasSecret = url.pathname.replace(/^\/+$/, '') !== '' || url.search !== '' || url.hash !== ''
    return hasSecret ? `${url.protocol}//${url.host}/${MASK}` : `${url.protocol}//${url.host}`
  } catch {
    return MASK.repeat(4)
  }
}

/** Elapsed milliseconds → "h:mm:ss" past the first hour, "m:ss" before it.
 * Anything negative (a clock nudged backwards mid-session) reads as zero
 * rather than as a countdown. */
export const formatUptime = (elapsedMs: number): string => {
  const total = Math.max(0, Math.floor(elapsedMs / 1000))
  const seconds = String(total % 60).padStart(2, '0')
  const minutes = Math.floor(total / 60) % 60
  const hours = Math.floor(total / 3600)
  return hours > 0 ? `${hours}:${String(minutes).padStart(2, '0')}:${seconds}` : `${minutes}:${seconds}`
}
