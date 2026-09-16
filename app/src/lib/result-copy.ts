import type { TFunction } from 'i18next'

import type { TestOutcome } from '@/types/kagerou'

/** What a measurement reads as in the user's language. The backend sends a
 * kind, not a sentence, so nothing here matches on English text. */
export const resultLabel = (outcome: TestOutcome, translate: TFunction<'common'>) => {
  switch (outcome.kind) {
    case 'latency':
      return translate('status.latency', { millis: outcome.millis })
    case 'timeout':
      return translate('status.timeout')
    case 'noResponse':
      return translate('status.noResponse')
    case 'unavailable':
      return translate('status.unavailable')
    case 'notTested':
      return translate('status.notTested')
  }
}

/** Whether the profile failed to answer at all. A slow server answered, so
 * it is not one of these - "remove unavailable" must not take it. */
export const isUnreachable = (outcome: TestOutcome) =>
  outcome.kind === 'timeout' || outcome.kind === 'noResponse' || outcome.kind === 'unavailable'
