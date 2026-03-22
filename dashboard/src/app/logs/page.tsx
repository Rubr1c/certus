"use client";

import { useState, Fragment } from "react";
import { useQueryStates, parseAsInteger, parseAsString } from "nuqs";
import { useArgs } from "@/hooks/use-args";
import { useLogs } from "@/hooks/use-logs";
import { LiveSwitch } from "@/components/LiveSwitch";
import { WEB_SOCKET_TYPE } from "@/lib/args";
import {
  LOG_LEVELS,
  getLevelBadgeClass,
  formatFieldsJson,
} from "@/lib/log/utils";

const PER_PAGE = 25;

const inputBase =
  "h-10 rounded-lg border border-slate-200 px-3 py-2 text-sm text-text-main placeholder-text-muted focus:border-ocean-500 focus:outline-none focus:ring-1 focus:ring-ocean-500 transition-all bg-white hover:border-slate-300";

const logQuerySchema = {
  page: parseAsInteger.withDefault(1),
  level: parseAsString.withDefault("All"),
  search: parseAsString.withDefault(""),
  target: parseAsString.withDefault(""),
  from: parseAsString.withDefault(""),
  to: parseAsString.withDefault(""),
};

export default function LogsPage() {
  const [query, setQuery] = useQueryStates(logQuerySchema, {
    shallow: false,
    history: "push",
  });

  const [expandedId, setExpandedId] = useState<number | null>(null);
  const [isLive, setIsLive] = useState(false);

  const { data: args } = useArgs();
  const wsEnabled = Boolean(args?.ws?.includes(WEB_SOCKET_TYPE.Logs));

  const { logs, isLoading, error } = useLogs({
    query: { ...query, per_page: PER_PAGE },
    isLive,
    wsEnabled,
  });

  const updateFilter = (updates: Partial<typeof query>) => {
    setQuery({ ...updates, page: 1 });
  };

  const hasFilters =
    query.level !== "All" ||
    query.search !== "" ||
    query.target !== "" ||
    query.from !== "" ||
    query.to !== "";

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold text-text-main">Logs</h1>
        <div className="flex items-center gap-3">
          <button
            onClick={() =>
              setQuery({
                page: 1,
                level: "All",
                search: "",
                target: "",
                from: "",
                to: "",
              })
            }
            disabled={!hasFilters}
            className={`flex h-8 items-center gap-1.5 rounded-lg border px-3 text-xs font-semibold transition-all ${
              hasFilters
                ? "border-slate-200 bg-white text-slate-600 hover:bg-slate-50 hover:text-slate-900 shadow-sm"
                : "border-transparent bg-transparent text-slate-300 opacity-0 pointer-events-none"
            }`}
          >
            Clear Filters
          </button>
          <div className="h-6 w-px bg-slate-100 mx-1" />
          <LiveSwitch
            isActive={isLive && wsEnabled}
            onToggle={(val) => {
              setIsLive(val);
              if (val) setExpandedId(null);
            }}
            disabled={!wsEnabled}
            tooltipMessage={
              wsEnabled
                ? undefined
                : "Enable logs WebSocket in the gateway configuration"
            }
          />
        </div>
      </div>

      <div className="overflow-hidden rounded-2xl border border-slate-100 bg-surface shadow-soft">
        <div className="grid grid-cols-1 gap-3 border-b border-slate-100 bg-slate-50/30 px-4 py-4 sm:grid-cols-2 lg:grid-cols-5 lg:gap-4">
          <input
            type="text"
            placeholder="Search messages..."
            value={query.search}
            onChange={(e) => updateFilter({ search: e.target.value })}
            className={inputBase}
          />
          <select
            value={query.level}
            onChange={(e) => updateFilter({ level: e.target.value })}
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
            value={query.target}
            onChange={(e) => updateFilter({ target: e.target.value })}
            className={inputBase}
          />
          <input
            type="datetime-local"
            value={query.from}
            onChange={(e) => updateFilter({ from: e.target.value })}
            disabled={isLive}
            className={`${inputBase} font-mono disabled:opacity-50`}
          />
          <input
            type="datetime-local"
            value={query.to}
            onChange={(e) => updateFilter({ to: e.target.value })}
            disabled={isLive}
            className={`${inputBase} font-mono disabled:opacity-50`}
          />
        </div>

        <div className="p-0">
          {error && (
            <div className="m-4 rounded-lg border border-red-100 bg-red-50 px-4 py-3 text-sm text-red-600">
              {error instanceof Error ? error.message : "Failed to load logs"}
            </div>
          )}

          <div className="min-h-[400px] overflow-x-auto">
            {!isLive && isLoading ? (
              <div className="flex h-[400px] items-center justify-center text-sm text-text-muted">
                <div className="flex flex-col items-center gap-2">
                  <div className="h-5 w-5 animate-spin rounded-full border-2 border-ocean-500 border-t-transparent" />
                  Loading logs...
                </div>
              </div>
            ) : logs.length === 0 ? (
              <div className="flex h-[400px] flex-col items-center justify-center text-center text-sm text-text-muted">
                <div className="mb-2 text-slate-300">
                  <svg
                    className="h-12 w-12"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={1}
                      d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                    />
                  </svg>
                </div>
                {isLive ? "Listening for logs..." : "No logs found"}
              </div>
            ) : (
              <table className="w-full border-collapse">
                <thead>
                  <tr className="border-b border-slate-100 bg-slate-50/30">
                    <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wider text-slate-400">
                      Timestamp
                    </th>
                    <th className="w-24 px-6 py-3 text-left text-xs font-semibold uppercase tracking-wider text-slate-400">
                      Level
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wider text-slate-400">
                      Message
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-50">
                  {logs.map((log) => (
                    <Fragment key={log.id}>
                      <tr
                        onClick={() =>
                          setExpandedId(expandedId === log.id ? null : log.id)
                        }
                        className={`group cursor-pointer select-none transition-colors hover:bg-slate-50/80 ${
                          expandedId === log.id ? "bg-ocean-50/50" : ""
                        }`}
                      >
                        <td className="whitespace-nowrap px-6 py-3 font-mono text-xs text-text-muted">
                          {log.timestamp}
                        </td>
                        <td className="px-6 py-3">
                          <span className={getLevelBadgeClass(log.level)}>
                            {log.level}
                          </span>
                        </td>
                        <td className="px-6 py-3 font-mono text-sm text-text-main group-hover:text-ocean-600">
                          {log.message}
                        </td>
                      </tr>
                      {expandedId === log.id && (
                        <tr className="bg-slate-50/30">
                          <td colSpan={3} className="px-6 py-4">
                            <div
                              className="animate-in fade-in slide-in-from-top-1 select-none space-y-3 duration-200"
                              onClick={(e) => e.stopPropagation()}
                            >
                              <div className="flex items-center gap-2 text-xs font-medium text-slate-500">
                                <span className="rounded bg-slate-100 px-1.5 py-0.5 font-mono">
                                  {log.target}
                                </span>
                              </div>
                              {(log.fields || "{}") !== "{}" && (
                                <div className="overflow-hidden rounded-xl bg-slate-900 shadow-lg">
                                  <div className="border-b border-slate-800 bg-slate-800/50 px-4 py-2 text-[10px] font-bold uppercase tracking-widest text-slate-500">
                                    Fields
                                  </div>
                                  <pre className="overflow-x-auto p-4 font-mono text-xs leading-relaxed text-slate-300">
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

          {!isLive && logs.length > 0 && (
            <div className="flex items-center justify-between border-t border-slate-100 px-6 py-4 bg-white/50">
              <span className="text-xs font-medium text-slate-500">
                Page <span className="text-slate-900">{query.page}</span>
              </span>
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={() => setQuery({ page: Math.max(1, query.page - 1) })}
                  disabled={query.page <= 1 || isLoading}
                  className="inline-flex h-8 cursor-pointer items-center rounded-lg border border-slate-200 px-3 text-xs font-semibold text-slate-600 transition-all hover:bg-slate-50 hover:text-slate-900 disabled:cursor-not-allowed disabled:opacity-40"
                >
                  Previous
                </button>
                <button
                  type="button"
                  onClick={() => setQuery({ page: query.page + 1 })}
                  disabled={logs.length < PER_PAGE || isLoading}
                  className="inline-flex h-8 cursor-pointer items-center rounded-lg border border-slate-200 px-3 text-xs font-semibold text-slate-600 transition-all hover:bg-slate-50 hover:text-slate-900 disabled:cursor-not-allowed disabled:opacity-40"
                >
                  Next
                </button>
              </div>
            </div>
          )}

          {isLive && (
            <div className="border-t border-slate-100 px-6 py-3 bg-emerald-50/30">
              <div className="flex items-center gap-2 text-xs font-medium text-emerald-700">
                <span className="relative flex h-2 w-2">
                  <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75"></span>
                  <span className="relative inline-flex h-2 w-2 rounded-full bg-emerald-500"></span>
                </span>
                Live stream · {logs.length} logs buffered
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
