"use client";

import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { signOut } from "next-auth/react";
import {
  LayoutDashboard,
  Package,
  ShoppingCart,
  Users,
  Settings,
  Menu,
  X,
  LogOut,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { SectionHeading } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

const ADMIN_BASE = "/imtheboss";
const STORE_URL =
  (typeof process !== "undefined" && process.env.NEXT_PUBLIC_STORE_URL) || "/";

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
          className="fixed inset-0 z-30 bg-[var(--color-ivory)]/90 backdrop-blur-[2px] md:hidden"
          aria-label="Close menu"
        />
      )}

      <aside
        className={cn(
          "fixed inset-y-0 left-0 z-40 w-64 flex flex-col",
          "border-r border-[var(--color-line)] bg-white transition-transform duration-200 ease-out",
          sidebarOpen ? "translate-x-0" : "-translate-x-full md:translate-x-0"
        )}
      >
        <div className="flex h-16 items-center justify-between border-b border-[var(--color-line)] px-5">
          <span className="text-sm font-semibold uppercase tracking-wider text-[var(--color-muted)]">
            Admin
          </span>
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
        <nav className="flex-1 space-y-0.5 p-4">
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
                  "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors border-l-2",
                  isActive
                    ? "border-[var(--color-accent-gold)] bg-[var(--color-line)]/30 text-[var(--color-ink)]"
                    : "border-transparent text-[var(--color-muted)] hover:border-[var(--color-line)] hover:bg-[var(--color-warm-white)] hover:text-[var(--color-ink)]"
                )}
                aria-current={isActive ? "page" : undefined}
              >
                <Icon className="h-5 w-5 shrink-0" />
                {label}
              </Link>
            );
          })}
        </nav>
        <div className="border-t border-[var(--color-line)] space-y-0.5 p-4">
          <Link
            href={STORE_URL}
            className="flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm text-[var(--color-muted)] transition-colors hover:bg-[var(--color-warm-white)] hover:text-[var(--color-ink)]"
          >
            ← Back to store
          </Link>
          <button
            type="button"
            onClick={() => signOut({ callbackUrl: "/imtheboss/login" })}
            className="flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-sm text-[var(--color-muted)] transition-colors hover:bg-[var(--color-warm-white)] hover:text-[var(--color-ink)] text-left"
          >
            <LogOut className="h-4 w-4 shrink-0" />
            Sign out
          </button>
        </div>
      </aside>

      <main className="flex flex-1 flex-col min-w-0 md:ml-64">
        <header className="sticky top-0 z-20 flex h-16 items-center gap-3 border-b border-[var(--color-line)] bg-[var(--color-ivory)]/95 px-4 md:px-6 backdrop-blur">
          <Button
            variant="outline"
            size="icon"
            onClick={() => setSidebarOpen(true)}
            className="md:hidden rounded-lg border-[var(--color-line)]"
            aria-label="Open menu"
          >
            <Menu className="h-5 w-5" />
          </Button>
          <SectionHeading className="text-xl md:text-2xl">{title}</SectionHeading>
        </header>
        <div className="flex-1 p-4 md:p-6 lg:p-8">{children}</div>
      </main>
    </div>
  );
}
