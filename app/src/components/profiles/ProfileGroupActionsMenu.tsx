import { Eraser, LockKeyhole, MoreHorizontal, Pencil, Plug, Trash2, Waypoints, Wifi } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import type { ProfileGroup, TestMethod } from '@/types/kagerou'

interface ProfileGroupActionsMenuProps {
  group: ProfileGroup
  testRunning: boolean
  onRename: () => void
  onTestGroup: (method: TestMethod) => void
  onClearResults: () => void
  onDeleteUnavailable: (method: TestMethod) => void
}

export function ProfileGroupActionsMenu({ group, testRunning, onRename, onTestGroup, onClearResults, onDeleteUnavailable }: ProfileGroupActionsMenuProps) {
  const { t } = useTranslation('profiles')
  const canRename = group.kind !== 'default'
  const groupLabel = group.kind === 'default' ? t('group.defaultName') : group.label

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button aria-label={t('menu.more', { name: groupLabel })} className="size-9 text-muted-copy hover:bg-raised hover:text-primary" size="icon" type="button" variant="ghost">
          <MoreHorizontal aria-hidden="true" className="size-[18px]" strokeWidth={1.7} />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-52 border-hairline bg-popover p-1.5 text-[11px]" sideOffset={8}>
        <DropdownMenuItem disabled={!canRename} onSelect={onRename}>
          <Pencil aria-hidden="true" className="size-3.5" />
          <span>{canRename ? t('menu.renameGroup') : t('menu.defaultRenameDisabled')}</span>
        </DropdownMenuItem>
        {group.kind === 'subscription' ? (
          <DropdownMenuItem disabled>
            <LockKeyhole aria-hidden="true" className="size-3.5" />
            <span>{t('menu.locked')}</span>
          </DropdownMenuItem>
        ) : null}
        <DropdownMenuSeparator />
        <DropdownMenuSub>
          <DropdownMenuSubTrigger disabled={testRunning}>
            <Waypoints aria-hidden="true" className="size-3.5" />
            <span>{t('menu.testGroup')}</span>
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="w-48 border-hairline bg-popover p-1.5 text-[11px]">
            <div className="px-2 py-1.5 font-mono text-[9px] uppercase tracking-[0.08em] text-muted-copy">{groupLabel}</div>
            <DropdownMenuItem onSelect={() => onTestGroup('tcp')}>
              <Plug aria-hidden="true" className="size-3.5" />
              <span>{t('test.tcp')}</span>
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={() => onTestGroup('url')}>
              <Wifi aria-hidden="true" className="size-3.5" />
              <span>{t('test.url')}</span>
            </DropdownMenuItem>
          </DropdownMenuSubContent>
        </DropdownMenuSub>
        <DropdownMenuItem disabled={testRunning} onSelect={onClearResults}>
          <Eraser aria-hidden="true" className="size-3.5" />
          <span>{t('menu.clearResults')}</span>
        </DropdownMenuItem>
        <DropdownMenuSub>
          <DropdownMenuSubTrigger disabled={testRunning} className="text-bad focus:bg-bad/10 focus:text-bad">
            <Trash2 aria-hidden="true" className="size-3.5" />
            <span>{t('menu.deleteUnavailable')}</span>
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="w-48 border-hairline bg-popover p-1.5 text-[11px]">
            <DropdownMenuItem className="text-bad focus:bg-bad/10 focus:text-bad" onSelect={() => onDeleteUnavailable('tcp')}>
              <Plug aria-hidden="true" className="size-3.5" />
              <span>{t('test.tcp')}</span>
            </DropdownMenuItem>
            <DropdownMenuItem className="text-bad focus:bg-bad/10 focus:text-bad" onSelect={() => onDeleteUnavailable('url')}>
              <Wifi aria-hidden="true" className="size-3.5" />
              <span>{t('test.url')}</span>
            </DropdownMenuItem>
          </DropdownMenuSubContent>
        </DropdownMenuSub>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
