import { ChevronDown, ChevronRight, Lock, MonitorCog, Radar } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { ProfileTable } from '@/components/profiles/ProfileTable'
import { cn } from '@/lib/utils'
import type { Profile, ProfileGroup, TestMethod } from '@/types/kagerou'

interface ProfileGroupCardProps {
  group: ProfileGroup
  profiles: Profile[]
  runningTests: Record<string, boolean>
  onToggle: () => void
  onSelect: (id: string) => void
  onRename: (profile: Profile) => void
  onMove: (id: string, direction: 'up' | 'down') => void
  onDelete: (profile: Profile) => void
  onTest: (id: string, method: TestMethod) => void
  onReorder: (fromId: string, toId: string) => void
}

export function ProfileGroupCard({ group, profiles, runningTests, onToggle, onSelect, onRename, onMove, onDelete, onTest, onReorder }: ProfileGroupCardProps) {
  return (
    <Card className="overflow-visible rounded-[10px] border border-hairline bg-surface p-0 shadow-none">
      <div className={cn('flex min-h-[74px] items-center justify-between gap-5 border-b border-hairline px-5', !group.open && 'border-b-transparent')}>
        <Button aria-controls={`${group.id}-panel`} aria-expanded={group.open} className="min-w-0 flex-1 justify-start gap-3 !bg-transparent py-2 text-left text-primary hover:!bg-transparent focus-visible:!bg-transparent hover:text-lavender-hi" onClick={onToggle} type="button" variant="ghost">
          {group.open ? <ChevronDown aria-hidden="true" className="size-[18px] shrink-0 text-muted-copy" strokeWidth={1.7} /> : <ChevronRight aria-hidden="true" className="size-[18px] shrink-0 text-muted-copy" strokeWidth={1.7} />}
          <span className="min-w-0">
            <span className="block truncate text-[17px] font-semibold tracking-[-0.015em]">{group.label}</span>
            <span className="mt-1 block text-[12px] font-normal text-muted-copy">{profiles.length} {profiles.length === 1 ? 'profile' : 'profiles'}{group.id === 'my-profiles' ? ' · drag to reorder' : ''}</span>
          </span>
        </Button>
        {group.managed ? (
          <Badge className="gap-1.5 rounded-md border border-good/20 bg-transparent px-2.5 py-2 font-mono text-[10px] font-normal text-good" variant="outline"><Lock aria-hidden="true" className="size-3" />Managed on Sources</Badge>
        ) : (
          <span className="flex items-center gap-2 font-mono text-[10px] uppercase tracking-[0.14em] text-muted-copy"><MonitorCog aria-hidden="true" className="size-3.5" />Local collection</span>
        )}
      </div>
      {group.open ? (
        <div aria-hidden={!group.open} id={`${group.id}-panel`} role="region">
          <ProfileTable onDelete={onDelete} onMove={onMove} onRename={onRename} onReorder={onReorder} onSelect={onSelect} onTest={onTest} profiles={profiles} runningTests={runningTests} />
          {group.managed ? (
            <div className="flex items-center gap-1.5 border-t border-hairline px-5 py-3.5 text-[11px] text-muted-copy"><Radar aria-hidden="true" className="size-3.5 text-lavender" />Source-managed profiles stay grouped here. Edit the source to refresh or remove them.</div>
          ) : null}
        </div>
      ) : null}
    </Card>
  )
}
