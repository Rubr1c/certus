"use client";

import { useMetricFilter } from "@/hooks/use-metric-filter";
import { VALID_INTERVALS } from "@/lib/types";
import { CacheOperationsChart } from "@/components/charts/CacheOperationsChart";
import { HitRateTrendChart } from "@/components/charts/HitRateTrendChart";
import { CacheHitRateKPI } from "@/components/charts/CacheHitRateKPI";
import { Suspense } from "react";

function CacheMetricsContent() {
  const { filter, updateFilter } = useMetricFilter();

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-text-main">Cache Metrics</h1>
          <p className="text-sm text-text-muted mt-1">Monitor caching efficiency and volume</p>
        </div>
        
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

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="md:col-span-1">
          <CacheHitRateKPI />
        </div>
        <div className="md:col-span-2">
          <HitRateTrendChart />
        </div>
      </div>

      <div className="grid grid-cols-1 gap-6">
        <CacheOperationsChart />
      </div>
    </div>
  );
}

export default function CacheMetricsPage() {
  return (
    <Suspense fallback={<div className="animate-pulse bg-slate-100 h-96 rounded-xl" />}>
      <CacheMetricsContent />
    </Suspense>
  );
}
