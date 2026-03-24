"use client";

import { useState, Fragment } from "react";
import { useQueryStates, parseAsInteger, parseAsString } from "nuqs";
import { useArgs } from "@/hooks/use-args";
import { useLogs } from "@/hooks/use-logs";
import { WEB_SOCKET_TYPE } from "@/lib/types";
import { getLevelBadgeClass, formatFieldsJson } from "@/lib/log/utils";
import { LogToolbar } from "./LogToolbar";
import { ChevronRight, ChevronDown } from "lucide-react";

const PER_PAGE = 50;

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

  const [expandedId, setExpandedId] = useState<string | number | null>(null);
  const [isLive, setIsLive] = useState(false);

  const { data: args } = useArgs();
  const wsEnabled = Boolean(args?.ws?.includes(WEB_SOCKET_TYPE.Logs));

  const { logs, isLoading, error, wsStatus } = useLogs({
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
      <LogToolbar 
        query={query}
        updateFilter={updateFilter}
        hasFilters={hasFilters}
        isLive={isLive}
        setIsLive={(val) => {
           setIsLive(val);
           if (val) setExpandedId(null);
        }}
        wsEnabled={wsEnabled}
        wsStatus={wsStatus}
      />

      <div className="card-container overflow-hidden shadow-sm border border-slate-200 bg-white">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm border-collapse">
            <thead className="bg-slate-50/50 border-b border-slate-100">
              <tr>
                <th className="w-12 px-4 py-4"></th>
                <th className="px-4 py-4 font-bold text-slate-500 uppercase tracking-wider text-[11px]">Level</th>
                <th className="px-4 py-4 font-bold text-slate-500 uppercase tracking-wider text-[10px]">Time</th>
                <th className="px-4 py-4 font-bold text-slate-500 uppercase tracking-wider text-[11px]">Target</th>
                <th className="px-4 py-4 font-bold text-slate-500 uppercase tracking-wider text-[11px]">Message</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {isLoading ? (
                [1, 2, 3, 4, 5].map((i) => (
                  <tr key={i} className="animate-pulse">
                    <td colSpan={5} className="h-14 bg-slate-50/30"></td>
                  </tr>
                ))
              ) : logs.length === 0 ? (
                <tr>
                  <td colSpan={5} className="py-20 text-center text-slate-400">
                    No logs found matching your criteria.
                  </td>
                </tr>
              ) : (
                logs.map((log) => (
                  <Fragment key={log.id}>
                    <tr
                      className={`group cursor-pointer transition-colors hover:bg-slate-50/50 ${
                        expandedId === log.id ? "bg-slate-50 shadow-inner" : ""
                      }`}
                      onClick={() =>
                        setExpandedId(expandedId === log.id ? null : log.id)
                      }
                    >
                      <td className="px-4 py-4">
                        {expandedId === log.id ? (
                          <ChevronDown className="h-4.5 w-4.5 text-slate-400 mx-auto" />
                        ) : (
                          <ChevronRight className="h-4.5 w-4.5 text-slate-400 group-hover:text-ocean-500 mx-auto transition-transform" />
                        )}
                      </td>
                      <td>
                        <span className={`badge text-[11px] uppercase font-bold tracking-tight px-3 py-1 ${getLevelBadgeClass(log.level)}`}>
                          {log.level}
                        </span>
                      </td>
                      <td className="px-4 py-4 text-slate-400 font-mono text-xs whitespace-nowrap">
                        {log.timestamp.split('T')[1].split('.')[0]}
                      </td>
                      <td className="px-4 py-4">
                        <span className="rounded-md bg-slate-100 px-2.5 py-1 text-[11px] font-bold text-slate-600 border border-slate-200/60 uppercase tracking-wide">
                          {log.target}
                        </span>
                      </td>
                      <td className="px-4 py-4 text-slate-900 text-sm font-semibold truncate max-w-xl">
                        {log.message}
                      </td>
                    </tr>
                    {expandedId === log.id && (
                      <tr className="bg-slate-50/80 border-y border-slate-200/50">
                        <td colSpan={5} className="px-16 py-6">
                          <div className="space-y-6">
                             {log.fields && Object.keys(log.fields).length > 0 ? (
                               <div>
                                  <h4 className="text-[11px] font-bold uppercase tracking-widest text-slate-400 mb-3">Structured Metadata</h4>
                                  <div className="rounded-xl bg-slate-900 border border-slate-800 p-5 shadow-lg">
                                     <pre className="text-xs text-slate-300 font-mono whitespace-pre-wrap overflow-x-auto">
                                       {formatFieldsJson(log.fields)}
                                     </pre>
                                  </div>
                               </div>
                             ) : (
                               <div className="py-4 text-center text-slate-400 text-xs font-medium">
                                 No additional metadata for this entry.
                               </div>
                             )}
                             <div className="text-[11px] text-slate-400 font-mono border-t border-slate-200 pt-4">
                                Full Precision Timestamp: {log.timestamp}
                             </div>
                          </div>
                        </td>
                      </tr>
                    )}
                  </Fragment>
                ))
              )}
            </tbody>
          </table>
        </div>
        
        {!isLive && !isLoading && logs.length > 0 && (
          <div className="flex items-center justify-between border-t border-slate-100 bg-slate-50/30 px-6 py-4">
            <span className="text-xs text-slate-400 font-bold uppercase tracking-wider">
              Page {query.page}
            </span>
            <div className="flex gap-3">
              <button
                disabled={query.page <= 1}
                onClick={() => setQuery({ page: query.page - 1 })}
                className="btn-ghost-sm h-9 px-4 font-bold uppercase tracking-tight"
              >
                Previous
              </button>
              <button
                disabled={logs.length < PER_PAGE}
                onClick={() => setQuery({ page: query.page + 1 })}
                className="btn-ghost-sm h-9 px-4 font-bold uppercase tracking-tight"
              >
                Next
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
