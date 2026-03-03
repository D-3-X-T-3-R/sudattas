"use client";

import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  LayoutDashboard,
  Package,
  ShoppingCart,
  Users,
  Settings,
  Menu,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

const ADMIN_BASE = "/imtheboss";

const NAV = [
  { href: `${ADMIN_BASE}`, icon: LayoutDashboard, label: "Dashboard" },
  { href: `${ADMIN_BASE}/orders`, icon: ShoppingCart, label: "Orders" },
  { href: `${ADMIN_BASE}/products`, icon: Package, label: "Products" },
  { href: `${ADMIN_BASE}/customers`, icon: Users, label: "Customers" },
  { href: `${ADMIN_BASE}/settings`, icon: Settings, label: "Settings" },
] as const;

function getTitle(pathname: string): string {
  const segment =
    pathname === ADMIN_BASE || pathname === `${ADMIN_BASE}/`
      ? "dashboard"
      : pathname.replace(new RegExp(`^${ADMIN_BASE}/?`), "");
  const titles: Record<string, string> = {
    dashboard: "Dashboard",
    orders: "Orders",
    products: "Products",
    customers: "Customers",
    settings: "Settings",
  };
  return titles[segment] ?? "Admin";
}

export function AdminShell({ children }: { children: React.ReactNode }) {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const pathname = usePathname() ?? "";
  const title = getTitle(pathname);

  return (
    <div className="flex min-h-screen bg-[var(--color-ivory)] text-[var(--color-ink)]">
      {sidebarOpen && (
        <button
          type="button"
          onClick={() => setSidebarOpen(false)}
          className="fixed inset-0 z-30 bg-[rgba(247,245,240,0.85)] md:hidden"
          aria-label="Close menu"
        />
      )}

      <aside
        className={cn(
          "fixed md:static inset-y-0 left-0 z-40 w-64 flex flex-col",
          "border-r border-[var(--color-line)] bg-white transition-transform duration-200 ease-out",
          sidebarOpen ? "translate-x-0" : "-translate-x-full md:translate-x-0"
        )}
      >
        <div className="flex h-14 items-center justify-between border-b border-[var(--color-line)] px-4">
          <span className="text-sm font-semibold tracking-wide">ADMIN</span>
          <Button
            variant="outline"
            size="icon"
            onClick={() => setSidebarOpen(false)}
            className="md:hidden rounded-lg border-[var(--color-line)]"
            aria-label="Close menu"
          >
            <X className="h-5 w-5" />
          </Button>
        </div>
        <nav className="flex-1 space-y-1 p-3">
          {NAV.map(({ href, icon: Icon, label }) => {
            const isActive =
              href === ADMIN_BASE
                ? pathname === ADMIN_BASE || pathname === `${ADMIN_BASE}/`
                : pathname?.startsWith(href);
            return (
              <Link
                key={href}
                href={href}
                onClick={() => setSidebarOpen(false)}
                className={cn(
                  "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors",
                  isActive
                    ? "bg-[var(--color-line)]/40 text-[var(--color-ink)]"
                    : "text-[var(--color-muted)] hover:bg-[#F5F5F4] hover:text-[var(--color-ink)]"
                )}
              >
                <Icon className="h-5 w-5 shrink-0" />
                {label}
              </Link>
            );
          })}
        </nav>
        <div className="border-t border-[var(--color-line)] p-3">
          <Link
            href="/"
            className="flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm text-[var(--color-muted)] transition-colors hover:bg-[#F5F5F4] hover:text-[var(--color-ink)]"
          >
            ← Back to store
          </Link>
        </div>
      </aside>

      <main className="flex flex-1 flex-col min-w-0">
        <header className="sticky top-0 z-20 flex h-14 items-center gap-2 border-b border-[var(--color-line)] bg-[var(--color-ivory)]/95 px-4 backdrop-blur">
          <Button
            variant="outline"
            size="icon"
            onClick={() => setSidebarOpen(true)}
            className="md:hidden rounded-lg border-[var(--color-line)]"
            aria-label="Open menu"
          >
            <Menu className="h-5 w-5" />
          </Button>
          <h1 className="text-lg font-semibold">{title}</h1>
        </header>
        <div className="flex-1 p-4 md:p-6">{children}</div>
      </main>
    </div>
  );
}
