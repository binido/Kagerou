export const formatSourceTimestamp = (value: string) => value

/** Bytes/sec (what the Clash API reports) → Mbit/s with one decimal,
 * matching the telemetry panel's fixed "Mbps" unit label. */
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
export const regionToCountry = (region: string, language: string): string | null => {
  if (!/^[A-Z]{2}$/.test(region)) return null
  const flag = String.fromCodePoint(...[...region].map((c) => 0x1f1e6 + c.charCodeAt(0) - 65))
  return `${flag} ${new Intl.DisplayNames([language], { type: 'region' }).of(region)}`
}
