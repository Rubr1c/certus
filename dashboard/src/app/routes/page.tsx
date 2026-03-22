"use client";

import { useMetricFilter } from "@/hooks/use-metric-filter";
import { VALID_INTERVALS } from "@/lib/types";
import { UpstreamLoadChart } from "@/components/charts/UpstreamLoadChart";
import { UpstreamHealthChart } from "@/components/charts/UpstreamHealthChart";

export default function RoutesPage() {
  const { filter, updateFilter } = useMetricFilter();

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-text-main">Traffic Management</h1>
          <p className="text-sm text-text-muted mt-1">Monitor backend upstream server load and health</p>
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

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <UpstreamLoadChart />
        <UpstreamHealthChart />
      </div>
    </div>
  );
}
