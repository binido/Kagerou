import { describe, expect, it } from 'vitest'

import { isUnreachable, resultLabel } from './result-copy'
import type { TestOutcome } from '@/types/kagerou'

const translate = ((key: string, options?: Record<string, unknown>) =>
  options ? `${key}:${JSON.stringify(options)}` : key) as never

describe('resultLabel', () => {
  it('passes the measured number to the translation rather than formatting it here', () => {
    expect(resultLabel({ kind: 'latency', millis: 42 }, translate)).toBe(
      'status.latency:{"millis":42}',
    )
  })

  it('has a key for every outcome the backend can send', () => {
    const outcomes: TestOutcome[] = [
      { kind: 'notTested' },
      { kind: 'timeout' },
      { kind: 'noResponse' },
      { kind: 'unavailable' },
    ]

    expect(outcomes.map((outcome) => resultLabel(outcome, translate))).toEqual([
      'status.notTested',
      'status.timeout',
      'status.noResponse',
      'status.unavailable',
    ])
  })
})

describe('isUnreachable', () => {
  it('counts only the outcomes where nothing answered', () => {
    expect(isUnreachable({ kind: 'timeout' })).toBe(true)
    expect(isUnreachable({ kind: 'noResponse' })).toBe(true)
    expect(isUnreachable({ kind: 'unavailable' })).toBe(true)
  })

  it('leaves a slow server alone, however badly it is coloured', () => {
    expect(isUnreachable({ kind: 'latency', millis: 4000 })).toBe(false)
  })

  it('does not count a profile nobody has measured', () => {
    expect(isUnreachable({ kind: 'notTested' })).toBe(false)
  })
})
