"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/api";
import { ChartCard } from "./ChartCard";
import { useMetricFilter } from "@/hooks/use-metric-filter";

export function CacheHitRateKPI() {
  const { queryParams } = useMetricFilter();

  const { data = [], isLoading, error } = useQuery({
    queryKey: ["metrics", "cache", "aggregate", "kpi", queryParams],
    queryFn: () => api.metrics.cache.aggregate(queryParams),
  });

  const total = data.reduce((acc, curr) => acc + curr.total, 0);
  const hits = data.reduce((acc, curr) => acc + curr.hits, 0);
  
  const globalHitRate = total > 0 ? (hits / total) * 100 : 0;
  const formattedRate = globalHitRate.toFixed(1);

  return (
    <ChartCard
      title="Cache Efficiency"
      description="Global hit rate for the selected period"
      isLoading={isLoading}
      error={error}
      height={150} 
    >
      <div className="flex h-full flex-col items-center justify-center pt-4">
        <div className="text-5xl font-bold text-ocean-600 font-mono tracking-tight">
          {total === 0 ? "-" : `${formattedRate}%`}
        </div>
        <div className="mt-2 text-sm font-medium text-slate-500">
          {total === 0 ? "No cache requests" : `${hits} hits / ${total} total`}
        </div>
      </div>
    </ChartCard>
  );
}
