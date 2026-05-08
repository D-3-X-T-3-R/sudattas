"use client";

import { useState } from "react";
import Link from "next/link";
import Image from "next/image";
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
  ExternalLink,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { publicEnv } from "@/lib/env/public";

const ADMIN_BASE = "/imtheboss";
const STORE_URL = publicEnv.NEXT_PUBLIC_STORE_URL || "/";

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
    <div className="flex min-h-screen bg-[var(--admin-surface)] text-[var(--foreground)]">
      <a
        href="#admin-main-content"
        className="sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-50 focus:rounded-md focus:bg-white focus:px-3 focus:py-2 focus:text-sm focus:text-[var(--color-ink)] focus:shadow"
      >
        Skip to main content
      </a>

      {sidebarOpen ? (
        <button
          type="button"
          onClick={() => setSidebarOpen(false)}
          className="fixed inset-0 z-30 bg-black/30 md:hidden"
          aria-label="Close menu"
        />
      ) : null}

      <aside
        aria-label="Admin sidebar"
        className={cn(
          "fixed inset-y-0 left-0 z-40 flex w-[268px] flex-col border-r border-[var(--admin-border-subtle)] bg-[var(--admin-sidebar-bg)] transition-transform duration-200",
          sidebarOpen ? "translate-x-0" : "-translate-x-full md:translate-x-0"
        )}
      >
        <div className="flex h-16 items-center justify-between border-b border-[var(--admin-border-subtle)] px-4">
          <Link href={ADMIN_BASE} className="flex items-center gap-3">
            <Image
              src="/logo.png"
              alt="Sudatta's"
              width={120}
              height={34}
              className="h-7 w-auto"
            />
          </Link>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setSidebarOpen(false)}
            className="md:hidden"
            aria-label="Close menu"
          >
            <X className="h-5 w-5" />
          </Button>
        </div>

        <nav className="flex-1 space-y-1 p-3" aria-label="Admin navigation">
          {NAV.map(({ href, icon: Icon, label }) => {
            const isActive =
              href === ADMIN_BASE
                ? pathname === ADMIN_BASE || pathname === `${ADMIN_BASE}/`
                : pathname.startsWith(href);

            return (
              <Link
                key={href}
                href={href}
                onClick={() => setSidebarOpen(false)}
                className={cn(
                  "flex items-center gap-3 rounded-md border px-3 py-2.5 text-sm font-semibold uppercase tracking-[0.12em]",
                  isActive
                    ? "border-[var(--admin-sidebar-accent)] bg-[#EAF1EE] text-[var(--color-green)]"
                    : "border-transparent text-[var(--admin-sidebar-text-muted)] hover:border-[var(--color-line)] hover:bg-[var(--admin-sidebar-hover)] hover:text-[var(--admin-sidebar-text)]"
                )}
                aria-current={isActive ? "page" : undefined}
              >
                <Icon className="h-4 w-4 shrink-0" />
                {label}
              </Link>
            );
          })}
        </nav>

        <div className="space-y-1 border-t border-[var(--admin-border-subtle)] p-3">
          <Link
            href={STORE_URL}
            className="flex items-center gap-2 rounded-md border border-transparent px-3 py-2 text-xs font-semibold uppercase tracking-[0.12em] text-[var(--admin-sidebar-text-muted)] hover:border-[var(--color-line)] hover:bg-[var(--admin-sidebar-hover)] hover:text-[var(--admin-sidebar-text)]"
          >
            <ExternalLink className="h-3.5 w-3.5" />
            Back To Store
          </Link>
          <button
            type="button"
            onClick={() => signOut({ callbackUrl: "/imtheboss/login" })}
            className="flex w-full items-center gap-2 rounded-md border border-transparent px-3 py-2 text-left text-xs font-semibold uppercase tracking-[0.12em] text-[var(--admin-sidebar-text-muted)] hover:border-[var(--color-line)] hover:bg-[var(--admin-sidebar-hover)] hover:text-[var(--admin-sidebar-text)]"
          >
            <LogOut className="h-3.5 w-3.5" />
            Sign Out
          </button>
        </div>
      </aside>

      <main id="admin-main-content" className="flex min-w-0 flex-1 flex-col md:ml-[268px]">
        <header className="sticky top-0 z-20 flex h-16 items-center gap-3 border-b border-[var(--admin-border-subtle)] bg-[var(--admin-surface)]/95 px-4 backdrop-blur md:px-6">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setSidebarOpen(true)}
            className="md:hidden"
            aria-label="Open menu"
          >
            <Menu className="h-5 w-5" />
          </Button>
          <div>
            <p className="text-[10px] font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">Sudatta&apos;s Admin</p>
            <h1 className="font-display text-xl leading-none text-[var(--color-ink)] md:text-2xl">{title}</h1>
          </div>
        </header>

        <div className="flex-1 p-4 md:p-6 lg:p-7">{children}</div>
      </main>
    </div>
  );
}
