import { LockKeyhole, MoreHorizontal, Pencil } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import type { ProfileGroup } from '@/types/kagerou'

interface ProfileGroupActionsMenuProps {
  group: ProfileGroup
  onRename: () => void
}

export function ProfileGroupActionsMenu({ group, onRename }: ProfileGroupActionsMenuProps) {
  const { t } = useTranslation('profiles')
  const canRename = group.kind !== 'default'

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button aria-label={t('menu.more', { name: group.kind === 'default' ? t('group.defaultName') : group.label })} className="size-9 text-muted-copy hover:bg-raised hover:text-primary" size="icon" type="button" variant="ghost">
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
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
