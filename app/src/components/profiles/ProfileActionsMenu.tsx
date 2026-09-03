import {
  ArrowRight,
  ExternalLink,
  MoreHorizontal,
  Pencil,
  Plug,
  Trash2,
  Waypoints,
  Wifi,
} from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import type { Profile, ProfileGroup, TestMethod } from '@/types/kagerou'

interface ProfileActionsMenuProps {
  profile: Profile
  movableGroups: ProfileGroup[]
  onRename: () => void
  onMoveToGroup: (groupId: string) => void
  onDelete: () => void
  onTest: (method: TestMethod) => void
}

export function ProfileActionsMenu({ profile, movableGroups, onRename, onMoveToGroup, onDelete, onTest }: ProfileActionsMenuProps) {
  const { t } = useTranslation('profiles')
  const local = profile.origin === 'local'
  const targetGroups = movableGroups.filter((group) => group.kind !== 'subscription' && group.id !== profile.groupId)
  const groupLabel = (group: ProfileGroup) => group.kind === 'default' ? t('group.defaultName') : group.label

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button aria-label={t('menu.more', { name: profile.name })} className="size-9 text-muted-copy hover:bg-raised hover:text-primary" size="icon" type="button" variant="ghost">
          <MoreHorizontal aria-hidden="true" className="size-[18px]" strokeWidth={1.7} />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-56 border-hairline bg-popover p-1.5 text-[11px]" sideOffset={8}>
        <DropdownMenuItem disabled={!local} onSelect={onRename}>
          <Pencil aria-hidden="true" className="size-3.5" />
          <span>{t('menu.rename')}</span>
        </DropdownMenuItem>
        <DropdownMenuSub>
          <DropdownMenuSubTrigger disabled={!local || targetGroups.length === 0}>
            <ArrowRight aria-hidden="true" className="size-3.5" />
            <span>{t('menu.move')}</span>
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="w-48 border-hairline bg-popover p-1.5 text-[11px]">
            {targetGroups.map((group) => (
              <DropdownMenuItem key={group.id} onSelect={() => onMoveToGroup(group.id)}>
                <span className="min-w-0 truncate">{groupLabel(group)}</span>
              </DropdownMenuItem>
            ))}
          </DropdownMenuSubContent>
        </DropdownMenuSub>
        <DropdownMenuSeparator />
        <DropdownMenuItem className="text-bad focus:bg-bad/10 focus:text-bad" disabled={!local} onSelect={onDelete}>
          <Trash2 aria-hidden="true" className="size-3.5" />
          <span>{t('menu.delete')}</span>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuSub>
          <DropdownMenuSubTrigger>
            <Waypoints aria-hidden="true" className="size-3.5" />
            <span>{t('menu.test')}</span>
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="w-48 border-hairline bg-popover p-1.5 text-[11px]">
            <div className="px-2 py-1.5 font-mono text-[9px] uppercase tracking-[0.08em] text-muted-copy">{profile.name}</div>
            <DropdownMenuItem onSelect={() => onTest('tcp')}>
              <Plug aria-hidden="true" className="size-3.5" />
              <span>{t('test.tcp')}</span>
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={() => onTest('url')}>
              <Wifi aria-hidden="true" className="size-3.5" />
              <span>{t('test.url')}</span>
            </DropdownMenuItem>
          </DropdownMenuSubContent>
        </DropdownMenuSub>
        {!local ? (
          <>
            <DropdownMenuSeparator />
            <DropdownMenuItem disabled>
              <ExternalLink aria-hidden="true" className="size-3.5" />
              <span>{t('menu.managed')}</span>
            </DropdownMenuItem>
          </>
        ) : null}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
