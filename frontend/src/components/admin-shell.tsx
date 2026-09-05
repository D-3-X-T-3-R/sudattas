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
  Boxes,
  Truck,
  Settings,
  Menu,
  X,
  LogOut,
  ExternalLink,
  MessageSquareText,
  Receipt,
  ScrollText,
  Mail,
  Tag,
  Undo2,
  Banknote,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { publicEnv } from "@/lib/env/public";

const ADMIN_BASE = "/imtheboss";
const STORE_URL = publicEnv.NEXT_PUBLIC_STORE_URL || "/";

const NAV = [
  { href: `${ADMIN_BASE}`, icon: LayoutDashboard, label: "Dashboard" },
  { href: `${ADMIN_BASE}/orders`, icon: ShoppingCart, label: "Orders" },
  { href: `${ADMIN_BASE}/shipments`, icon: Truck, label: "Shipments" },
  { href: `${ADMIN_BASE}/products`, icon: Package, label: "Products" },
  { href: `${ADMIN_BASE}/coupons`, icon: Tag, label: "Coupons" },
  { href: `${ADMIN_BASE}/inventory`, icon: Boxes, label: "Inventory" },
  { href: `${ADMIN_BASE}/customers`, icon: Users, label: "Customers" },
  { href: `${ADMIN_BASE}/reviews`, icon: MessageSquareText, label: "Reviews" },
  { href: `${ADMIN_BASE}/returns`, icon: Undo2, label: "Returns" },
  { href: `${ADMIN_BASE}/refunds`, icon: Banknote, label: "Refunds" },
  { href: `${ADMIN_BASE}/transactions`, icon: Receipt, label: "Transactions" },
  { href: `${ADMIN_BASE}/newsletter`, icon: Mail, label: "Newsletter" },
  { href: `${ADMIN_BASE}/activity-log`, icon: ScrollText, label: "Activity log" },
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
    shipments: "Shipments",
    products: "Products",
    coupons: "Coupons",
    inventory: "Inventory",
    customers: "Customers",
    reviews: "Reviews",
    returns: "Returns",
    refunds: "Refunds",
    transactions: "Transactions",
    newsletter: "Newsletter",
    "activity-log": "Activity log",
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
          "fixed inset-y-0 left-0 z-40 flex w-[288px] flex-col border-r border-[var(--admin-border-subtle)] bg-[var(--admin-sidebar-bg)] transition-transform duration-200",
          sidebarOpen ? "translate-x-0" : "-translate-x-full md:translate-x-0"
        )}
      >
        <div className="flex h-[72px] items-center justify-between border-b border-[var(--admin-border-subtle)] px-5">
          <Link href={ADMIN_BASE} className="flex items-center gap-3">
            <Image
              src="/logo.png"
              alt="Sudatta's"
              width={130}
              height={37}
              className="h-8 w-auto"
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

        <nav className="flex-1 space-y-1.5 p-4" aria-label="Admin navigation">
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
                  "flex items-center gap-3.5 rounded-xl border px-4 py-3.5 text-[15px] font-semibold",
                  isActive
                    ? "border-transparent bg-[var(--color-green)] text-white shadow-[var(--shadow-action)]"
                    : "border-transparent text-[var(--admin-sidebar-text-muted)] hover:bg-[var(--admin-sidebar-hover)] hover:text-[var(--admin-sidebar-text)]"
                )}
                aria-current={isActive ? "page" : undefined}
              >
                <Icon className="h-5 w-5 shrink-0" />
                {label}
              </Link>
            );
          })}
        </nav>

        <div className="space-y-1.5 border-t border-[var(--admin-border-subtle)] p-4">
          <Link
            href={STORE_URL}
            className="flex items-center gap-3 rounded-xl border border-transparent px-4 py-3 text-sm font-semibold text-[var(--admin-sidebar-text-muted)] hover:bg-[var(--admin-sidebar-hover)] hover:text-[var(--admin-sidebar-text)]"
          >
            <ExternalLink className="h-4 w-4" />
            Back to store
          </Link>
          <button
            type="button"
            onClick={() => signOut({ callbackUrl: "/imtheboss/login" })}
            className="flex w-full items-center gap-3 rounded-xl border border-transparent px-4 py-3 text-left text-sm font-semibold text-[var(--admin-sidebar-text-muted)] hover:bg-[var(--admin-sidebar-hover)] hover:text-[var(--admin-sidebar-text)]"
          >
            <LogOut className="h-4 w-4" />
            Sign out
          </button>
        </div>
      </aside>

      <main id="admin-main-content" className="flex min-w-0 flex-1 flex-col md:ml-[288px]">
        <header className="sticky top-0 z-20 flex h-[72px] items-center gap-3 border-b border-[var(--admin-border-subtle)] bg-[var(--admin-surface)]/95 px-4 backdrop-blur md:px-7">
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
            <p className="text-sm font-medium text-[var(--color-muted)]">Sudatta&apos;s Admin</p>
            <h1 className="font-display text-2xl leading-none text-[var(--color-ink)] md:text-[1.7rem]">{title}</h1>
          </div>
        </header>

        <div className="flex-1 p-4 md:p-7 lg:p-8">{children}</div>
      </main>
    </div>
  );
}
