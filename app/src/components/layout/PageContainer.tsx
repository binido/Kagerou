import type { ReactNode } from 'react'

import { cn } from '@/lib/utils'

interface PageContainerProps {
  children: ReactNode
  className?: string
  contentClassName?: string
}

export function PageContainer({ children, className, contentClassName }: PageContainerProps) {
  return (
    <div className={cn('min-h-screen min-w-0 bg-canvas px-6 pb-10 pt-8 lg:px-12 lg:pt-10', className)}>
      <div className={cn('mx-auto w-full max-w-[1040px]', contentClassName)}>{children}</div>
    </div>
  )
}
