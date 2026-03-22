import { Loader2, AlertCircle } from "lucide-react";
import { ReactNode } from "react";

interface ChartCardProps {
  title: string;
  description?: string;
  isLoading?: boolean;
  error?: Error | null;
  isEmpty?: boolean;
  children: ReactNode;
  action?: ReactNode;
  height?: string | number;
}

export function ChartCard({
  title,
  description,
  isLoading,
  error,
  isEmpty,
  children,
  action,
  height = 300,
}: ChartCardProps) {
  return (
    <div className="card-container flex flex-col p-5">
      <div className="flex items-start justify-between mb-4">
        <div>
          <h3 className="font-semibold text-text-main">{title}</h3>
          {description && (
            <p className="text-sm text-text-muted mt-1">{description}</p>
          )}
        </div>
        {action && <div>{action}</div>}
      </div>

      <div
        className="relative w-full flex-1"
        style={{ minHeight: height }}
      >
        {isLoading ? (
          <div className="absolute inset-0 flex flex-col items-center justify-center text-text-muted bg-white/50 backdrop-blur-sm z-10 rounded-lg">
            <Loader2 className="h-6 w-6 animate-spin text-ocean-500 mb-2" />
            <span className="text-sm font-medium">Loading data...</span>
          </div>
        ) : error ? (
          <div className="absolute inset-0 flex flex-col items-center justify-center text-red-500 bg-red-50/50 rounded-lg border border-red-100">
            <AlertCircle className="h-6 w-6 mb-2" />
            <span className="text-sm font-medium text-red-600">Failed to load chart</span>
          </div>
        ) : isEmpty ? (
          <div className="absolute inset-0 flex flex-col items-center justify-center text-text-muted bg-slate-50/50 rounded-lg border border-slate-100 border-dashed">
            <svg
              className="h-10 w-10 mb-2 text-slate-300"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
              />
            </svg>
            <span className="text-sm font-medium">No data available</span>
          </div>
        ) : (
          <div className="w-full h-full absolute inset-0 select-none cursor-default [&_svg]:select-none [&_text]:select-none [&_text]:cursor-default">
            {children}
          </div>
        )}
      </div>
    </div>
  );
}
