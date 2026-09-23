import { describe, expect, it } from 'vitest'

import { describeUnsupported } from './unsupported'

describe('describeUnsupported', () => {
  it('says nothing when nothing was left out', () => {
    expect(describeUnsupported([])).toBeUndefined()
  })

  it('counts each reason once, in the order it first appeared', () => {
    const line = describeUnsupported([
      { kind: 'transport', name: 'xhttp' },
      { kind: 'protocol', name: 'ssr' },
      { kind: 'transport', name: 'xhttp' },
    ])
    expect(line).toContain('3')
    expect(line).toContain('xhttp (2), ssr (1)')
  })

  it('names reasons that carry no name of their own', () => {
    const line = describeUnsupported([{ kind: 'balancer' }, { kind: 'chain' }, { kind: 'invalid' }])
    expect(line).not.toContain('undefined')
    expect(line).toContain('(1), ')
  })
})
