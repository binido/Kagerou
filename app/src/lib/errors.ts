import i18n from '@/i18n'

/** Mirrors `usecase::error::ErrorCode`. */
export type ErrorCode =
  | 'storage'
  | 'notFound'
  | 'invalidInput'
  | 'configInvalid'
  | 'coreFailed'
  | 'network'
  | 'subscriptionInvalid'
  | 'notASubscriptionUrl'
  | 'notASubscription'
  | 'activeProfileInUse'
  | 'testRunInProgress'
  | 'lookupFailed'
  | 'systemSetting'

export interface BackendError {
  code: ErrorCode
  detail: string
}

/** Tauri rejects `invoke` with whatever the command's `Err` serialises to,
 * which for every command here is this shape rather than an `Error`. */
export const asBackendError = (error: unknown): BackendError | null =>
  typeof error === 'object' && error !== null && 'code' in error && 'detail' in error
    ? (error as BackendError)
    : null

/** What the user is told when an action fails.
 *
 * The backend's code picks the sentence, so it arrives in their language;
 * `fallback` names what was being attempted and covers a rejection that did
 * not come from a command. The backend's own English wording goes to the
 * console - it can name a host or a path, which is not something to put in
 * front of someone. */
export const backendErrorMessage = (error: unknown, fallback: string): string => {
  const backend = asBackendError(error)
  if (!backend) return fallback
  console.error(`${backend.code}: ${backend.detail}`)
  return i18n.t(`common:errors.${backend.code}` as never, { defaultValue: fallback })
}
