/** Mirrors gateway/src/db/models/log.rs */

export interface LogEntry {
  id: number
  timestamp: string
  level: string
  target: string
  message: string
  fields: string
}
