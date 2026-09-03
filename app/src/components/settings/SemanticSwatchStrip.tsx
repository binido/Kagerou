import type { CSSProperties } from 'react'

import type { Theme } from '@/themes/types'

const swatches = [
  { key: 'canvas', width: '18px' },
  { key: 'surface', width: '14px' },
  { key: 'text', width: '6px' },
  { key: 'accent', width: '10px' },
  { key: 'success', width: '8px' },
  { key: 'warning', width: '8px' },
  { key: 'error', width: '8px' },
] as const

export function SemanticSwatchStrip({ theme }: { theme: Theme }) {
  return (
    <span aria-hidden="true" className="flex h-[18px] w-[72px] shrink-0 overflow-hidden rounded-[4px]">
      {swatches.map(({ key, width }) => (
        <span className="h-full" key={key} style={{ backgroundColor: theme.tokens[key], width } satisfies CSSProperties} />
      ))}
    </span>
  )
}
