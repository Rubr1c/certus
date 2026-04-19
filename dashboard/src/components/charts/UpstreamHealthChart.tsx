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
  LabelList,
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
    .filter((d) => d.key.trim().toLowerCase() !== "none")
    .map((d) => ({
      name: d.key,
      avg_upstream_ms_raw: Number(d.avg_upstream_ms.toFixed(2)),
      avg_upstream_ms: d.avg_upstream_ms > 0 && d.avg_upstream_ms < 1 ? 0.4 : Number(d.avg_upstream_ms.toFixed(2)),
      avg_upstream_label:
        d.avg_upstream_ms > 0 && d.avg_upstream_ms < 1
          ? "<1 ms"
          : `${Number(d.avg_upstream_ms.toFixed(2))} ms`,
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
            formatter={(
              _value: number | string | ReadonlyArray<number | string> | undefined,
              _name,
              item,
            ) => {
              const payload = item?.payload as { avg_upstream_label?: string } | undefined;
              return [payload?.avg_upstream_label ?? "0 ms", "Avg Latency"];
            }}
            labelFormatter={() => ""} 
          />
          <Bar
            dataKey="avg_upstream_ms"
            name="Latency"
            radius={[0, 4, 4, 0]}
            barSize={24}
            minPointSize={3}
          >
            <LabelList
              dataKey="avg_upstream_label"
              position="right"
              fill={chartColors.slate}
              fontSize={11}
            />
            {chartData.map((entry, index) => {
              const isSlowest = chartData.length > 1 && entry.avg_upstream_ms_raw === Math.max(...chartData.map((d) => d.avg_upstream_ms_raw));
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
