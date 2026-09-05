import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Info } from 'lucide-react'
import { toast } from 'sonner'

import { AddSourceMenu } from '@/components/sources/AddSourceMenu'
import { RemoveSourceDialog } from '@/components/sources/RemoveSourceDialog'
import { SourceCard } from '@/components/sources/SourceCard'
import { SourceDialog } from '@/components/sources/SourceDialog'
import { PageContainer } from '@/components/layout/PageContainer'
import { PageHeader } from '@/components/layout/PageHeader'
import { deriveSubscriptionName } from '@/lib/formatters'
import { useKagerouStore } from '@/store/kagerou-store'
import type { AddSourceInput, Source, SourceType } from '@/types/kagerou'

export function SourcesPage() {
  const { t } = useTranslation('sources')
  const sources = useKagerouStore((state) => state.sources)
  const profiles = useKagerouStore((state) => state.profiles)
  const addSource = useKagerouStore((state) => state.addSource)
  const updateSource = useKagerouStore((state) => state.updateSource)
  const refreshSource = useKagerouStore((state) => state.refreshSource)
  const removeSource = useKagerouStore((state) => state.removeSource)
  const [dialogOpen, setDialogOpen] = useState(false)
  const [dialogType, setDialogType] = useState<SourceType>('url')
  const [editingSource, setEditingSource] = useState<Source | null>(null)
  const [removingSource, setRemovingSource] = useState<Source | null>(null)
  const [refreshingIds, setRefreshingIds] = useState<Record<string, boolean>>({})

  const profileCountBySourceId = useMemo(() => profiles.reduce<Record<string, number>>((counts, profile) => {
    if (profile.sourceId) counts[profile.sourceId] = (counts[profile.sourceId] ?? 0) + 1
    return counts
  }, {}), [profiles])

  const openAdd = (type: SourceType) => {
    setEditingSource(null)
    setDialogType(type)
    setDialogOpen(true)
  }

  const openEdit = (source: Source) => {
    setEditingSource(source)
    setDialogType(source.type)
    setDialogOpen(true)
  }

  const handleSourceSubmit = async (input: AddSourceInput) => {
    if (editingSource) {
      const nextName = input.name ?? editingSource.name
      if (!(await updateSource(editingSource.id, { name: nextName, value: input.value }))) {
        throw new Error(t('feedback.nameError'))
      }
      if (editingSource.type === 'url') {
        await refreshSource(editingSource.id)
      }
      toast.success(t('feedback.saved'))
      return
    }

    const sourceName = input.name ?? (input.type === 'url'
      ? deriveSubscriptionName(input.value, sources.length + 1, t('defaults.subscription'))
      : t('defaults.keyName', { scheme: input.value.split('://')[0]?.toUpperCase() || 'VPN', number: String(sources.length + 1).padStart(2, '0') }))
    const sourceId = await addSource({ ...input, name: sourceName })
    if (!sourceId) throw new Error(t('feedback.importError'))
    toast.success(input.type === 'url' ? t('feedback.subscriptionAdded') : t('feedback.keyAdded'))
  }

  const refresh = async (source: Source) => {
    if (refreshingIds[source.id]) return
    setRefreshingIds((state) => ({ ...state, [source.id]: true }))
    const toastId = toast.loading(t('feedback.refreshing', { name: source.name }))
    try {
      await refreshSource(source.id)
      toast.success(t('feedback.refreshed'), { id: toastId })
    } catch (refreshError) {
      toast.error(refreshError instanceof Error ? refreshError.message : t('feedback.refreshFailed'), { id: toastId })
    } finally {
      setRefreshingIds((state) => { const next = { ...state }; delete next[source.id]; return next })
    }
  }

  const confirmRemove = () => {
    if (!removingSource) return
    const name = removingSource.name
    removeSource(removingSource.id)
    setRemovingSource(null)
    toast.success(t('feedback.removed', { name }))
  }

  const profileCount = profiles.length

  return (
    <PageContainer>
        <PageHeader actions={<AddSourceMenu onChoose={openAdd} />} description={t('page.description')} eyebrow={t('page.eyebrow')} title={t('page.title')} />
        <div className="mt-8 flex items-center justify-between border-b border-hairline pb-3 max-[720px]:items-start max-[720px]:gap-4">
          <p className="type-data text-body">{t('page.summary', { sources: sources.length, vpns: profileCount })}</p>
          <p className="flex items-center gap-2 text-[11px] text-muted-copy max-[720px]:text-right"><Info aria-hidden="true" className="size-3.5 shrink-0" />{t('page.info')}</p>
        </div>
        <section aria-label={t('page.ariaLabel')} className="mt-4 grid gap-3 lg:grid-cols-2">
          {sources.map((source) => <SourceCard key={source.id} onEdit={() => openEdit(source)} onRefresh={() => void refresh(source)} onRemove={() => setRemovingSource(source)} profileCount={profileCountBySourceId[source.id] ?? 0} refreshing={Boolean(refreshingIds[source.id])} source={source} />)}
        </section>
      <SourceDialog key={`${editingSource?.id ?? 'new'}-${dialogType}-${dialogOpen}`} initialType={dialogType} onOpenChange={(open) => { setDialogOpen(open); if (!open) setEditingSource(null) }} onSubmit={handleSourceSubmit} open={dialogOpen} source={editingSource} />
      <RemoveSourceDialog onConfirm={confirmRemove} onOpenChange={(open) => { if (!open) setRemovingSource(null) }} source={removingSource} />
    </PageContainer>
  )
}
