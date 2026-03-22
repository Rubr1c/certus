"use client";

import { useQuery } from "@tanstack/react-query";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { format } from "date-fns";
import { api } from "@/api";
import { ChartCard } from "./ChartCard";
import { useMetricFilter } from "@/hooks/use-metric-filter";
import { chartColors, defaultGrid, defaultTooltip, defaultXAxis, defaultYAxis } from "./config";

export function HitRateTrendChart() {
  const { queryParams } = useMetricFilter();

  const { data = [], isLoading, error } = useQuery({
    queryKey: ["metrics", "cache", "aggregate", "hit-rate", queryParams],
    queryFn: () => api.metrics.cache.aggregate(queryParams),
  });

  const chartData = data.map((d) => ({
    ...d,
    displayTime: format(new Date(d.bucket), "HH:mm"),
    hit_rate_pct: Number((d.hit_rate * 100).toFixed(2)),
  }));

  return (
    <ChartCard
      title="Cache Hit Rate Trend"
      description="Percentage of requests served from cache"
      isLoading={isLoading}
      error={error}
      isEmpty={chartData.length === 0}
    >
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={chartData} margin={{ top: 10, right: 10, left: -20, bottom: 0 }}>
          <CartesianGrid {...defaultGrid} />
          <XAxis dataKey="displayTime" {...defaultXAxis} minTickGap={30} />
          <YAxis 
            {...defaultYAxis} 
            domain={[0, 100]} 
            tickFormatter={(val) => `${val}%`} 
          />
          <Tooltip
            {...defaultTooltip}
            labelFormatter={(label) => `Time: ${label}`}
            formatter={(value: number) => [`${value}%`, "Hit Rate"]}
          />
          <Line
            type="monotone"
            dataKey="hit_rate_pct"
            name="Hit Rate"
            stroke={chartColors.ocean}
            strokeWidth={3}
            dot={false}
            activeDot={{ r: 6, strokeWidth: 0, fill: chartColors.ocean }}
          />
        </LineChart>
      </ResponsiveContainer>
    </ChartCard>
  );
}
