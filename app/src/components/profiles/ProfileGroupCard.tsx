import { ChevronDown, ChevronRight, Folder, Lock, MonitorCog, Radar } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { ProfileGroupActionsMenu } from '@/components/profiles/ProfileGroupActionsMenu'
import { ProfileTable } from '@/components/profiles/ProfileTable'
import { cn } from '@/lib/utils'
import type { Profile, ProfileGroup, TestMethod } from '@/types/kagerou'

interface ProfileGroupCardProps {
  group: ProfileGroup
  profiles: Profile[]
  movableGroups: ProfileGroup[]
  runningTests: Record<string, boolean>
  onToggle: () => void
  onRenameGroup: (group: ProfileGroup) => void
  onSelect: (id: string) => void
  onRename: (profile: Profile) => void
  onMove: (id: string, direction: 'up' | 'down') => void
  onMoveToGroup: (profileId: string, groupId: string) => void
  onDelete: (profile: Profile) => void
  onTest: (id: string, method: TestMethod) => void
  onReorder: (fromId: string, toId: string) => void
}

export function ProfileGroupCard({ group, profiles, movableGroups, runningTests, onToggle, onRenameGroup, onSelect, onRename, onMove, onMoveToGroup, onDelete, onTest, onReorder }: ProfileGroupCardProps) {
  const isSubscription = group.kind === 'subscription'
  const isDefault = group.kind === 'default'

  return (
    <Card className="overflow-visible rounded-[10px] border border-hairline bg-surface p-0 shadow-none">
      <div className={cn('flex min-h-[74px] items-center justify-between gap-5 border-b border-hairline px-5', !group.open && 'border-b-transparent')}>
        <Button aria-controls={`${group.id}-panel`} aria-expanded={group.open} className="min-w-0 flex-1 justify-start gap-3 !bg-transparent py-2 text-left text-primary hover:!bg-transparent focus-visible:!bg-transparent hover:text-lavender-hi" onClick={onToggle} type="button" variant="ghost">
          {group.open ? <ChevronDown aria-hidden="true" className="size-[18px] shrink-0 text-muted-copy" strokeWidth={1.7} /> : <ChevronRight aria-hidden="true" className="size-[18px] shrink-0 text-muted-copy" strokeWidth={1.7} />}
          <span className="min-w-0">
            <span className="block truncate text-[17px] font-semibold tracking-[-0.015em]">{group.label}</span>
            <span className="mt-1 block text-[12px] font-normal text-muted-copy">{profiles.length} {profiles.length === 1 ? 'VPN' : 'VPNs'}{isDefault ? ' · single keys start here' : ''}</span>
          </span>
        </Button>
        <div className="flex shrink-0 items-center gap-1.5">
          {isSubscription ? (
            <Badge className="gap-1.5 rounded-md border border-good/20 bg-transparent px-2.5 py-2 font-mono text-[10px] font-normal text-good" variant="outline"><Lock aria-hidden="true" className="size-3" />Managed on Sources</Badge>
          ) : isDefault ? (
            <span className="flex items-center gap-2 font-mono text-[10px] uppercase tracking-[0.14em] text-muted-copy"><MonitorCog aria-hidden="true" className="size-3.5" />Default group</span>
          ) : (
            <span className="flex items-center gap-2 font-mono text-[10px] uppercase tracking-[0.14em] text-muted-copy"><Folder aria-hidden="true" className="size-3.5" />Custom group</span>
          )}
          <ProfileGroupActionsMenu group={group} onRename={() => onRenameGroup(group)} />
        </div>
      </div>
      {group.open ? (
        <div aria-hidden={!group.open} id={`${group.id}-panel`} role="region">
          <ProfileTable movableGroups={movableGroups} onDelete={onDelete} onMove={onMove} onMoveToGroup={onMoveToGroup} onRename={onRename} onReorder={onReorder} onSelect={onSelect} onTest={onTest} profiles={profiles} runningTests={runningTests} />
          {isSubscription ? (
            <div className="flex items-center gap-1.5 border-t border-hairline px-5 py-3.5 text-[11px] text-muted-copy"><Radar aria-hidden="true" className="size-3.5 text-lavender" />Subscription VPNs stay with this source. Rename or refresh the source to update them.</div>
          ) : isDefault ? (
            <div className="flex items-center gap-1.5 border-t border-hairline px-5 py-3.5 text-[11px] text-muted-copy"><Folder aria-hidden="true" className="size-3.5 text-lavender" />Single-key VPNs can be moved from Default into any custom group.</div>
          ) : null}
        </div>
      ) : null}
    </Card>
  )
}
