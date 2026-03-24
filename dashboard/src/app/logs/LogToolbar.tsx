"use client";

import { LogLevel } from "@/lib/log";
import { Search, Filter, Trash2, Wifi, WifiOff } from "lucide-react";
import { LOG_LEVELS } from "@/lib/log/utils";
import { LiveSwitch } from "@/components/LiveSwitch";

interface LogToolbarProps {
  query: any;
  updateFilter: (updates: any) => void;
  hasFilters: boolean;
  isLive: boolean;
  setIsLive: (val: boolean) => void;
  wsEnabled: boolean;
  wsStatus: string;
}

export function LogToolbar({
  query,
  updateFilter,
  hasFilters,
  isLive,
  setIsLive,
  wsEnabled,
  wsStatus
}: LogToolbarProps) {
  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h1 className="text-2xl font-bold text-text-main">System Logs</h1>
          {isLive && (
            <div className="flex items-center gap-2 rounded-full bg-emerald-50 px-2.5 py-0.5 text-[10px] font-bold text-emerald-600 border border-emerald-100 uppercase">
              <span className="relative flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
              </span>
              Live
            </div>
          )}
        </div>
        <div className="flex items-center gap-3">
          {isLive && (
             <div className="flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wider text-slate-400 mr-2">
                {wsStatus === 'open' ? <Wifi className="h-3 w-3 text-emerald-500" /> : <WifiOff className="h-3 w-3 text-rose-500" />}
                {wsStatus}
             </div>
          )}
          <button
            onClick={() =>
              updateFilter({
                level: "All",
                search: "",
                target: "",
                from: "",
                to: "",
              })
            }
            disabled={!hasFilters}
            className={`btn-ghost-sm gap-1.5 px-3 transition-all ${
              hasFilters ? "opacity-100" : "pointer-events-none opacity-0"
            }`}
          >
            <Trash2 className="h-3.5 w-3.5" />
            Clear
          </button>
          <div className="mx-1 h-6 w-px bg-slate-100" />
          <LiveSwitch
            isActive={isLive && wsEnabled}
            onToggle={setIsLive}
            disabled={!wsEnabled}
          />
        </div>
      </div>

      <div className="card-container grid grid-cols-1 gap-3 border border-slate-200 bg-white p-3 sm:grid-cols-2 lg:grid-cols-5 lg:gap-4 shadow-sm">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
          <input
            type="text"
            placeholder="Search logs..."
            className="input-base py-1.5 pl-9 h-9 text-sm"
            value={query.search}
            onChange={(e) => updateFilter({ search: e.target.value })}
          />
        </div>

        <div className="relative">
          <Filter className="absolute left-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-slate-400" />
          <select
            value={query.level}
            onChange={(e) => updateFilter({ level: e.target.value })}
            className="input-base py-1.5 pl-9 h-9 text-sm appearance-none"
          >
            {LOG_LEVELS.map((level) => (
              <option key={level} value={level}>
                {level === "All" ? "All Levels" : level}
              </option>
            ))}
          </select>
        </div>

        <input
          type="text"
          placeholder="Target (e.g. gateway)"
          className="input-base py-1.5 h-9 text-sm"
          value={query.target}
          onChange={(e) => updateFilter({ target: e.target.value })}
        />

        <input
          type="datetime-local"
          className="input-base py-1.5 h-9 text-sm"
          value={query.from}
          onChange={(e) => updateFilter({ from: e.target.value })}
        />

        <input
          type="datetime-local"
          className="input-base py-1.5 h-9 text-sm"
          value={query.to}
          onChange={(e) => updateFilter({ to: e.target.value })}
        />
      </div>
    </div>
  );
}
