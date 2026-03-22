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
  Legend,
} from "recharts";
import { format } from "date-fns";
import { api } from "@/api";
import { ChartCard } from "./ChartCard";
import { useMetricFilter } from "@/hooks/use-metric-filter";
import { chartColors, defaultGrid, defaultTooltip, defaultXAxis, defaultYAxis } from "./config";

export function GlobalLatencyChart() {
  const { queryParams } = useMetricFilter();

  const { data = [], isLoading, error } = useQuery({
    queryKey: ["metrics", "requests", "aggregate", "latency", queryParams],
    queryFn: () => api.metrics.request.aggregate(queryParams),
  });

  const chartData = data.map((d) => ({
    ...d,
    displayTime: format(new Date(d.bucket), "HH:mm"),
    // Format to 2 decimal places for cleaner tooltips
    avg_duration_ms: Number(d.avg_duration_ms.toFixed(2)),
    avg_upstream_ms: Number(d.avg_upstream_ms.toFixed(2)),
  }));

  return (
    <ChartCard
      title="Global Latency Pulse"
      description="Average total duration vs upstream duration (ms)"
      isLoading={isLoading}
      error={error}
      isEmpty={chartData.length === 0}
    >
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={chartData} margin={{ top: 10, right: 10, left: 10, bottom: 0 }}>
          <CartesianGrid {...defaultGrid} />
          <XAxis dataKey="displayTime" {...defaultXAxis} minTickGap={30} />
          <YAxis 
            {...defaultYAxis} 
            tickFormatter={(val) => `${val}ms`} 
            width={60}
          />
          <Tooltip
            {...defaultTooltip}
            labelFormatter={(label) => `Time: ${label}`}
            formatter={(value: number | string | ReadonlyArray<number | string> | undefined) => [
              `${value || 0} ms`,
              undefined,
            ]}
          />
          <Legend wrapperStyle={{ paddingTop: "20px", fontSize: "12px" }} />
          <Line
            type="monotone"
            dataKey="avg_duration_ms"
            name="Total Duration"
            stroke={chartColors.indigo}
            strokeWidth={2}
            dot={false}
            activeDot={{ r: 4, strokeWidth: 0 }}
          />
          <Line
            type="monotone"
            dataKey="avg_upstream_ms"
            name="Upstream Duration"
            stroke={chartColors.emerald}
            strokeWidth={2}
            strokeDasharray="5 5"
            dot={false}
            activeDot={{ r: 4, strokeWidth: 0 }}
          />
        </LineChart>
      </ResponsiveContainer>
    </ChartCard>
  );
}
