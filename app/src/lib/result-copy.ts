import type { TFunction } from 'i18next'

const resultTranslationKeys = {
  '200 OK': 'status.ok',
  'Checking…': 'status.checking',
  'No response': 'status.noResponse',
  'Not connected': 'status.notConnected',
  'Not tested': 'status.notTested',
  'Running…': 'status.running',
  'Testing…': 'status.checking',
  Timeout: 'status.timeout',
  Unavailable: 'status.unavailable',
} as const

export const localizeResultValue = (value: string, translate: TFunction<'common'>) => {
  const key = resultTranslationKeys[value as keyof typeof resultTranslationKeys]
  return key ? translate(key) : value
}
