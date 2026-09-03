import { Check } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { ProfileActionsMenu } from '@/components/profiles/ProfileActionsMenu'
import { ResultBadge } from '@/components/common/ResultBadge'
import { getReachabilityAwarePing } from '@/lib/profile-sorting'
import { cn } from '@/lib/utils'
import type { Profile, ProfileGroup, TestMethod } from '@/types/kagerou'

interface ProfileTableProps {
  profiles: Profile[]
  movableGroups: ProfileGroup[]
  runningTests: Record<string, boolean>
  onSelect: (id: string) => void
  onRename: (profile: Profile) => void
  onMoveToGroup: (profileId: string, groupId: string) => void
  onDelete: (profile: Profile) => void
  onTest: (id: string, method: TestMethod) => void
}

export function ProfileTable({ profiles, movableGroups, runningTests, onSelect, onRename, onMoveToGroup, onDelete, onTest }: ProfileTableProps) {

  return (
    <div className="overflow-x-auto">
      <Table className="min-w-[900px] text-left">
        <TableHeader>
          <TableRow className="border-b border-hairline hover:bg-transparent">
            <TableHead className="w-[58px] px-5 py-3 text-[10px] font-medium uppercase tracking-[0.14em] text-muted-copy">Order</TableHead>
            <TableHead className="min-w-[270px] px-3 py-3 text-[10px] font-medium uppercase tracking-[0.14em] text-muted-copy">VPN</TableHead>
            <TableHead className="w-[108px] px-3 py-3 text-[10px] font-medium uppercase tracking-[0.14em] text-muted-copy">Protocol</TableHead>
            <TableHead className="w-[122px] px-3 py-3 text-[10px] font-medium uppercase tracking-[0.14em] text-muted-copy">Ping</TableHead>
            <TableHead className="w-[126px] px-3 py-3 text-[10px] font-medium uppercase tracking-[0.14em] text-muted-copy">URL</TableHead>
            <TableHead className="w-[100px] px-3 py-3 text-[10px] font-medium uppercase tracking-[0.14em] text-muted-copy">Use</TableHead>
            <TableHead className="w-[54px] px-3 py-3 text-right text-[10px] font-medium uppercase tracking-[0.14em] text-muted-copy"><span className="sr-only">Actions</span></TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {profiles.map((profile, index) => {
            const runningTcp = runningTests[`${profile.id}:tcp`]
            const runningUrl = runningTests[`${profile.id}:url`]
            const pingResult = runningUrl
              ? { value: 'Checking…', tone: 'warn' as const }
              : profile.url.tone === 'good' && runningTcp
                ? { value: 'Running…', tone: 'warn' as const }
                : getReachabilityAwarePing(profile)
            return (
              <TableRow
                className={cn('min-h-[75px] border-b border-white/[0.055] text-body hover:bg-row-hover focus-within:bg-row-hover', profile.selected && 'bg-selected hover:bg-selected')}
                data-profile-id={profile.id}
                key={profile.id}
              >
                <TableCell className="px-5 py-4 align-middle">
                  <div className="font-mono text-[12px] tabular-nums text-muted-copy">
                    <span className="w-3 text-center">{index + 1}</span>
                  </div>
                </TableCell>
                <TableCell className="max-w-[330px] px-3 py-4 align-middle">
                  <div className="min-w-0">
                    <div className="flex min-w-0 items-center gap-2"><span className="truncate text-[14px] font-medium text-primary">{profile.name}</span><Badge className={cn('h-5 rounded-md px-1.5 py-0 text-[10px] font-semibold', profile.origin === 'local' ? 'bg-lavender/15 text-lavender-hi' : 'bg-good/15 text-good')} variant="outline">{profile.origin === 'local' ? 'Local' : 'Imported'}</Badge></div>
                    <span className="mt-1 block truncate text-[11px] text-muted-copy">{profile.origin === 'local' ? 'Local VPN' : 'Managed by subscription'}</span>
                  </div>
                </TableCell>
                <TableCell className="px-3 py-4 align-middle"><span className="inline-flex rounded-md bg-raised px-2 py-1 font-mono text-[10px] text-body">{profile.protocol}</span></TableCell>
                <TableCell className="px-3 py-4 align-middle"><ResultBadge tone={pingResult.tone} value={pingResult.value} /></TableCell>
                <TableCell className="px-3 py-4 align-middle"><ResultBadge tone={runningUrl ? 'warn' : profile.url.tone} value={runningUrl ? 'Running…' : profile.url.value} /></TableCell>
                <TableCell className="px-3 py-4 align-middle"><Button aria-pressed={profile.selected} className="h-[34px] w-[84px] justify-center gap-1.5 rounded-md border-hairline px-2.5 text-[11px] text-body hover:border-[#464650] hover:bg-raised hover:text-primary aria-pressed:border-transparent aria-pressed:bg-lavender aria-pressed:font-bold aria-pressed:text-ink aria-pressed:hover:bg-lavender-hi" onClick={() => onSelect(profile.id)} type="button" variant="outline">{profile.selected ? <Check aria-hidden="true" className="size-3" strokeWidth={2.5} /> : null}{profile.selected ? 'Selected' : 'Use'}</Button></TableCell>
                <TableCell className="px-3 py-4 text-right align-middle"><ProfileActionsMenu movableGroups={movableGroups} onDelete={() => onDelete(profile)} onMoveToGroup={(groupId) => onMoveToGroup(profile.id, groupId)} onRename={() => onRename(profile)} onTest={(method) => onTest(profile.id, method)} profile={profile} /></TableCell>
              </TableRow>
            )
          })}
        </TableBody>
      </Table>
    </div>
  )
}
