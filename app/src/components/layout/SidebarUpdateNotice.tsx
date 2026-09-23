import { openUrl } from '@tauri-apps/plugin-opener'
import { CircleArrowUp, Download, ExternalLink, LoaderCircle, RotateCw } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import type { UpdateDownload, UpdateInfo } from '@/types/kagerou'

interface SidebarUpdateNoticeProps {
  collapsed: boolean
  update: UpdateInfo
  download: UpdateDownload
  onDownload: () => void
  onInstall: () => void
}

const downloadPercent = (download: UpdateDownload): number | null =>
  download.phase === 'downloading' && download.total
    ? Math.min(100, Math.round((download.downloaded / download.total) * 100))
    : null

export function SidebarUpdateNotice({
  collapsed,
  update,
  download,
  onDownload,
  onInstall,
}: SidebarUpdateNoticeProps) {
  const { t } = useTranslation('common')
  const updateAvailable = t('sidebar.updateAvailable', { version: update.version })
  const percent = downloadPercent(download)

  let label = updateAvailable
  let hint = t('sidebar.viewReleases')
  let LeadingIcon = CircleArrowUp
  let TrailingIcon: typeof ExternalLink | null = ExternalLink
  // A request to the host OS rather than navigation inside the document:
  // an anchor here opens nothing in a bundled app.
  let action: (() => void) | null = () => {
    void openUrl(update.url)
  }

  if (update.installable) {
    switch (download.phase) {
      case 'idle':
        hint = t('sidebar.updateInstall', { version: update.version })
        TrailingIcon = Download
        action = onDownload
        break
      case 'downloading':
        label =
          percent === null
            ? t('sidebar.updateDownloadingUnknown')
            : t('sidebar.updateDownloading', { percent })
        hint = label
        LeadingIcon = LoaderCircle
        TrailingIcon = null
        action = null
        break
      case 'ready':
        label = t('sidebar.updateReady')
        hint = t('sidebar.updateReadyHint', { version: update.version })
        LeadingIcon = RotateCw
        TrailingIcon = null
        action = onInstall
        break
      case 'installing':
        label = t('sidebar.updateInstalling')
        hint = label
        LeadingIcon = LoaderCircle
        TrailingIcon = null
        action = null
        break
    }
  }

  const busy = action === null
  const button = (
    <button
      aria-disabled={busy || undefined}
      aria-label={hint}
      className={cn(
        'group relative flex min-h-10 items-center gap-2 overflow-hidden rounded-lg border border-lavender/20 bg-lavender/10 px-2.5 py-2 text-[10px] leading-4 text-lavender-hi transition-colors hover:border-lavender/35 hover:bg-lavender/15 focus-visible:focus-ring',
        busy && 'cursor-default hover:border-lavender/20 hover:bg-lavender/10',
        collapsed && 'size-10 justify-center border-transparent bg-transparent p-0',
        'max-[960px]:size-10 max-[960px]:justify-center max-[960px]:border-transparent max-[960px]:bg-transparent max-[960px]:p-0',
      )}
      onClick={() => action?.()}
      type="button"
    >
      <LeadingIcon
        aria-hidden="true"
        className={cn('size-4 shrink-0 text-lavender', busy && 'animate-spin')}
        strokeWidth={1.8}
      />
      <span className={cn('min-w-0 truncate', collapsed && 'hidden', 'max-[960px]:hidden')}>
        {label}
      </span>
      {TrailingIcon ? (
        <TrailingIcon
          aria-hidden="true"
          className={cn(
            'ml-auto size-3 shrink-0 text-lavender/70',
            collapsed && 'hidden',
            'max-[960px]:hidden',
          )}
          strokeWidth={1.8}
        />
      ) : null}
      {download.phase === 'downloading' && update.installable ? (
        <span
          aria-label={label}
          aria-valuemax={100}
          aria-valuemin={0}
          aria-valuenow={percent ?? undefined}
          className="absolute inset-x-0 bottom-0 h-0.5 bg-lavender/15"
          role="progressbar"
        >
          <span
            className="block h-full bg-lavender transition-[width] duration-200"
            style={{ width: `${percent ?? 0}%` }}
          />
        </span>
      ) : null}
    </button>
  )

  return (
    <Tooltip>
      <TooltipTrigger asChild>{button}</TooltipTrigger>
      <TooltipContent side="right">{hint}</TooltipContent>
    </Tooltip>
  )
}
