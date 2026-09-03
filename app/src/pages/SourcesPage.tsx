import { useMemo, useState } from 'react'
import { Info } from 'lucide-react'
import { toast } from 'sonner'

import { AddSourceMenu } from '@/components/sources/AddSourceMenu'
import { RemoveSourceDialog } from '@/components/sources/RemoveSourceDialog'
import { SourceCard } from '@/components/sources/SourceCard'
import { SourceDialog } from '@/components/sources/SourceDialog'
import { PageHeader } from '@/components/layout/PageHeader'
import { deriveSubscriptionName } from '@/lib/formatters'
import { mockApi } from '@/lib/mock-api'
import { useKagerouStore } from '@/store/kagerou-store'
import type { AddSourceInput, Source, SourceType } from '@/types/kagerou'

export function SourcesPage() {
  const sources = useKagerouStore((state) => state.sources)
  const profiles = useKagerouStore((state) => state.profiles)
  const addSource = useKagerouStore((state) => state.addSource)
  const updateSource = useKagerouStore((state) => state.updateSource)
  const replaceSubscriptionProfiles = useKagerouStore((state) => state.replaceSubscriptionProfiles)
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
      if (!updateSource(editingSource.id, { name: nextName, value: input.value })) {
        throw new Error('The source name cannot be empty.')
      }
      if (editingSource.type === 'url') {
        const imported = await mockApi.importSubscription(input.value)
        replaceSubscriptionProfiles(editingSource.id, imported.profiles)
      }
      toast.success('Source changes saved')
      return
    }

    const sourceName = input.name ?? (input.type === 'url' ? deriveSubscriptionName(input.value, sources.length + 1) : undefined)
    const imported = input.type === 'url' ? await mockApi.importSubscription(input.value) : undefined
    const sourceId = addSource({ ...input, name: sourceName }, imported?.profiles)
    if (!sourceId) throw new Error('The source could not be added.')
    toast.success(input.type === 'url' ? 'Subscription added · VPNs grouped' : 'Single key added to Default')
  }

  const refresh = async (source: Source) => {
    if (refreshingIds[source.id]) return
    setRefreshingIds((state) => ({ ...state, [source.id]: true }))
    updateSource(source.id, { status: 'updating' })
    const toastId = toast.loading(`Refreshing ${source.name}…`)
    try {
      const imported = await mockApi.refreshSource(source)
      if (source.type === 'url') replaceSubscriptionProfiles(source.id, imported.profiles)
      updateSource(source.id, { status: source.type === 'key' ? 'ready' : 'up-to-date', lastRefresh: source.type === 'key' ? 'Checked just now' : 'Updated just now' })
      toast.success('Source refreshed · VPNs are ready', { id: toastId })
    } catch (refreshError) {
      updateSource(source.id, { status: 'refresh-due' })
      toast.error(refreshError instanceof Error ? refreshError.message : 'Source refresh failed.', { id: toastId })
    } finally {
      setRefreshingIds((state) => { const next = { ...state }; delete next[source.id]; return next })
    }
  }

  const confirmRemove = () => {
    if (!removingSource) return
    const name = removingSource.name
    removeSource(removingSource.id)
    setRemovingSource(null)
    toast.success(`${name} removed · VPNs are now local`)
  }

  const profileCount = profiles.length

  return (
    <div className="min-h-screen min-w-0 bg-canvas px-6 pb-12 pt-8 lg:px-12 lg:pt-10">
      <div className="mx-auto w-full max-w-[1040px]">
        <PageHeader actions={<AddSourceMenu onChoose={openAdd} />} description="Manage subscriptions and single keys that provide VPNs to Kagerou." eyebrow="Kagerou  /  VPN sources" title="Sources" />
        <div className="mt-8 flex items-center justify-between border-b border-hairline pb-3 max-[720px]:items-start max-[720px]:gap-4">
          <p className="type-data text-body">{sources.length} sources <span className="px-1.5 text-quiet">·</span> {profileCount} VPNs</p>
          <p className="flex items-center gap-2 text-[11px] text-muted-copy max-[720px]:text-right"><Info aria-hidden="true" className="size-3.5 shrink-0" />Groups and order live on Groups</p>
        </div>
        <section aria-label="VPN sources" className="mt-4 space-y-3">
          {sources.map((source) => <SourceCard key={source.id} onEdit={() => openEdit(source)} onRefresh={() => void refresh(source)} onRemove={() => setRemovingSource(source)} profileCount={profileCountBySourceId[source.id] ?? 0} refreshing={Boolean(refreshingIds[source.id])} source={source} />)}
        </section>
      </div>
      <SourceDialog key={`${editingSource?.id ?? 'new'}-${dialogType}-${dialogOpen}`} initialType={dialogType} onOpenChange={(open) => { setDialogOpen(open); if (!open) setEditingSource(null) }} onSubmit={handleSourceSubmit} open={dialogOpen} source={editingSource} />
      <RemoveSourceDialog onConfirm={confirmRemove} onOpenChange={(open) => { if (!open) setRemovingSource(null) }} source={removingSource} />
    </div>
  )
}
