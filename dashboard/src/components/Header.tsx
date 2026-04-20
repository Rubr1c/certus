"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { ChevronRight } from "lucide-react";

import { useArgs } from "@/hooks/use-args";

const CHEVRON_SIZE = 14;

function formatSegment(segment: string): string {
  if (!segment) return "Dashboard";
  const label = segment.charAt(0).toUpperCase() + segment.slice(1).toLowerCase();
  if (label === "Config") return "Configuration";
  return label;
}

function getBreadcrumbs(pathname: string): { label: string; href: string; isLast: boolean }[] {
  const segments = pathname.split("/").filter(Boolean);
  if (segments.length === 0) {
    return [{ label: "Dashboard", href: "/", isLast: true }];
  }
  return segments.map((segment, index) => {
    const href = "/" + segments.slice(0, index + 1).join("/");
    const isLast = index === segments.length - 1;
    return {
      label: formatSegment(segment),
      href,
      isLast,
    };
  });
}

export function Header() {
  const pathname = usePathname();
  const breadcrumbs = getBreadcrumbs(pathname);
  const { isSuccess, isLoading } = useArgs();

  const isOnline = isSuccess;

  return (
    <header className="fixed top-0 right-0 left-64 z-40 flex h-16 items-center justify-between border-b border-slate-100 bg-white/80 px-8 backdrop-blur-md">
      <nav className="flex items-center gap-1" aria-label="Breadcrumb">
        {breadcrumbs.map((crumb, index) => (
          <span key={crumb.href} className="flex items-center gap-1">
            {index > 0 && (
              <ChevronRight
                size={CHEVRON_SIZE}
                className="shrink-0 text-slate-400"
                aria-hidden
              />
            )}
            {crumb.isLast ? (
              <span className="text-sm font-bold text-slate-900 uppercase tracking-tight">
                {crumb.label}
              </span>
            ) : (
              <Link
                href={crumb.href}
                className="text-sm font-bold text-slate-400 transition-colors hover:text-slate-600 uppercase tracking-tight"
              >
                {crumb.label}
              </Link>
            )}
          </span>
        ))}
      </nav>

      <div className="flex items-center gap-4">
        <div className={`flex items-center gap-2 rounded-full border px-3 py-1.5 shadow-sm transition-colors ${
          isLoading ? "bg-slate-50 border-slate-100" : isOnline ? "bg-emerald-50 border-emerald-100" : "bg-red-50 border-red-100"
        }`}>
          {isLoading ? (
            <div className="h-2 w-2 shrink-0 rounded-full bg-slate-300 animate-pulse" />
          ) : (
            <span
              className={`h-2 w-2 shrink-0 rounded-full ${isOnline ? "bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]" : "bg-red-500"}`}
              aria-hidden
            />
          )}
          <span className={`text-[10px] font-bold uppercase tracking-widest ${
            isLoading ? "text-slate-400" : isOnline ? "text-emerald-600" : "text-red-600"
          }`}>
            {isLoading ? "Checking..." : isOnline ? "Gateway Online" : "Gateway Offline"}
          </span>
        </div>
      </div>
    </header>
  );
}
