import type { LogEntry, LogLevel } from '@/types/kagerou'

import type { Slice } from '../shared'

/** The viewer keeps a window, not a history: the core's own ring buffer is
 * the same size, and everything before it is gone anyway. */
const MAX_LOG_ENTRIES = 500

let sequence = 0

const detectLogLevel = (line: string): LogLevel => {
  if (/\berror\b/i.test(line)) return 'ERROR'
  if (/\bwarn(ing)?\b/i.test(line)) return 'WARN'
  return 'INFO'
}

/** Timestamped here rather than by the core: sing-box writes its own stamp
 * into the line, and this one is when the window heard about it. */
export const toLogEntry = (line: string): LogEntry => ({
  id: `log-${Date.now()}-${sequence++}`,
  timestamp: new Date().toISOString(),
  level: detectLogLevel(line),
  message: line,
})

export const appendLog = (logs: LogEntry[], line: string) => [
  ...logs.slice(-(MAX_LOG_ENTRIES - 1)),
  toLogEntry(line),
]

export interface LogsSlice {
  logs: LogEntry[]
}

export const createLogsSlice: Slice<LogsSlice> = () => ({
  logs: [],
})
