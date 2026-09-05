import { ChevronDown, ChevronRight, Folder, Lock, MonitorCog, Radar } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { ProfileGroupActionsMenu } from '@/components/profiles/ProfileGroupActionsMenu'
import { ProfileTable } from '@/components/profiles/ProfileTable'
import { cn } from '@/lib/utils'
import type { Profile, ProfileGroup } from '@/types/kagerou'

interface ProfileGroupCardProps {
  group: ProfileGroup
  profiles: Profile[]
  movableGroups: ProfileGroup[]
  runningTests: Record<string, boolean>
  testRunning: boolean
  onToggle: () => void
  onRenameGroup: (group: ProfileGroup) => void
  onSelect: (id: string) => void
  onRename: (profile: Profile) => void
  onMoveToGroup: (profileId: string, groupId: string) => void
  onDelete: (profile: Profile) => void
  onTest: (id: string) => void
  onTestGroup: () => void
  onClearResults: () => void
  onDeleteUnavailable: () => void
}

export function ProfileGroupCard({ group, profiles, movableGroups, runningTests, testRunning, onToggle, onRenameGroup, onSelect, onRename, onMoveToGroup, onDelete, onTest, onTestGroup, onClearResults, onDeleteUnavailable }: ProfileGroupCardProps) {
  const { t } = useTranslation('profiles')
  const { t: tc } = useTranslation('common')
  const isSubscription = group.kind === 'subscription'
  const isDefault = group.kind === 'default'
  const groupLabel = isDefault ? t('group.defaultName') : group.label
  const profileCount = tc(profiles.length === 1 ? 'counts.vpnOne' : 'counts.vpnMany', { count: profiles.length })

  return (
    <Card className="overflow-visible rounded-[10px] border border-hairline bg-surface p-0 shadow-none">
      <div className={cn('flex min-h-[74px] items-center justify-between gap-5 border-b border-hairline px-5', !group.open && 'border-b-transparent')}>
        <Button aria-controls={`${group.id}-panel`} aria-expanded={group.open} className="min-w-0 flex-1 justify-start gap-3 !bg-transparent py-2 text-left text-primary hover:!bg-transparent focus-visible:!bg-transparent hover:text-lavender-hi" onClick={onToggle} type="button" variant="ghost">
          {group.open ? <ChevronDown aria-hidden="true" className="size-[18px] shrink-0 text-muted-copy" strokeWidth={1.7} /> : <ChevronRight aria-hidden="true" className="size-[18px] shrink-0 text-muted-copy" strokeWidth={1.7} />}
          <span className="min-w-0">
            <span className="block truncate text-[17px] font-semibold tracking-[-0.015em]">{groupLabel}</span>
            <span className="mt-1 block text-[12px] font-normal text-muted-copy">{profileCount}{isDefault ? t('group.singleKeysStartHere') : ''}</span>
          </span>
        </Button>
        <div className="flex shrink-0 items-center gap-1.5">
          {isSubscription ? (
            <Badge className="gap-1.5 rounded-md border border-good/20 bg-transparent px-2.5 py-2 font-mono text-[10px] font-normal text-good" variant="outline"><Lock aria-hidden="true" className="size-3" />{t('group.managedOnSources')}</Badge>
          ) : isDefault ? (
            <span className="flex items-center gap-2 font-mono text-[10px] uppercase tracking-[0.14em] text-muted-copy"><MonitorCog aria-hidden="true" className="size-3.5" />{t('group.defaultGroup')}</span>
          ) : (
            <span className="flex items-center gap-2 font-mono text-[10px] uppercase tracking-[0.14em] text-muted-copy"><Folder aria-hidden="true" className="size-3.5" />{t('group.customGroup')}</span>
          )}
          <ProfileGroupActionsMenu group={group} onClearResults={onClearResults} onDeleteUnavailable={onDeleteUnavailable} onRename={() => onRenameGroup(group)} onTestGroup={onTestGroup} testRunning={testRunning} />
        </div>
      </div>
      {group.open ? (
        <div aria-hidden={!group.open} id={`${group.id}-panel`} role="region">
          <ProfileTable movableGroups={movableGroups} onDelete={onDelete} onMoveToGroup={onMoveToGroup} onRename={onRename} onSelect={onSelect} onTest={onTest} profiles={profiles} runningTests={runningTests} />
          {isSubscription ? (
            <div className="flex items-center gap-1.5 border-t border-hairline px-5 py-3.5 text-[11px] text-muted-copy"><Radar aria-hidden="true" className="size-3.5 text-lavender" />{t('group.subscriptionNote')}</div>
          ) : isDefault ? (
            <div className="flex items-center gap-1.5 border-t border-hairline px-5 py-3.5 text-[11px] text-muted-copy"><Folder aria-hidden="true" className="size-3.5 text-lavender" />{t('group.defaultNote')}</div>
          ) : null}
        </div>
      ) : null}
    </Card>
  )
}
