import { describe, expect, it } from 'vitest'

import { highlightPattern } from '@/lib/log-search'

describe('highlightPattern', () => {
  it('returns null for an empty or blank query', () => {
    expect(highlightPattern('')).toBeNull()
    expect(highlightPattern('   ')).toBeNull()
  })

  it('builds a pattern for every metacharacter instead of throwing', () => {
    for (const query of ['(', ')', '[', ']', '{', '}', '.', '*', '+', '?', '^', '$', '|', '\\']) {
      expect(() => highlightPattern(query)).not.toThrow()
      expect('a'.split(highlightPattern(query) as RegExp)).toEqual(['a'])
    }
  })

  it('matches a metacharacter literally rather than as a pattern', () => {
    expect('sing-box (1.14.0) started'.split(highlightPattern('(1.14.0)') as RegExp)).toEqual([
      'sing-box ',
      '(1.14.0)',
      ' started',
    ])
  })

  it('does not let a quantifier through as a quantifier', () => {
    expect('aaa'.split(highlightPattern('a+') as RegExp)).toEqual(['aaa'])
    expect('a+b'.split(highlightPattern('a+') as RegExp)).toEqual(['', 'a+', 'b'])
  })

  it('ignores case, the way the list filter does', () => {
    expect('ERROR here'.split(highlightPattern('error') as RegExp)).toEqual(['', 'ERROR', ' here'])
  })
})
