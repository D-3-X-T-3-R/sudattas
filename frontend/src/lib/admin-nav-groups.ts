/**
 * Single source of truth for the admin panel's grouped nav sections. Each group collapses
 * to one sidebar entry (in admin-shell.tsx) whose page renders a tab bar (AdminGroupTabs)
 * for switching between the group's members — the pages themselves are unchanged, separate
 * routes; this only adds a shared tab affordance across them so they read as one section.
 */

export interface AdminGroupTab {
  href: string;
  label: string;
}

export const CATALOG_GROUP_TABS: AdminGroupTab[] = [
  { href: "/imtheboss/products", label: "Products" },
  { href: "/imtheboss/inventory", label: "Inventory" },
];

export const ORDERS_GROUP_TABS: AdminGroupTab[] = [
  { href: "/imtheboss/orders", label: "Orders" },
  { href: "/imtheboss/shipments", label: "Shipments" },
  // Returns tab temporarily hidden (2026-09-12) while testing the exchange flow in
  // isolation — the page/route itself is untouched, just unreachable via nav. Re-add this
  // line to bring it back.
  // { href: "/imtheboss/returns", label: "Returns" },
  { href: "/imtheboss/exchanges", label: "Exchanges" },
];

export const FINANCE_GROUP_TABS: AdminGroupTab[] = [
  { href: "/imtheboss/payments", label: "Payments" },
  { href: "/imtheboss/refunds", label: "Refunds" },
  { href: "/imtheboss/transactions", label: "Transactions" },
];
