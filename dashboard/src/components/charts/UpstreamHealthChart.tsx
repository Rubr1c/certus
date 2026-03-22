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
  Cell,
} from "recharts";
import { api } from "@/api";
import { ChartCard } from "./ChartCard";
import { useMetricFilter } from "@/hooks/use-metric-filter";
import { chartColors, defaultGrid, defaultTooltip, defaultXAxis, defaultYAxis } from "./config";

export function UpstreamHealthChart() {
  const { queryParams } = useMetricFilter();

  const { data = [], isLoading, error } = useQuery({
    queryKey: ["metrics", "requests", "summary", "upstreams-health", queryParams],
    queryFn: () => api.metrics.request.summary({ ...queryParams, group_by: "upstream" }),
  });

  const chartData = data
    .filter(d => d.key !== "None") // Filter out early exits/cache hits
    .map((d) => ({
      name: d.key,
      avg_upstream_ms: Number(d.avg_upstream_ms.toFixed(2)),
    }));

  return (
    <ChartCard
      title="Upstream Health Comparison"
      description="Average response time per backend server"
      isLoading={isLoading}
      error={error}
      isEmpty={chartData.length === 0}
    >
      <ResponsiveContainer width="100%" height="100%">
        <BarChart
          data={chartData}
          layout="vertical"
          margin={{ top: 10, right: 20, left: 10, bottom: 0 }}
        >
          <CartesianGrid {...defaultGrid} horizontal={true} vertical={false} />
          <XAxis type="number" {...defaultXAxis} hide />
          <YAxis
            type="category"
            dataKey="name"
            {...defaultYAxis}
            width={120}
            axisLine={false}
            tickLine={false}
            dx={-10}
          />
          <Tooltip
            {...defaultTooltip}
            cursor={{ fill: chartColors.slateLight, opacity: 0.4 }}
            formatter={(value: number) => [`${value} ms`, "Avg Latency"]}
            labelFormatter={() => ""} 
          />
          <Bar
            dataKey="avg_upstream_ms"
            name="Latency"
            radius={[0, 4, 4, 0]}
            barSize={24}
          >
            {chartData.map((entry, index) => {
              // Highlight the slowest server in amber/red if we have multiple
              const isSlowest = chartData.length > 1 && entry.avg_upstream_ms === Math.max(...chartData.map(d => d.avg_upstream_ms));
              return (
                <Cell 
                  key={`cell-${index}`} 
                  fill={isSlowest ? chartColors.amber : chartColors.oceanLight} 
                />
              );
            })}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </ChartCard>
  );
}
