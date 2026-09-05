import type { LogEntry } from '@/types/kagerou'
import { cn } from '@/lib/utils'

const levelClasses: Record<LogEntry['level'], string> = {
  INFO: 'text-[#b8b1cf]',
  WARN: 'text-warn',
  ERROR: 'text-bad',
}

function HighlightedMessage({ message, query }: { message: string; query: string }) {
  const needle = query.trim()
  if (!needle) return message
  const parts = message.split(new RegExp(`(${needle.replace(/[.*+?^${}()|[\\]\\]/g, '\\$&')})`, 'ig'))
  return parts.map((part, index) =>
    part.toLowerCase() === needle.toLowerCase() ? (
      <mark className="bg-transparent text-inherit underline decoration-lavender decoration-2 underline-offset-3" key={`${part}-${index}`}>
        {part}
      </mark>
    ) : (
      part
    ),
  )
}

export function LogRow({ entry, query }: { entry: LogEntry; query: string }) {
  return (
    <div className="log-row-grid grid items-baseline gap-4 font-mono text-[13px] leading-[1.55] max-[760px]:grid-cols-1 max-[760px]:gap-0 max-[760px]:py-2">
      <span className="whitespace-nowrap text-quiet max-[760px]:text-[11px]">{entry.timestamp}</span>
      <span className={cn('font-medium', levelClasses[entry.level])}>{entry.level}</span>
      <span className="min-w-0 break-words text-body">
        <HighlightedMessage message={entry.message} query={query} />
      </span>
    </div>
  )
}
