"use client";

import { useQuery } from "@tanstack/react-query";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";
import { format } from "date-fns";
import { api } from "@/api";
import { ChartCard } from "./ChartCard";
import { useMetricFilter } from "@/hooks/use-metric-filter";
import { chartColors, defaultGrid, defaultTooltip, defaultXAxis, defaultYAxis } from "./config";

export function CacheOperationsChart() {
  const { queryParams } = useMetricFilter();

  const { data = [], isLoading, error } = useQuery({
    queryKey: ["metrics", "cache", "aggregate", "operations", queryParams],
    queryFn: () => api.metrics.cache.aggregate(queryParams),
  });

  const chartData = data.map((d) => ({
    ...d,
    displayTime: format(new Date(d.bucket), "HH:mm"),
  }));

  return (
    <ChartCard
      title="Cache Operations"
      description="Hits, misses, and bypasses over time"
      isLoading={isLoading}
      error={error}
      isEmpty={chartData.length === 0}
    >
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={chartData} margin={{ top: 10, right: 10, left: -20, bottom: 0 }}>
          <CartesianGrid {...defaultGrid} />
          <XAxis dataKey="displayTime" {...defaultXAxis} minTickGap={30} />
          <YAxis {...defaultYAxis} />
          <Tooltip
            {...defaultTooltip}
            labelFormatter={(label) => `Time: ${label}`}
            cursor={{ fill: chartColors.slateLight, opacity: 0.2 }}
          />
          <Legend wrapperStyle={{ paddingTop: "20px", fontSize: "12px" }} />
          <Bar
            dataKey="hits"
            stackId="a"
            name="Cache Hits"
            fill={chartColors.emerald}
            radius={[0, 0, 4, 4]}
          />
          <Bar
            dataKey="misses"
            stackId="a"
            name="Cache Misses"
            fill={chartColors.amber}
          />
          <Bar
            dataKey="bypasses"
            stackId="a"
            name="Cache Bypasses"
            fill={chartColors.slate}
            radius={[4, 4, 0, 0]}
          />
        </BarChart>
      </ResponsiveContainer>
    </ChartCard>
  );
}
