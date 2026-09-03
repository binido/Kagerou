export const pluralize = (
  count: number,
  singular: string,
  plural = `${singular}s`,
) => `${count} ${count === 1 ? singular : plural}`

export const formatVpnCount = (count: number) => pluralize(count, 'VPN', 'VPNs')
export const formatSourceCount = (count: number) => pluralize(count, 'source')

export const formatLogCount = (count: number, query: string) =>
  query.trim()
    ? `${count} ${count === 1 ? 'matching entry' : 'matching entries'}`
    : `${count} entries`

export const formatSourceTimestamp = (value: string) => value

export const deriveSubscriptionName = (value: string, fallbackNumber: number) => {
  try {
    const host = new URL(value).hostname.replace(/^www\./i, '')
    return host || `Subscription ${String(fallbackNumber).padStart(2, '0')}`
  } catch {
    return `Subscription ${String(fallbackNumber).padStart(2, '0')}`
  }
}
