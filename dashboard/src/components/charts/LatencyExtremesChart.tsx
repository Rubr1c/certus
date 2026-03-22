"use client";

import { useQuery } from "@tanstack/react-query";
import {
  ComposedChart,
  Line,
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

export function LatencyExtremesChart() {
  const { queryParams } = useMetricFilter();

  const { data = [], isLoading, error } = useQuery({
    queryKey: ["metrics", "requests", "aggregate", "extremes", queryParams],
    queryFn: () => api.metrics.request.aggregate(queryParams),
  });

  const chartData = data.map((d) => ({
    ...d,
    displayTime: format(new Date(d.bucket), "HH:mm"),
    // Format to 2 decimal places
    avg_duration_ms: Number(d.avg_duration_ms.toFixed(2)),
    // Recharts Area needs an array [min, max] to shade a region
    range: [d.min_duration_ms, d.max_duration_ms],
  }));

  return (
    <ChartCard
      title="Latency Extremes"
      description="Min, Max, and Average request duration (ms)"
      isLoading={isLoading}
      error={error}
      isEmpty={chartData.length === 0}
    >
      <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={chartData} margin={{ top: 10, right: 10, left: 10, bottom: 0 }}>
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
            formatter={(value: any, name: string) => {
              if (name === "Min/Max Range") {
                return [`${value[0]}ms - ${value[1]}ms`, undefined];
              }
              return [`${value} ms`, undefined];
            }}
          />
          <Legend wrapperStyle={{ paddingTop: "20px", fontSize: "12px" }} />
          <Area
            type="monotone"
            dataKey="range"
            name="Min/Max Range"
            stroke="none"
            fill={chartColors.slateLight}
            fillOpacity={0.4}
          />
          <Line
            type="monotone"
            dataKey="avg_duration_ms"
            name="Average Duration"
            stroke={chartColors.indigo}
            strokeWidth={3}
            dot={false}
            activeDot={{ r: 4, strokeWidth: 0 }}
          />
        </ComposedChart>
      </ResponsiveContainer>
    </ChartCard>
  );
}
