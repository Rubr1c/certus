"use client";

import { useState, Fragment } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/api";
import type { LogEntry } from "@/lib/log";
import type { LogQuery as LibLogQuery } from "@/lib/query";

interface LogQuery extends LibLogQuery {
  page: number;
  per_page: number;
}

const LOG_LEVELS = ["All", "DEBUG", "INFO", "WARN", "ERROR"] as const;
const PER_PAGE = 20;

const META_LOG_MESSAGE = "Querying stored logs";

const inputBase =
  "rounded-lg border border-slate-200 px-3 py-2 text-sm text-text-main placeholder-text-muted focus:border-ocean-500 focus:outline-none focus:ring-1 focus:ring-ocean-500";

const LEVEL_BADGE: Record<string, string> = {
  ERROR: "badge badge-error",
  WARN: "badge badge-warn",
  INFO: "badge badge-info",
  DEBUG: "badge badge-neutral",
};

function getLevelBadgeClass(level: string): string {
  return LEVEL_BADGE[level.toUpperCase()] ?? "badge badge-neutral";
}

function formatFieldsJson(fields: string): string {
  try {
    const parsed = JSON.parse(fields);
    return JSON.stringify(parsed, null, 2);
  } catch {
    return fields;
  }
}

function buildParams(query: LogQuery): Record<string, string | number> {
  const params: Record<string, string | number> = {
    page: query.page,
    per_page: query.per_page,
  };
  if (query.from) params.from = query.from;
  if (query.to) params.to = query.to;
  if (query.level && query.level !== "All") params.level = query.level;
  if (query.target) params.target = query.target;
  if (query.search) params.search = query.search;
  return params;
}

function filterMetaLogs(logs: LogEntry[]): LogEntry[] {
  return logs.filter((log) => log.message !== META_LOG_MESSAGE);
}

export default function LogsPage() {
  const [query, setQuery] = useState<LogQuery>({
    page: 1,
    per_page: PER_PAGE,
  });
  const [expandedId, setExpandedId] = useState<number | null>(null);

  const { data: rawLogs = [], isLoading, error } = useQuery({
    queryKey: ["logs", query],
    queryFn: () => api.logs.get(buildParams(query)),
  });

  const logs = filterMetaLogs(rawLogs);

  const handleFilter = (key: keyof LogQuery, value: string | number) => {
    setQuery((q) => {
      const next = { ...q, [key]: value, page: 1 };
      if (value === "" || value === "All") {
        delete next[key as keyof typeof next];
      }
      return next;
    });
  };

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold text-text-main">Logs</h1>
      <div className="flex flex-col gap-4 rounded-2xl border border-slate-50 bg-surface p-6 shadow-soft">
        <div className="flex flex-wrap items-center gap-3">
          <input
            type="text"
            placeholder="Search logs..."
            value={query.search ?? ""}
            onChange={(e) => handleFilter("search", e.target.value)}
            className={`${inputBase} w-48`}
          />
          <select
            value={query.level ?? "All"}
            onChange={(e) => handleFilter("level", e.target.value)}
            className={inputBase}
          >
            {LOG_LEVELS.map((l) => (
              <option key={l} value={l}>
                {l}
              </option>
            ))}
          </select>
          <input
            type="text"
            placeholder="Target (module)"
            value={query.target ?? ""}
            onChange={(e) => handleFilter("target", e.target.value)}
            className={`${inputBase} w-40`}
          />
          <input
            type="datetime-local"
            value={query.from ?? ""}
            onChange={(e) => handleFilter("from", e.target.value)}
            className={`${inputBase} font-mono`}
          />
          <input
            type="datetime-local"
            value={query.to ?? ""}
            onChange={(e) => handleFilter("to", e.target.value)}
            className={`${inputBase} font-mono`}
          />
        </div>

        {error && (
          <div className="rounded-lg border border-red-100 bg-red-50 px-4 py-2 text-sm text-red-600">
            {error instanceof Error ? error.message : "Failed to load logs"}
          </div>
        )}

        <div className="min-h-[300px] overflow-x-auto">
          {isLoading ? (
            <div className="flex items-center justify-center py-12 text-sm text-text-muted">
              Loading logs...
            </div>
          ) : logs.length === 0 ? (
            <div className="py-12 text-center text-sm text-text-muted">
              No logs found
            </div>
          ) : (
            <table className="w-full min-w-[500px]">
              <thead>
                <tr className="border-b border-slate-200">
                  <th className="px-4 py-2 text-left text-xs font-semibold uppercase tracking-wider text-text-muted">
                    Timestamp
                  </th>
                  <th className="w-24 px-4 py-2 text-left text-xs font-semibold uppercase tracking-wider text-text-muted">
                    Level
                  </th>
                  <th className="px-4 py-2 text-left text-xs font-semibold uppercase tracking-wider text-text-muted">
                    Message
                  </th>
                </tr>
              </thead>
              <tbody>
                {logs.map((log) => (
                  <Fragment key={log.id}>
                    <tr
                      onClick={() =>
                        setExpandedId(expandedId === log.id ? null : log.id)
                      }
                      className="cursor-pointer select-none border-b border-slate-50 transition-colors hover:bg-ocean-50"
                    >
                      <td className="whitespace-nowrap px-4 py-2 font-mono text-sm text-text-main">
                        {log.timestamp}
                      </td>
                      <td className="px-4 py-2">
                        <span className={getLevelBadgeClass(log.level)}>
                          {log.level}
                        </span>
                      </td>
                      <td className="px-4 py-2 font-mono text-sm text-text-main">
                        {log.message}
                      </td>
                    </tr>
                    {expandedId === log.id && (
                      <tr className="bg-ocean-50/30">
                        <td colSpan={3} className="px-4 py-3">
                          <div
                            className="select-none space-y-2"
                            onClick={(e) => e.stopPropagation()}
                          >
                            {log.target && (
                              <div className="font-mono text-sm text-text-main">
                                Module: {log.target}
                              </div>
                            )}
                            {(log.fields || "{}") !== "{}" && (
                              <div className="overflow-x-auto rounded-lg bg-slate-900 p-4 font-mono text-xs text-slate-300">
                                <pre className="whitespace-pre-wrap break-words">
                                  {formatFieldsJson(log.fields || "{}")}
                                </pre>
                              </div>
                            )}
                          </div>
                        </td>
                      </tr>
                    )}
                  </Fragment>
                ))}
              </tbody>
            </table>
          )}
        </div>

        <div className="flex items-center justify-between border-t border-slate-100 pt-4">
          <span className="text-sm text-text-muted">Page {query.page}</span>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() =>
                setQuery((q) => ({ ...q, page: Math.max(1, q.page - 1) }))
              }
              disabled={query.page <= 1 || isLoading}
              className="cursor-pointer rounded-lg border border-slate-200 px-3 py-1.5 text-sm font-medium text-text-main transition-colors hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
            >
              Previous
            </button>
            <button
              type="button"
              onClick={() => setQuery((q) => ({ ...q, page: q.page + 1 }))}
              disabled={logs.length < PER_PAGE || isLoading}
              className="cursor-pointer rounded-lg border border-slate-200 px-3 py-1.5 text-sm font-medium text-text-main transition-colors hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
            >
              Next
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
