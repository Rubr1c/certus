"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { ChevronRight, Calendar } from "lucide-react";

const CHEVRON_SIZE = 14;

function formatSegment(segment: string): string {
  if (!segment) return "Dashboard";
  return segment.charAt(0).toUpperCase() + segment.slice(1).toLowerCase();
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
              <span className="text-sm font-semibold text-slate-900">
                {crumb.label}
              </span>
            ) : (
              <Link
                href={crumb.href}
                className="text-sm font-medium text-slate-500 transition-colors hover:text-slate-700"
              >
                {crumb.label}
              </Link>
            )}
          </span>
        ))}
      </nav>

      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2 rounded-full border border-slate-100 bg-white px-3 py-1.5 shadow-sm">
          <span
            className="h-2 w-2 shrink-0 rounded-full bg-emerald-500 animate-pulse"
            aria-hidden
          />
          <span className="text-xs font-medium text-slate-600">Gateway Online</span>
        </div>

        <button
          type="button"
          className="flex items-center gap-2 rounded-lg border border-slate-200 bg-white px-3 py-1.5 text-sm font-medium text-slate-600 shadow-sm transition-colors hover:bg-slate-50"
        >
          <Calendar size={16} className="text-slate-500" aria-hidden />
          Last 24 Hours
        </button>
      </div>
    </header>
  );
}
