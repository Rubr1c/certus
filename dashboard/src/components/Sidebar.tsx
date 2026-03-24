"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  LayoutDashboard,
  Route,
  FileText,
  BarChart3,
  ScrollText,
  Settings,
  type LucideIcon,
} from "lucide-react";

export type NavItem = {
  href: string;
  label: string;
  icon: LucideIcon;
};

export type NavCategory = {
  label: string;
  items: NavItem[];
};

const NAV_STRUCTURE: NavCategory[] = [
  {
    label: "Overview",
    items: [{ href: "/", label: "Dashboard", icon: LayoutDashboard }],
  },
  {
    label: "Traffic Management",
    items: [
      { href: "/routes", label: "Routes", icon: Route },
      { href: "/docs", label: "Docs", icon: FileText },
    ],
  },
  {
    label: "Observability",
    items: [
      { href: "/metrics", label: "Metrics", icon: BarChart3 },
      { href: "/logs", label: "Logs", icon: ScrollText },
    ],
  },
  {
    label: "System",
    items: [{ href: "/config", label: "Configuration", icon: Settings }],
  },
];

const ICON_SIZE = 18;

export function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="fixed inset-y-0 left-0 z-50 flex w-64 flex-col border-r border-slate-100 bg-[#FFFFFF]">
      <div className="flex h-16 shrink-0 items-center px-4">
        <span className="font-sans text-xl font-bold text-slate-900">
          Certus
        </span>
      </div>

      <nav className="flex flex-1 flex-col overflow-y-auto px-2 pb-4">
        {NAV_STRUCTURE.map((category) => (
          <div key={category.label}>
            <h2 className="mb-2 mt-6 px-4 font-sans text-xs font-semibold uppercase tracking-wider text-slate-400">
              {category.label}
            </h2>
            {category.items.map((item) => {
              const isActive = pathname === item.href;
              const Icon = item.icon;
              return (
                <Link
                  key={item.href}
                  href={item.href}
                  className={`mt-1 flex items-center gap-3 rounded-lg px-4 py-2 font-sans transition-colors ${
                    isActive
                      ? "bg-[#f0f9ff] font-semibold text-[#0ea5e9] shadow-sm"
                      : "font-medium text-slate-500 hover:bg-[#f0f9ff] hover:text-[#0284c7]"
                  }`}
                >
                  <Icon
                    size={ICON_SIZE}
                    className="shrink-0"
                    {...(isActive ? { color: "#0ea5e9" } : {})}
                  />
                  {item.label}
                </Link>
              );
            })}
          </div>
        ))}
      </nav>
    </aside>
  );
}
