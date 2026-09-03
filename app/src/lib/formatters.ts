export const formatSourceTimestamp = (value: string) => value

export const deriveSubscriptionName = (value: string, fallbackNumber: number, fallbackLabel: string) => {
  try {
    const host = new URL(value).hostname.replace(/^www\./i, '')
    return host || `${fallbackLabel} ${String(fallbackNumber).padStart(2, '0')}`
  } catch {
    return `${fallbackLabel} ${String(fallbackNumber).padStart(2, '0')}`
  }
}
