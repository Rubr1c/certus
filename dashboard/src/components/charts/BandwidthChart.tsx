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
  Legend,
} from "recharts";
import { format } from "date-fns";
import { api } from "@/api";
import { ChartCard } from "./ChartCard";
import { useMetricFilter } from "@/hooks/use-metric-filter";
import { chartColors, defaultGrid, defaultTooltip, defaultXAxis, defaultYAxis } from "./config";

function formatBytes(bytes: number) {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

export function BandwidthChart() {
  const { queryParams } = useMetricFilter();

  const { data = [], isLoading, error } = useQuery({
    queryKey: ["metrics", "requests", "aggregate", "bandwidth", queryParams],
    queryFn: () => api.metrics.request.aggregate(queryParams),
  });

  const chartData = data.map((d) => ({
    ...d,
    displayTime: format(new Date(d.bucket), "HH:mm"),
  }));

  return (
    <ChartCard
      title="Bandwidth Throughput"
      description="Data transfer over time (In / Out)"
      isLoading={isLoading}
      error={error}
      isEmpty={chartData.length === 0}
    >
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={chartData} margin={{ top: 10, right: 10, left: 10, bottom: 0 }}>
          <defs>
            <linearGradient id="colorIn" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor={chartColors.violet} stopOpacity={0.3} />
              <stop offset="95%" stopColor={chartColors.violet} stopOpacity={0} />
            </linearGradient>
            <linearGradient id="colorOut" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor={chartColors.ocean} stopOpacity={0.3} />
              <stop offset="95%" stopColor={chartColors.ocean} stopOpacity={0} />
            </linearGradient>
          </defs>
          <CartesianGrid {...defaultGrid} />
          <XAxis dataKey="displayTime" {...defaultXAxis} minTickGap={30} />
          <YAxis 
            {...defaultYAxis} 
            tickFormatter={formatBytes}
            width={60}
          />
          <Tooltip
            {...defaultTooltip}
            labelFormatter={(label) => `Time: ${label}`}
            formatter={(value: number) => [formatBytes(value), undefined]}
          />
          <Legend wrapperStyle={{ paddingTop: "20px", fontSize: "12px" }} />
          <Area
            type="monotone"
            dataKey="bytes_in"
            name="Bytes In (Request)"
            stroke={chartColors.violet}
            strokeWidth={2}
            fillOpacity={1}
            fill="url(#colorIn)"
          />
          <Area
            type="monotone"
            dataKey="bytes_out"
            name="Bytes Out (Response)"
            stroke={chartColors.ocean}
            strokeWidth={2}
            fillOpacity={1}
            fill="url(#colorOut)"
          />
        </AreaChart>
      </ResponsiveContainer>
    </ChartCard>
  );
}
