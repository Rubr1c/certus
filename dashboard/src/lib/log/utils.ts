import { type LogEntry } from "./index";

export const LOG_LEVELS = ["All", "DEBUG", "INFO", "WARN", "ERROR"] as const;
export type LogLevel = (typeof LOG_LEVELS)[number];

export const LEVEL_BADGE: Record<string, string> = {
  ERROR: "badge badge-error",
  WARN: "badge badge-warn",
  INFO: "badge badge-info",
  DEBUG: "badge badge-neutral",
};

export function getLevelBadgeClass(level: string): string {
  return LEVEL_BADGE[level.toUpperCase()] ?? "badge badge-neutral";
}

export function formatFieldsJson(fields: string | Record<string, any>): string {
  if (typeof fields === "object") {
    return JSON.stringify(fields, null, 2);
  }
  try {
    const parsed = JSON.parse(fields);
    return JSON.stringify(parsed, null, 2);
  } catch {
    return fields;
  }
}

/**
 * Normalizes log entries coming from WebSocket (LogEntryDTO)
 * to match the DB LogEntry structure used in the UI.
 */
export function normalizeWSLog(log: any, index: number): LogEntry {
  let fields = "{}";
  if (typeof log.fields === "object") {
    fields = JSON.stringify(log.fields || {});
  } else if (typeof log.fields === "string") {
    fields = log.fields;
  }

  return {
    ...log,
    id: -(index + 1), // Temporary negative ID for unique keys
    fields,
  };
}
