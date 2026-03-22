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

export function StatusCodesChart() {
  const { queryParams } = useMetricFilter();

  const { data = [], isLoading, error } = useQuery({
    queryKey: ["metrics", "requests", "aggregate", "status", queryParams],
    queryFn: () => api.metrics.request.aggregate(queryParams),
  });

  const chartData = data.map((d) => ({
    ...d,
    displayTime: format(new Date(d.bucket), "HH:mm"),
  }));

  return (
    <ChartCard
      title="Status Code Distribution"
      description="HTTP response codes over time"
      isLoading={isLoading}
      error={error}
      isEmpty={chartData.length === 0}
    >
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={chartData} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <CartesianGrid {...defaultGrid} />
          <XAxis dataKey="displayTime" {...defaultXAxis} minTickGap={30} />
          <YAxis {...defaultYAxis} width={40} />
          <Tooltip
            {...defaultTooltip}
            labelFormatter={(label) => `Time: ${label}`}
            cursor={{ fill: chartColors.slateLight, opacity: 0.2 }}
          />
          <Legend wrapperStyle={{ paddingTop: "20px", fontSize: "12px" }} />
          <Bar
            dataKey="status_2xx"
            stackId="a"
            name="2xx Success"
            fill={chartColors.emerald}
            radius={[0, 0, 4, 4]} // bottom rounded
          />
          <Bar
            dataKey="status_3xx"
            stackId="a"
            name="3xx Redirect"
            fill={chartColors.ocean}
          />
          <Bar
            dataKey="status_4xx"
            stackId="a"
            name="4xx Client Error"
            fill={chartColors.amber}
          />
          <Bar
            dataKey="status_5xx"
            stackId="a"
            name="5xx Server Error"
            fill={chartColors.rose}
            radius={[4, 4, 0, 0]} // top rounded
          />
        </BarChart>
      </ResponsiveContainer>
    </ChartCard>
  );
}
