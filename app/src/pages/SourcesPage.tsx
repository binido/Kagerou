import { useState } from 'react'
import { Info } from 'lucide-react'
import { toast } from 'sonner'

import { AddSourceMenu } from '@/components/sources/AddSourceMenu'
import { RemoveSourceDialog } from '@/components/sources/RemoveSourceDialog'
import { SourceCard } from '@/components/sources/SourceCard'
import { SourceDialog } from '@/components/sources/SourceDialog'
import { PageHeader } from '@/components/layout/PageHeader'
import { mockApi } from '@/lib/mock-api'
import { useKagerouStore } from '@/store/kagerou-store'
import type { Source, SourceType } from '@/types/kagerou'

export function SourcesPage() {
  const sources = useKagerouStore((state) => state.sources)
  const addSource = useKagerouStore((state) => state.addSource)
  const updateSource = useKagerouStore((state) => state.updateSource)
  const removeSource = useKagerouStore((state) => state.removeSource)
  const [dialogOpen, setDialogOpen] = useState(false)
  const [dialogType, setDialogType] = useState<SourceType>('url')
  const [editingSource, setEditingSource] = useState<Source | null>(null)
  const [removingSource, setRemovingSource] = useState<Source | null>(null)
  const [refreshingIds, setRefreshingIds] = useState<Record<string, boolean>>({})

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

  const handleSourceSubmit = (type: SourceType, value: string) => {
    if (editingSource) {
      updateSource(editingSource.id, {
        type,
        value,
        originLabel: type === 'url' ? 'Remote URL' : 'Local key',
        status: type === 'url' ? 'up-to-date' : 'ready',
      })
      toast.success('Source changes saved')
    } else {
      const sourceNumber = sources.length + 1
      addSource({
        id: `source-${Date.now()}`,
        name: type === 'url' ? `Imported source ${String(sourceNumber).padStart(2, '0')}` : `Access key ${String(sourceNumber).padStart(2, '0')}`,
        type,
        value,
        profileCount: type === 'url' ? 0 : 1,
        status: type === 'url' ? 'up-to-date' : 'ready',
        lastRefresh: type === 'url' ? 'Updated just now' : 'Added just now',
        originLabel: type === 'url' ? 'Remote URL' : 'Local key',
      })
      toast.success(type === 'url' ? 'Source added · profiles imported' : 'Source added · 1 profile ready')
    }
    setDialogOpen(false)
  }

  const refresh = async (source: Source) => {
    if (refreshingIds[source.id]) return
    setRefreshingIds((state) => ({ ...state, [source.id]: true }))
    updateSource(source.id, { status: 'updating' })
    const toastId = toast.loading(`Refreshing ${source.name}…`)
    await mockApi.refreshSource(source.id)
    updateSource(source.id, { status: source.type === 'key' ? 'ready' : 'up-to-date', lastRefresh: source.type === 'key' ? 'Checked just now' : 'Updated just now' })
    setRefreshingIds((state) => { const next = { ...state }; delete next[source.id]; return next })
    toast.success('Source refreshed · profiles are ready', { id: toastId })
  }

  const confirmRemove = () => {
    if (!removingSource) return
    const name = removingSource.name
    removeSource(removingSource.id)
    setRemovingSource(null)
    toast.success(`${name} removed · profiles remain available`)
  }

  const profileCount = sources.reduce((total, source) => total + source.profileCount, 0)

  return (
    <div className="min-h-screen min-w-0 bg-canvas px-6 pb-12 pt-8 lg:px-12 lg:pt-10">
      <div className="mx-auto w-full max-w-[1040px]">
        <PageHeader actions={<AddSourceMenu onChoose={openAdd} />} description="Manage URLs and keys that provide profiles to Kagerou." eyebrow="Kagerou  /  Profile sources" title="Sources" />
        <div className="mt-8 flex items-center justify-between border-b border-hairline pb-3 max-[720px]:items-start max-[720px]:gap-4">
          <p className="type-data text-body">{sources.length} sources <span className="px-1.5 text-quiet">·</span> {profileCount} profiles</p>
          <p className="flex items-center gap-2 text-[11px] text-muted-copy max-[720px]:text-right"><Info aria-hidden="true" className="size-3.5 shrink-0" />Selection and order live on Profiles</p>
        </div>
        <section aria-label="Profile sources" className="mt-4 space-y-3">
          {sources.map((source) => <SourceCard key={source.id} onEdit={() => openEdit(source)} onRefresh={() => void refresh(source)} onRemove={() => setRemovingSource(source)} refreshing={Boolean(refreshingIds[source.id])} source={source} />)}
        </section>
      </div>
      <SourceDialog key={`${editingSource?.id ?? 'new'}-${dialogType}-${dialogOpen}`} initialType={dialogType} onOpenChange={(open) => { setDialogOpen(open); if (!open) setEditingSource(null) }} onSubmit={handleSourceSubmit} open={dialogOpen} source={editingSource} />
      <RemoveSourceDialog onConfirm={confirmRemove} onOpenChange={(open) => { if (!open) setRemovingSource(null) }} source={removingSource} />
    </div>
  )
}
