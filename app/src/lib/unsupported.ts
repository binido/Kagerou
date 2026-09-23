import i18n from '@/i18n'
import type { Unsupported } from '@/types/kagerou'

const label = (entry: Unsupported): string =>
  entry.kind === 'invalid' ? i18n.t('profiles:import.unreadableEntry') : entry.name

/** One line naming what an import left out and how many of each, or
 * `undefined` when it left out nothing. */
export const describeUnsupported = (unsupported: Unsupported[]): string | undefined => {
  if (unsupported.length === 0) return undefined
  const counts = new Map<string, number>()
  for (const entry of unsupported) counts.set(label(entry), (counts.get(label(entry)) ?? 0) + 1)
  const list = [...counts].map(([name, count]) => `${name} (${count})`).join(', ')
  return i18n.t('profiles:import.unsupported', { count: unsupported.length, list })
}
