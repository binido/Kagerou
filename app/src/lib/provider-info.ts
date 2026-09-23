import type { ProviderInfo } from '@/types/kagerou'

export type ProviderTone = 'normal' | 'warn' | 'bad'

const DAY_MS = 86_400_000
/** How close to the end a subscription starts to warn. */
const EXPIRY_WARNING_MS = 3 * DAY_MS
const USAGE_WARNING_RATIO = 0.9

export const hasProviderInfo = (info: ProviderInfo): boolean =>
  Object.values(info).some((value) => value !== null)

export const usageTone = (used: number, total: number): ProviderTone =>
  used >= total ? 'bad' : used / total >= USAGE_WARNING_RATIO ? 'warn' : 'normal'

export const expiryTone = (expiresAt: number, now: number): ProviderTone =>
  expiresAt <= now ? 'bad' : expiresAt - now < EXPIRY_WARNING_MS ? 'warn' : 'normal'

/** Whole days to the end date, rounded towards now. Negative once it has passed. */
export const daysUntil = (expiresAt: number, now: number): number =>
  Math.trunc((expiresAt - now) / DAY_MS)
