"use client";

import { useQuery } from "@tanstack/react-query";
import {
  PieChart,
  Pie,
  Cell,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";
import { api } from "@/api";
import { ChartCard } from "./ChartCard";
import { useMetricFilter } from "@/hooks/use-metric-filter";
import { chartColors, defaultTooltip } from "./config";

export function UpstreamLoadChart() {
  const { queryParams } = useMetricFilter();

  const { data = [], isLoading, error } = useQuery({
    queryKey: ["metrics", "requests", "summary", "upstreams-load", queryParams],
    queryFn: () => api.metrics.request.summary({ ...queryParams, group_by: "upstream" }),
  });

  const chartData = data
    .filter((d) => d.key.trim().toLowerCase() !== "none")
    .map((d) => ({
      name: d.key,
      value: d.count,
    }));

  const COLORS = [chartColors.ocean, chartColors.indigo, chartColors.violet, chartColors.emerald, chartColors.amber];

  return (
    <ChartCard
      title="Upstream Load Distribution"
      description="Request volume per backend server"
      isLoading={isLoading}
      error={error}
      isEmpty={chartData.length === 0}
    >
      <ResponsiveContainer width="100%" height="100%">
        <PieChart margin={{ top: 10, right: 10, left: 10, bottom: 0 }}>
          <Pie
            data={chartData}
            cx="50%"
            cy="45%"
            innerRadius={60}
            outerRadius={90}
            paddingAngle={2}
            dataKey="value"
          >
            {chartData.map((entry, index) => (
              <Cell
                key={`cell-${index}`}
                fill={COLORS[index % COLORS.length]}
              />
            ))}
          </Pie>
          <Tooltip
            {...defaultTooltip}
            formatter={(value: number | string | ReadonlyArray<number | string> | undefined) => [
              `${(Number(value) || 0).toLocaleString()}`,
              "Requests",
            ]}
          />
          <Legend
            verticalAlign="bottom"
            height={36}
            wrapperStyle={{ fontSize: "12px" }}
          />
        </PieChart>
      </ResponsiveContainer>
    </ChartCard>
  );
}
