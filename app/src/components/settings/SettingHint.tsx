import { Info } from 'lucide-react'

import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'

interface SettingHintProps {
  id?: string
  description: string
}

export function SettingHint({ id, description }: SettingHintProps) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          className="inline-flex size-5 shrink-0 items-center justify-center rounded-full text-muted-copy hover:text-body focus-visible:focus-ring"
          type="button"
        >
          <Info aria-hidden="true" className="size-3.5" strokeWidth={1.8} />
          <span className="sr-only" id={id}>
            {description}
          </span>
        </button>
      </TooltipTrigger>
      <TooltipContent className="leading-4" side="top">
        {description}
      </TooltipContent>
    </Tooltip>
  )
}
