import { CartesianGridProps, TooltipProps, XAxisProps, YAxisProps } from "recharts";

export const chartColors = {
  ocean: "#0ea5e9", 
  oceanLight: "#e0f2fe", 
  emerald: "#10b981", 
  amber: "#f59e0b", 
  rose: "#ef4444", 
  slate: "#64748b", 
  slateLight: "#e2e8f0", 
  indigo: "#6366f1",
  violet: "#8b5cf6",
};

export const defaultXAxis: XAxisProps = {
  stroke: chartColors.slateLight,
  tick: { fill: chartColors.slate, fontSize: 12 },
  tickLine: false,
  axisLine: false,
  dy: 10,
};

export const defaultYAxis: YAxisProps = {
  stroke: chartColors.slateLight,
  tick: { fill: chartColors.slate, fontSize: 12 },
  tickLine: false,
  axisLine: false,
  dx: -10,
};

export const defaultGrid: CartesianGridProps = {
  strokeDasharray: "3 3",
  vertical: false,
  stroke: "#f1f5f9",
};

export const defaultTooltip: TooltipProps<any, any> = {
  contentStyle: {
    backgroundColor: "#ffffff",
    borderRadius: "0.5rem",
    border: "1px solid #e2e8f0",
    boxShadow: "0 10px 30px -3px rgba(0, 0, 0, 0.06)",
    fontSize: "0.875rem",
    color: "#0f172a",
    padding: "8px 12px",
  },
  itemStyle: {
    color: "#0f172a",
    fontWeight: 500,
  },
  cursor: {
    stroke: chartColors.slateLight,
    strokeWidth: 1,
    strokeDasharray: "3 3",
  },
};
