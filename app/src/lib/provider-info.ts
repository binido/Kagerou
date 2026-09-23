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

const startOfDay = (millis: number): number => new Date(millis).setHours(0, 0, 0, 0)

/** Calendar days from today to the end date, so it agrees with the date shown
 * beside it. Negative once it has passed. Rounded because a day that crosses a
 * DST change is an hour off 24. */
export const daysUntil = (expiresAt: number, now: number): number =>
  Math.round((startOfDay(expiresAt) - startOfDay(now)) / DAY_MS)
