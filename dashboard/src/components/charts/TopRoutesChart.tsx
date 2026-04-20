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

export function TopRoutesChart() {
  const { queryParams } = useMetricFilter();

  const { data = [], isLoading, error } = useQuery({
    queryKey: ["metrics", "requests", "summary", "top-routes", queryParams],
    queryFn: () => api.metrics.request.summary({ ...queryParams, group_by: "route" }),
  });

  const chartData = [...data]
    .sort((a, b) => b.count - a.count)
    .slice(0, 5)
    .map(d => ({
      ...d,
      shortRoute: d.key.length > 25 ? d.key.substring(0, 25) + "..." : d.key
    }));

  return (
    <ChartCard
      title="Top 5 Busiest Routes"
      description="Total requests per endpoint"
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
            dataKey="shortRoute"
            {...defaultYAxis}
            width={120}
            axisLine={false}
            tickLine={false}
            dx={-10}
          />
          <Tooltip
            {...defaultTooltip}
            cursor={{ fill: chartColors.slateLight, opacity: 0.4 }}
            formatter={(value: number | string | ReadonlyArray<number | string> | undefined) => [
              `${(Number(value) || 0).toLocaleString()}`,
              "Requests",
            ]}
            labelFormatter={() => ""} 
          />
          <Bar
            dataKey="count"
            name="Requests"
            radius={[0, 4, 4, 0]}
            barSize={24}
          >
            {chartData.map((entry, index) => (
              <Cell 
                key={`cell-${index}`} 
                fill={index === 0 ? chartColors.ocean : chartColors.oceanLight} 
              />
            ))}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </ChartCard>
  );
}
