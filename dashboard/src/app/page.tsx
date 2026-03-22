"use client";

import { GlobalTrafficChart } from "@/components/charts/GlobalTrafficChart";
import { GlobalLatencyChart } from "@/components/charts/GlobalLatencyChart";
import { TopRoutesChart } from "@/components/charts/TopRoutesChart";
import { CacheHitRateKPI } from "@/components/charts/CacheHitRateKPI";
import { useMetricFilter } from "@/hooks/use-metric-filter";
import { VALID_INTERVALS } from "@/lib/types";
import { Suspense } from "react";

function HomeContent() {
  const { filter, updateFilter } = useMetricFilter();

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold text-text-main">Overview</h1>
        
        <div className="flex items-center gap-3">
          <select
            value={filter.interval}
            onChange={(e) => updateFilter({ interval: e.target.value })}
            className="input-base py-1.5 h-9"
          >
            {VALID_INTERVALS.map((int) => (
              <option key={int} value={int}>
                Interval: {int}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 items-start">
        <div className="md:col-span-2 space-y-6">
          <GlobalTrafficChart />
          <GlobalLatencyChart />
        </div>
        <div className="md:col-span-1 space-y-6">
          <CacheHitRateKPI />
          <TopRoutesChart />
        </div>
      </div>
    </div>
  );
}

export default function Home() {
  return (
    <Suspense fallback={<div className="animate-pulse bg-slate-100 h-96 rounded-xl" />}>
      <HomeContent />
    </Suspense>
  );
}
