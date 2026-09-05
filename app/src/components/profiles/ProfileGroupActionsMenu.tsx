import { Eraser, LockKeyhole, MoreHorizontal, Pencil, Trash2, Waypoints } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import type { ProfileGroup } from '@/types/kagerou'

interface ProfileGroupActionsMenuProps {
  group: ProfileGroup
  testRunning: boolean
  onRename: () => void
  onTestGroup: () => void
  onClearResults: () => void
  onDeleteUnavailable: () => void
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
        <DropdownMenuItem disabled={testRunning} onSelect={onTestGroup}>
          <Waypoints aria-hidden="true" className="size-3.5" />
          <span>{t('menu.testGroup')}</span>
        </DropdownMenuItem>
        <DropdownMenuItem disabled={testRunning} onSelect={onClearResults}>
          <Eraser aria-hidden="true" className="size-3.5" />
          <span>{t('menu.clearResults')}</span>
        </DropdownMenuItem>
        <DropdownMenuItem className="text-bad focus:bg-bad/10 focus:text-bad" disabled={testRunning} onSelect={onDeleteUnavailable}>
          <Trash2 aria-hidden="true" className="size-3.5" />
          <span>{t('menu.deleteUnavailable')}</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
