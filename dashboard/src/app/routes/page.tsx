"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/api";
import { Route, Activity, RefreshCw, Search } from "lucide-react";
import { useState, Suspense } from "react";
import { UpstreamLoadChart } from "@/components/charts/UpstreamLoadChart";
import { UpstreamHealthChart } from "@/components/charts/UpstreamHealthChart";
import { useMetricFilter } from "@/hooks/use-metric-filter";
import { VALID_INTERVALS } from "@/lib/types";

function RoutesContent() {
  const [search, setSearch] = useState("");
  const { filter, updateFilter } = useMetricFilter();

  const { data: routes = [], isLoading, refetch, isRefetching } = useQuery({
    queryKey: ["routes"],
    queryFn: () => api.routes.get(),
  });

  const filteredRoutes = routes.filter((r) =>
    r.path.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-900 flex items-center gap-2">
            <Route className="h-6 w-6 text-ocean-500" />
            Route Performance
          </h1>
          <p className="mt-1 text-sm text-slate-500">
            Monitor upstream health and traffic distribution
          </p>
        </div>

        <div className="flex items-center gap-3">
          <select
            value={filter.interval}
            onChange={(e) => updateFilter({ interval: e.target.value })}
            className="input-base py-1.5 h-9 text-sm w-32"
          >
            {VALID_INTERVALS.map((int) => (
              <option key={int} value={int}>
                Int: {int}
              </option>
            ))}
          </select>
          <button 
            onClick={() => refetch()}
            disabled={isRefetching}
            className="btn-ghost-sm h-9 px-3 gap-2"
          >
            <RefreshCw className={`h-4 w-4 ${isRefetching ? 'animate-spin' : ''}`} />
            Refresh
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <UpstreamLoadChart />
        <UpstreamHealthChart />
      </div>

      <div className="overflow-hidden rounded-xl border border-slate-100 bg-white shadow-sm">
        <div className="border-b border-slate-100 bg-slate-50/50 p-4">
          <div className="relative max-w-sm">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
            <input
              type="text"
              placeholder="Filter routes..."
              className="input-base py-1.5 pl-9 h-9 text-sm w-full"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm">
            <thead className="bg-slate-50/30 text-xs font-bold uppercase tracking-wider text-slate-400">
              <tr>
                <th className="px-6 py-4">Path</th>
                <th className="px-6 py-4">Upstreams</th>
                <th className="px-6 py-4">Status</th>
                <th className="px-6 py-4 text-right">Connections</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {isLoading ? (
                [1, 2, 3].map(i => <tr key={i} className="animate-pulse h-16"><td colSpan={4}></td></tr>)
              ) : filteredRoutes.length === 0 ? (
                <tr><td colSpan={4} className="py-10 text-center text-slate-400">No routes found</td></tr>
              ) : (
                filteredRoutes.map((route) => {
                  const healthy = route.upstreams.filter(u => u.healthy).length;
                  const total = route.upstreams.length;
                  const active = route.upstreams.reduce((acc, u) => acc + u.active_connections, 0);
                  
                  return (
                    <tr key={route.path} className="hover:bg-slate-50/50 transition-colors">
                      <td className="px-6 py-4 font-mono font-medium text-slate-900">{route.path}</td>
                      <td className="px-6 py-4 text-slate-600">{total} hosts</td>
                      <td className="px-6 py-4">
                        <div className="flex items-center gap-2">
                          <div className={`h-1.5 w-1.5 rounded-full ${healthy === total ? 'bg-emerald-500' : healthy > 0 ? 'bg-amber-500' : 'bg-rose-500'}`} />
                          <span className="text-xs font-medium">
                            {healthy}/{total} Healthy
                          </span>
                        </div>
                      </td>
                      <td className="px-6 py-4 text-right">
                        <span className="inline-flex items-center gap-1 text-xs font-semibold text-slate-700">
                          <Activity className="h-3 w-3 text-slate-400" />
                          {active}
                        </span>
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

export default function RoutesPage() {
  return (
    <Suspense fallback={<div className="animate-pulse bg-slate-50 h-96 rounded-xl" />}>
      <RoutesContent />
    </Suspense>
  );
}
