"use client";

import { useQuery } from "@tanstack/react-query";
import {
  AreaChart,
  Area,
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

export function GlobalTrafficChart() {
  const { queryParams } = useMetricFilter();

  const { data = [], isLoading, error } = useQuery({
    queryKey: ["metrics", "requests", "aggregate", queryParams],
    queryFn: () => api.metrics.request.aggregate(queryParams),
  });

  const chartData = data.map((d) => ({
    ...d,
    displayTime: format(new Date(d.bucket), "HH:mm"),
  }));

  return (
    <ChartCard
      title="Global Traffic Volume"
      description="Total requests processed by the gateway"
      isLoading={isLoading}
      error={error}
      isEmpty={chartData.length === 0}
    >
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={chartData} margin={{ top: 10, right: 10, left: 10, bottom: 0 }}>
          <defs>
            <linearGradient id="colorTraffic" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor={chartColors.ocean} stopOpacity={0.3} />
              <stop offset="95%" stopColor={chartColors.ocean} stopOpacity={0} />
            </linearGradient>
            <linearGradient id="colorErrors" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor={chartColors.rose} stopOpacity={0.3} />
              <stop offset="95%" stopColor={chartColors.rose} stopOpacity={0} />
            </linearGradient>
          </defs>
          <CartesianGrid {...defaultGrid} />
          <XAxis dataKey="displayTime" {...defaultXAxis} minTickGap={30} />
          <YAxis {...defaultYAxis} width={40} />
          <Tooltip
            {...defaultTooltip}
            labelFormatter={(label) => `Time: ${label}`}
          />
          <Area
            type="monotone"
            dataKey="count"
            name="Total Requests"
            stroke={chartColors.ocean}
            strokeWidth={2}
            fillOpacity={1}
            fill="url(#colorTraffic)"
          />
          <Area
            type="monotone"
            dataKey="status_5xx"
            name="5xx Errors"
            stroke={chartColors.rose}
            strokeWidth={2}
            fillOpacity={1}
            fill="url(#colorErrors)"
          />
        </AreaChart>
      </ResponsiveContainer>
    </ChartCard>
  );
}
