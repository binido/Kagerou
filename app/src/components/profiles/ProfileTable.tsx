import { useSyncExternalStore } from 'react'
import { useTranslation } from 'react-i18next'
import { Check } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { ProfileActionsMenu } from '@/components/profiles/ProfileActionsMenu'
import { ResultBadge } from '@/components/common/ResultBadge'
import { getReachabilityAwarePing } from '@/lib/profile-sorting'
import { cn } from '@/lib/utils'
import type { Profile, ProfileGroup, TestMethod, TestResult } from '@/types/kagerou'

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

interface ProfileResultState {
  ping: TestResult
  url: TestResult
}

// Tailwind's max-[1220px] compiles to `width < 1220px`; max-width: 1220px would disagree with it at exactly 1220
const NARROW_QUERY = '(max-width: 1219.98px)'

function subscribeNarrow(onChange: () => void) {
  const queryList = window.matchMedia(NARROW_QUERY)
  queryList.addEventListener('change', onChange)
  return () => queryList.removeEventListener('change', onChange)
}

function useNarrowViewport() {
  return useSyncExternalStore(subscribeNarrow, () => window.matchMedia(NARROW_QUERY).matches)
}

function getProfileResultState(profile: Profile, runningTests: Record<string, boolean>, labels: { checking: string; running: string }): ProfileResultState {
  const runningTcp = runningTests[`${profile.id}:tcp`]
  const runningUrl = runningTests[`${profile.id}:url`]

  return {
    ping: runningUrl
      ? { value: labels.checking, tone: 'warn' }
      : profile.url.tone === 'good' && runningTcp
        ? { value: labels.running, tone: 'warn' }
        : getReachabilityAwarePing(profile),
    url: runningUrl ? { value: labels.running, tone: 'warn' } : profile.url,
  }
}

function ProfileSelectButton({ profile, compact = false, onSelect }: { profile: Profile; compact?: boolean; onSelect: (id: string) => void }) {
  const { t } = useTranslation('profiles')

  return (
    <Button aria-pressed={profile.selected} className={cn('w-[104px] shrink-0 justify-center gap-1.5 whitespace-nowrap rounded-md border-hairline !bg-raised text-primary hover:!border-lavender/45 hover:!bg-selected hover:text-primary aria-pressed:!border-transparent aria-pressed:!bg-primary aria-pressed:font-bold aria-pressed:!text-primary-foreground aria-pressed:hover:!bg-primary max-[400px]:w-24', compact ? 'h-8 px-2 text-[10px]' : 'h-[34px] px-2.5 text-[11px]')} onClick={() => onSelect(profile.id)} type="button" variant="outline">
      {profile.selected ? <Check aria-hidden="true" className="size-3" strokeWidth={2.5} /> : null}
      {profile.selected ? t('table.selected') : t('table.use')}
    </Button>
  )
}

function ProfileCompactRow({
  profile,
  index,
  movableGroups,
  result,
  onSelect,
  onRename,
  onMoveToGroup,
  onDelete,
  onTest,
}: {
  profile: Profile
  index: number
  movableGroups: ProfileGroup[]
  result: ProfileResultState
  onSelect: (id: string) => void
  onRename: (profile: Profile) => void
  onMoveToGroup: (profileId: string, groupId: string) => void
  onDelete: (profile: Profile) => void
  onTest: (id: string, method: TestMethod) => void
}) {
  const { t } = useTranslation('profiles')

  return (
    <div className="flex min-w-0 gap-3 border-b border-hairline/55 px-4 py-3 last:border-b-0" data-profile-id={profile.id}>
      <span aria-hidden="true" className="w-4 shrink-0 pt-1 font-mono text-[11px] tabular-nums text-muted-copy">{index + 1}</span>
      <div className="min-w-0 flex-1">
        <div className="flex min-w-0 items-center gap-2">
          <span className="min-w-0 truncate text-[14px] font-medium text-primary">{profile.name}</span>
          <Badge className={cn('h-5 shrink-0 rounded-md px-1.5 py-0 text-[10px] font-semibold', profile.origin === 'local' ? 'bg-lavender/15 text-lavender-hi' : 'bg-good/15 text-good')} variant="outline">
            {profile.origin === 'local' ? t('table.local') : t('table.imported')}
          </Badge>
        </div>
        <span className="mt-1 block truncate text-[11px] text-muted-copy">{profile.origin === 'local' ? t('table.localVpn') : t('table.managedBySubscription')}</span>
        <div className="mt-3 flex min-w-0 flex-wrap items-center gap-x-3 gap-y-2 border-t border-hairline/55 pt-3 text-[10px] text-muted-copy">
          <span className="rounded-md bg-raised px-2 py-1 font-mono text-body">{profile.protocol}</span>
          <span className="inline-flex min-w-0 items-center gap-1.5">
            <span>{t('table.ping')}</span>
            <ResultBadge tone={result.ping.tone} value={result.ping.value} />
          </span>
          <span className="inline-flex min-w-0 items-center gap-1.5">
            <span>{t('table.url')}</span>
            <ResultBadge tone={result.url.tone} value={result.url.value} />
          </span>
          <span className="ml-auto flex shrink-0 items-center gap-1.5">
            <ProfileSelectButton compact onSelect={onSelect} profile={profile} />
            <ProfileActionsMenu movableGroups={movableGroups} onDelete={() => onDelete(profile)} onMoveToGroup={(groupId) => onMoveToGroup(profile.id, groupId)} onRename={() => onRename(profile)} onTest={(method) => onTest(profile.id, method)} profile={profile} />
          </span>
        </div>
      </div>
    </div>
  )
}

export function ProfileTable({ profiles, movableGroups, runningTests, onSelect, onRename, onMoveToGroup, onDelete, onTest }: ProfileTableProps) {
  const { t } = useTranslation('profiles')
  const testLabels = { checking: t('table.checking'), running: t('table.running') }
  const narrow = useNarrowViewport()

  return (
    <div className="min-w-0">
      {narrow ? (
        profiles.map((profile, index) => (
          <ProfileCompactRow
            index={index}
            key={profile.id}
            movableGroups={movableGroups}
            onDelete={onDelete}
            onMoveToGroup={onMoveToGroup}
            onRename={onRename}
            onSelect={onSelect}
            onTest={onTest}
            profile={profile}
            result={getProfileResultState(profile, runningTests, testLabels)}
          />
        ))
      ) : (
        <Table className="w-full text-left">
          <TableHeader>
            <TableRow className="border-b border-hairline hover:bg-transparent">
              <TableHead className="w-[58px] px-5 py-3 text-[10px] font-medium uppercase tracking-[0.14em] text-muted-copy">{t('table.order')}</TableHead>
              <TableHead className="w-[30%] px-3 py-3 text-[10px] font-medium uppercase tracking-[0.14em] text-muted-copy">{t('table.vpn')}</TableHead>
              <TableHead className="w-[12%] px-3 py-3 text-[10px] font-medium uppercase tracking-[0.14em] text-muted-copy">{t('table.protocol')}</TableHead>
              <TableHead className="w-[13%] px-3 py-3 text-[10px] font-medium uppercase tracking-[0.14em] text-muted-copy">{t('table.ping')}</TableHead>
              <TableHead className="w-[14%] px-3 py-3 text-[10px] font-medium uppercase tracking-[0.14em] text-muted-copy">{t('table.url')}</TableHead>
              <TableHead className="w-[13%] whitespace-nowrap px-3 py-3 text-[10px] font-medium uppercase tracking-[0.14em] text-muted-copy">{t('table.use')}</TableHead>
              <TableHead className="w-[54px] px-3 py-3 text-right text-[10px] font-medium uppercase tracking-[0.14em] text-muted-copy"><span className="sr-only">{t('table.actions')}</span></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {profiles.map((profile, index) => {
              const result = getProfileResultState(profile, runningTests, testLabels)
              return (
                <TableRow className={cn('min-h-[75px] border-b border-hairline/55 text-body hover:bg-row-hover focus-within:bg-row-hover', profile.selected && 'bg-selected hover:bg-selected')} data-profile-id={profile.id} key={profile.id}>
                  <TableCell className="px-5 py-4 align-middle"><div className="font-mono text-[12px] tabular-nums text-muted-copy"><span className="w-3 text-center">{index + 1}</span></div></TableCell>
                  <TableCell className="max-w-[330px] px-3 py-4 align-middle">
                    <div className="min-w-0">
                      <div className="flex min-w-0 items-center gap-2"><span className="truncate text-[14px] font-medium text-primary">{profile.name}</span><Badge className={cn('h-5 rounded-md px-1.5 py-0 text-[10px] font-semibold', profile.origin === 'local' ? 'bg-lavender/15 text-lavender-hi' : 'bg-good/15 text-good')} variant="outline">{profile.origin === 'local' ? t('table.local') : t('table.imported')}</Badge></div>
                      <span className="mt-1 block truncate text-[11px] text-muted-copy">{profile.origin === 'local' ? t('table.localVpn') : t('table.managedBySubscription')}</span>
                    </div>
                  </TableCell>
                  <TableCell className="px-3 py-4 align-middle"><span className="inline-flex rounded-md bg-raised px-2 py-1 font-mono text-[10px] text-body">{profile.protocol}</span></TableCell>
                  <TableCell className="px-3 py-4 align-middle"><ResultBadge tone={result.ping.tone} value={result.ping.value} /></TableCell>
                  <TableCell className="px-3 py-4 align-middle"><ResultBadge tone={result.url.tone} value={result.url.value} /></TableCell>
                  <TableCell className="px-3 py-4 align-middle"><ProfileSelectButton onSelect={onSelect} profile={profile} /></TableCell>
                  <TableCell className="px-3 py-4 text-right align-middle"><ProfileActionsMenu movableGroups={movableGroups} onDelete={() => onDelete(profile)} onMoveToGroup={(groupId) => onMoveToGroup(profile.id, groupId)} onRename={() => onRename(profile)} onTest={(method) => onTest(profile.id, method)} profile={profile} /></TableCell>
                </TableRow>
              )
            })}
          </TableBody>
        </Table>
      )}
    </div>
  )
}
