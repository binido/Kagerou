import { Gauge } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { ResultBadge } from '@/components/common/ResultBadge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { cn } from '@/lib/utils'
import type { Profile } from '@/types/kagerou'

interface QuickProfilesProps {
  profiles: Profile[]
  activeProfileId: string
  /** False when nothing in the group has a measured latency, which would
   * make the "fastest" list an arbitrary slice of the group's own order. */
  ranked: boolean
  testRunning: boolean
  onSelect: (id: string) => void
  onTestGroup: () => void
}

export function QuickProfiles({ profiles, activeProfileId, ranked, testRunning, onSelect, onTestGroup }: QuickProfilesProps) {
  const { t } = useTranslation('dashboard')

  return (
    <Card className="flex h-full min-w-0 flex-col gap-3 overflow-hidden rounded-[10px] border border-hairline bg-surface p-5 shadow-none">
      <p className="type-eyebrow">{t('quickProfiles.title')}</p>
      {profiles.length === 0 ? (
        <p className="type-meta">{t('quickProfiles.empty')}</p>
      ) : ranked ? (
        <ul className="flex min-w-0 flex-col gap-1">
          {profiles.map((profile) => (
            <li key={profile.id} className="min-w-0">
              <button
                aria-current={profile.id === activeProfileId}
                className={cn(
                  'flex w-full min-w-0 items-center justify-between gap-3 rounded-md px-2 py-1.5 text-left text-[13px] transition-colors hover:bg-selected',
                  profile.id === activeProfileId ? 'bg-selected text-primary' : 'text-body',
                )}
                onClick={() => onSelect(profile.id)}
                type="button"
              >
                <span className="truncate">{profile.name}</span>
                <ResultBadge tone={profile.url.tone} value={profile.url.value} />
              </button>
            </li>
          ))}
        </ul>
      ) : (
        <>
          <p className="type-meta">{t('quickProfiles.untested')}</p>
          <Button className="w-full" disabled={testRunning} onClick={onTestGroup} size="sm" type="button" variant="outline">
            <Gauge aria-hidden="true" className="size-4" strokeWidth={1.7} />
            {testRunning ? t('quickProfiles.testing') : t('quickProfiles.test')}
          </Button>
        </>
      )}
    </Card>
  )
}
