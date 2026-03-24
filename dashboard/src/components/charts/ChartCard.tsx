"use client";

import { ReactNode } from "react";
import { AlertCircle, Inbox, Loader2 } from "lucide-react";

interface ChartCardProps {
  title: string;
  description?: string;
  children: ReactNode;
  isLoading?: boolean;
  error?: any;
  isEmpty?: boolean;
  className?: string;
  height?: number | string;
}

export function ChartCard({
  title,
  description,
  children,
  isLoading,
  error,
  isEmpty,
  className = "",
  height = 350,
}: ChartCardProps) {
  return (
    <div className={`overflow-hidden rounded-xl border border-slate-100 bg-white p-6 shadow-sm ${className}`}>
      <div className="mb-6">
        <h3 className="text-sm font-semibold text-slate-900">{title}</h3>
        {description && (
          <p className="mt-1 text-xs text-slate-500">{description}</p>
        )}
      </div>

      <div style={{ height: typeof height === 'number' ? `${height}px` : height }} className="relative w-full">
        {isLoading ? (
          <div className="flex h-full w-full items-center justify-center bg-slate-50/50 rounded-lg">
            <Loader2 className="h-6 w-6 animate-spin text-slate-400" />
          </div>
        ) : error ? (
          <div className="flex h-full w-full flex-col items-center justify-center rounded-lg border border-dashed border-red-100 bg-red-50/30 text-red-600">
            <AlertCircle className="h-5 w-5 mb-2 opacity-50" />
            <p className="text-xs font-medium">Failed to load chart</p>
          </div>
        ) : isEmpty ? (
          <div className="flex h-full w-full flex-col items-center justify-center rounded-lg border border-dashed border-slate-100 bg-slate-50/30 text-slate-400">
            <Inbox className="h-5 w-5 mb-2 opacity-50" />
            <p className="text-xs font-medium">No data available</p>
          </div>
        ) : (
          children
        )}
      </div>
    </div>
  );
}
