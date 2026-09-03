import type { ReactNode } from 'react'

interface PageHeaderProps {
  eyebrow?: string
  title: string
  description?: string
  actions?: ReactNode
  status?: ReactNode
}

export function PageHeader({ eyebrow, title, description, actions, status }: PageHeaderProps) {
  return (
    <header className="flex items-start justify-between gap-6 max-[640px]:flex-col">
      <div className="min-w-0">
        {eyebrow ? <p className="type-eyebrow">{eyebrow}</p> : null}
        <h1 className={eyebrow ? 'type-display mt-3 text-[34px] leading-none text-primary' : 'type-display text-[34px] leading-none text-primary'}>{title}</h1>
        {description ? <p className="type-body mt-3 max-w-[610px] text-muted-copy">{description}</p> : null}
      </div>
      {(actions || status) && (
        <div className="flex shrink-0 flex-col items-end gap-3 pt-1 max-[640px]:w-full max-[640px]:items-start">
          {actions}
          {status}
        </div>
      )}
    </header>
  )
}
