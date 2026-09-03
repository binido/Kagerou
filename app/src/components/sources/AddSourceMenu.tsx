import { KeyRound, Plus, Rss } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import type { SourceType } from '@/types/kagerou'

export function AddSourceMenu({ onChoose }: { onChoose: (type: SourceType) => void }) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button className="h-10 gap-2 bg-lavender px-4 text-[13px] font-semibold text-ink hover:bg-lavender-hi" type="button"><Plus aria-hidden="true" className="size-4" />Add source</Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-[294px] border-hairline bg-raised p-0 text-body">
        <DropdownMenuLabel className="border-b border-hairline px-4 py-3"><span className="type-eyebrow block">Add a source</span><span className="mt-1 block text-[12px] font-normal text-body">Choose how Kagerou receives VPNs.</span></DropdownMenuLabel>
        <DropdownMenuItem className="items-start gap-3 rounded-none px-4 py-3" onSelect={() => onChoose('url')}>
          <span className="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-md bg-selected text-lavender-hi"><Rss aria-hidden="true" className="size-4" /></span>
          <span className="min-w-0"><span className="block text-[13px] font-medium text-primary">Subscription URL</span><span className="mt-1 block text-[11px] leading-4 text-muted-copy">Import and refresh VPNs from a remote URL.</span></span>
        </DropdownMenuItem>
        <DropdownMenuSeparator className="m-0" />
        <DropdownMenuItem className="items-start gap-3 rounded-none px-4 py-3" onSelect={() => onChoose('key')}>
          <span className="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-md bg-selected text-good"><KeyRound aria-hidden="true" className="size-4" /></span>
          <span className="min-w-0"><span className="block text-[13px] font-medium text-primary">Single key</span><span className="mt-1 block text-[11px] leading-4 text-muted-copy">Add one key as a single local VPN.</span></span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
