import { Moon, Sun } from 'lucide-react'

import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { Theme } from '@/types/kagerou'

export function ThemeToggle({ value, onChange }: { value: Theme; onChange: (value: Theme) => void }) {
  return (
    <ToggleGroup aria-label="Theme" className="rounded-lg bg-surface p-1" onValueChange={(next) => { if (next) onChange(next as Theme) }} type="single" value={value}>
      <ToggleGroupItem className="h-8 gap-1.5 rounded-md px-4 text-[13px] text-muted-copy data-[state=on]:bg-selected data-[state=on]:text-primary" value="dark"><Moon aria-hidden="true" className="size-3.5" />Dark</ToggleGroupItem>
      <ToggleGroupItem className="h-8 gap-1.5 rounded-md px-4 text-[13px] text-muted-copy data-[state=on]:bg-selected data-[state=on]:text-primary" value="light"><Sun aria-hidden="true" className="size-3.5" />Light</ToggleGroupItem>
    </ToggleGroup>
  )
}
