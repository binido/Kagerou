import { openUrl } from '@tauri-apps/plugin-opener'
import { CircleArrowUp, ExternalLink } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import type { UpdateInfo } from '@/types/kagerou'

interface SidebarUpdateNoticeProps {
  collapsed: boolean
  update: UpdateInfo
}

export function SidebarUpdateNotice({ collapsed, update }: SidebarUpdateNoticeProps) {
  const { t } = useTranslation('common')
  const updateAvailable = t('sidebar.updateAvailable', { version: update.version })
  const viewReleases = t('sidebar.viewReleases')
  // A request to the host OS rather than navigation inside the document:
  // an anchor here opens nothing in a bundled app.
  const link = (
    <button
      aria-label={viewReleases}
      className={cn(
        'group flex min-h-10 items-center gap-2 rounded-lg border border-lavender/20 bg-lavender/10 px-2.5 py-2 text-[10px] leading-4 text-lavender-hi transition-colors hover:border-lavender/35 hover:bg-lavender/15 focus-visible:focus-ring',
        collapsed && 'size-10 justify-center border-transparent bg-transparent p-0',
        'max-[960px]:size-10 max-[960px]:justify-center max-[960px]:border-transparent max-[960px]:bg-transparent max-[960px]:p-0',
      )}
      onClick={() => { void openUrl(update.url) }}
      type="button"
    >
      <CircleArrowUp aria-hidden="true" className="size-4 shrink-0 text-lavender" strokeWidth={1.8} />
      <span className={cn('min-w-0 truncate', collapsed && 'hidden', 'max-[960px]:hidden')}>{updateAvailable}</span>
      <ExternalLink aria-hidden="true" className={cn('ml-auto size-3 shrink-0 text-lavender/70', collapsed && 'hidden', 'max-[960px]:hidden')} strokeWidth={1.8} />
    </button>
  )

  return (
    <Tooltip>
      <TooltipTrigger asChild>{link}</TooltipTrigger>
      <TooltipContent side="right">{viewReleases}</TooltipContent>
    </Tooltip>
  )
}
