import {
  ArrowDown,
  ArrowUp,
  ExternalLink,
  MoreHorizontal,
  Pencil,
  Plug,
  Trash2,
  Waypoints,
  Wifi,
} from 'lucide-react'

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
import { Button } from '@/components/ui/button'
import type { Profile, TestMethod } from '@/types/kagerou'

interface ProfileActionsMenuProps {
  profile: Profile
  onRename: () => void
  onMove: (direction: 'up' | 'down') => void
  onDelete: () => void
  onTest: (method: TestMethod) => void
}

export function ProfileActionsMenu({ profile, onRename, onMove, onDelete, onTest }: ProfileActionsMenuProps) {
  const local = profile.origin === 'local'

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button aria-label={`More actions for ${profile.name}`} className="size-9 text-muted-copy hover:bg-raised hover:text-primary" size="icon" type="button" variant="ghost">
          <MoreHorizontal aria-hidden="true" className="size-[18px]" strokeWidth={1.7} />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-52 border-hairline bg-popover p-1.5 text-[11px]" sideOffset={8}>
        <DropdownMenuItem disabled={!local} onSelect={onRename}>
          <Pencil aria-hidden="true" className="size-3.5" />
          <span>{local ? 'Rename profile' : 'Rename profile'}</span>
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={() => onMove('up')}>
          <ArrowUp aria-hidden="true" className="size-3.5" />
          <span>Move up</span>
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={() => onMove('down')}>
          <ArrowDown aria-hidden="true" className="size-3.5" />
          <span>Move down</span>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem className="text-bad focus:bg-bad/10 focus:text-bad" disabled={!local} onSelect={onDelete}>
          <Trash2 aria-hidden="true" className="size-3.5" />
          <span>Delete profile</span>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuSub>
          <DropdownMenuSubTrigger>
            <Waypoints aria-hidden="true" className="size-3.5" />
            <span>Test</span>
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="w-48 border-[#3b3a45] bg-[#25262d] p-1.5 text-[11px]">
            <div className="px-2 py-1.5 font-mono text-[9px] uppercase tracking-[0.08em] text-muted-copy">{profile.name}</div>
            <DropdownMenuItem onSelect={() => onTest('tcp')}>
              <Plug aria-hidden="true" className="size-3.5" />
              <span>TCP connection</span>
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={() => onTest('url')}>
              <Wifi aria-hidden="true" className="size-3.5" />
              <span>URL check</span>
            </DropdownMenuItem>
          </DropdownMenuSubContent>
        </DropdownMenuSub>
        {!local ? (
          <>
            <DropdownMenuSeparator />
            <DropdownMenuItem disabled>
              <ExternalLink aria-hidden="true" className="size-3.5" />
              <span>Managed on Sources</span>
            </DropdownMenuItem>
          </>
        ) : null}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
