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

// Consistent colors for common HTTP methods
const METHOD_COLORS: Record<string, string> = {
  GET: chartColors.ocean,
  POST: chartColors.emerald,
  PUT: chartColors.indigo,
  DELETE: chartColors.rose,
  PATCH: chartColors.amber,
  OPTIONS: chartColors.slate,
  HEAD: chartColors.violet,
};

export function HttpMethodsChart() {
  const { queryParams } = useMetricFilter();

  const { data = [], isLoading, error } = useQuery({
    queryKey: ["metrics", "requests", "summary", "methods", queryParams],
    queryFn: () => api.metrics.request.summary({ ...queryParams, group_by: "method" }),
  });

  const chartData = data.map((d) => ({
    name: d.key.toUpperCase(),
    value: d.count,
  }));

  return (
    <ChartCard
      title="HTTP Methods"
      description="Distribution of request methods"
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
                fill={METHOD_COLORS[entry.name] || chartColors.slateLight}
              />
            ))}
          </Pie>
          <Tooltip
            {...defaultTooltip}
            formatter={(value: number) => [`${value.toLocaleString()}`, "Requests"]}
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
