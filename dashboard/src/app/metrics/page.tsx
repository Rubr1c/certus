"use client";

import { useMetricFilter } from "@/hooks/use-metric-filter";
import { VALID_INTERVALS } from "@/lib/types";
import { StatusCodesChart } from "@/components/charts/StatusCodesChart";
import { LatencyExtremesChart } from "@/components/charts/LatencyExtremesChart";
import { BandwidthChart } from "@/components/charts/BandwidthChart";
import { HttpMethodsChart } from "@/components/charts/HttpMethodsChart";
import { Suspense } from "react";

function MetricsContent() {
  const { filter, updateFilter } = useMetricFilter();

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-text-main">Detailed Metrics</h1>
          <p className="text-sm text-text-muted mt-1">Deep dive into gateway traffic and performance</p>
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

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <StatusCodesChart />
        <HttpMethodsChart />
      </div>

      <div className="grid grid-cols-1 gap-6">
        <LatencyExtremesChart />
        <BandwidthChart />
      </div>
    </div>
  );
}

export default function MetricsPage() {
  return (
    <Suspense fallback={<div className="animate-pulse bg-slate-100 h-96 rounded-xl" />}>
      <MetricsContent />
    </Suspense>
  );
}
