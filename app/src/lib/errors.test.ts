import { describe, expect, it, vi } from 'vitest'

import { asBackendError, backendErrorMessage } from './errors'

describe('asBackendError', () => {
  it('recognises what a command rejects with', () => {
    expect(asBackendError({ code: 'network', detail: '502' })).toEqual({ code: 'network', detail: '502' })
  })

  it('rejects anything that is not one', () => {
    expect(asBackendError(new Error('boom'))).toBeNull()
    expect(asBackendError('boom')).toBeNull()
    expect(asBackendError(undefined)).toBeNull()
    expect(asBackendError({ message: 'nope' })).toBeNull()
  })
})

describe('backendErrorMessage', () => {
  it('translates by the code rather than showing the backend sentence', () => {
    vi.spyOn(console, 'error').mockImplementation(() => {})

    const message = backendErrorMessage({ code: 'activeProfileInUse', detail: 'switch away first' }, 'fallback')

    expect(message).not.toBe('switch away first')
    expect(message).not.toBe('fallback')
  })

  it('keeps the backend wording out of the interface but not out of the console', () => {
    const logged = vi.spyOn(console, 'error').mockImplementation(() => {})

    backendErrorMessage({ code: 'network', detail: 'https://secret.example answered 502' }, 'fallback')

    expect(logged).toHaveBeenCalledWith('network: https://secret.example answered 502')
  })

  it('falls back when the rejection did not come from a command', () => {
    expect(backendErrorMessage(new Error('boom'), 'fallback')).toBe('fallback')
    expect(backendErrorMessage(undefined, 'fallback')).toBe('fallback')
  })
})
