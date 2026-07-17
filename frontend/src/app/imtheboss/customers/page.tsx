"use client";

import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  fetchAllCustomersList,
  fetchAllOrdersList,
  type CustomerListRow,
  type OrderListRow,
} from "@/lib/admin-queries";
import { toRouteFailureUi } from "@/lib/route-state";
import { ArrowDown, ArrowUp, ArrowUpDown, Users } from "lucide-react";
import { CustomersFiltersCard } from "@/domains/admin/customers/components/customers-filters-card";
import { CustomersTableCard } from "@/domains/admin/customers/components/customers-table-card";
import { CustomerProfileDialog } from "@/domains/admin/customers/components/customer-profile-dialog";
import { aggregateOrderStats, downloadCustomersCsv } from "@/domains/admin/customers/utils";
import { AdminPageShell } from "@/components/admin/admin-page-shell";

type SortKey = "name" | "email" | "created" | "orders" | "spent";
type SortDir = "asc" | "desc";
const MAX_CUSTOMER_PAGE_SIZE = 100;

export default function AdminCustomersPage() {
  const [searchQuery, setSearchQuery] = useState("");
  const [filterAuth, setFilterAuth] = useState("");
  const [sortKey, setSortKey] = useState<SortKey>("created");
  const [sortDir, setSortDir] = useState<SortDir>("desc");
  const [page, setPageRaw] = useState(1);
  const [pageSize, setPageSize] = useState(50);
  const [selectedCustomer, setSelectedCustomer] = useState<CustomerListRow | null>(null);

  const {
    data: customers = [],
    isLoading,
    isError,
    error,
    refetch,
  } = useQuery<CustomerListRow[], Error>({
    queryKey: ["admin", "customers"],
    queryFn: fetchAllCustomersList,
  });
  const customersErrorUi = isError ? toRouteFailureUi("admin", error) : null;

  const { data: allOrders = [] } = useQuery<OrderListRow[], Error>({
    queryKey: ["admin", "orders", "all-for-stats"],
    queryFn: () => fetchAllOrdersList(),
    enabled: !isError && customers.length > 0,
  });

  const orderStats = useMemo(() => aggregateOrderStats(allOrders), [allOrders]);

  const filteredAndSorted = useMemo(() => {
    const q = searchQuery.trim().toLowerCase();
    const auth = filterAuth.trim();
    let list = customers.filter((c) => {
      if (auth && c.authProvider !== auth) return false;
      if (!q) return true;
      const name = (c.fullName ?? c.username ?? "").toLowerCase();
      const email = (c.email ?? "").toLowerCase();
      const uid = (c.userId ?? "").toLowerCase();
      return name.includes(q) || email.includes(q) || uid.includes(q);
    });
    list = [...list].sort((a, b) => {
      let cmp = 0;
      const sa = orderStats.get(a.userId);
      const sb = orderStats.get(b.userId);
      switch (sortKey) {
        case "name":
          cmp = (a.fullName ?? a.username ?? "").localeCompare(b.fullName ?? b.username ?? "");
          break;
        case "email":
          cmp = (a.email ?? "").localeCompare(b.email ?? "");
          break;
        case "created":
          cmp = new Date(a.createDate).getTime() - new Date(b.createDate).getTime();
          break;
        case "orders":
          cmp = (sa?.count ?? 0) - (sb?.count ?? 0);
          break;
        case "spent":
          cmp = (sa?.totalPaise ?? 0) - (sb?.totalPaise ?? 0);
          break;
      }
      return sortDir === "asc" ? cmp : -cmp;
    });
    return list;
  }, [customers, searchQuery, filterAuth, sortKey, sortDir, orderStats]);

  const pagedCustomers = useMemo(() => {
    const safeSize = Math.min(Math.max(pageSize, 10), MAX_CUSTOMER_PAGE_SIZE);
    const start = (page - 1) * safeSize;
    return filteredAndSorted.slice(start, start + safeSize);
  }, [filteredAndSorted, page, pageSize]);

  const handleSort = (key: SortKey) => {
    if (sortKey === key) setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    else {
      setSortKey(key);
      setSortDir(key === "name" || key === "email" ? "asc" : "desc");
    }
  };

  const sortIconFor = (column: SortKey) =>
    sortKey !== column ? (
      <ArrowUpDown className="ml-1 inline h-3.5 w-3 opacity-50" />
    ) : sortDir === "asc" ? (
      <ArrowUp className="ml-1 inline h-3.5 w-3" />
    ) : (
      <ArrowDown className="ml-1 inline h-3.5 w-3" />
    );

  return (
    <AdminPageShell
      label="Customers"
      title="Customer directory"
      description="Search, filter, and inspect customer history with fast order context."
      action={
        <span className="inline-flex items-center gap-2 rounded-full border border-[var(--color-line)] bg-[var(--color-surface-soft)] px-4 py-2 text-sm font-semibold text-[var(--color-ink)]">
          <Users className="h-4 w-4" />
          {filteredAndSorted.length} customers
        </span>
      }
    >
      <CustomersFiltersCard
        searchQuery={searchQuery}
        setSearchQuery={setSearchQuery}
        filterAuth={filterAuth}
        setFilterAuth={setFilterAuth}
        pageSize={pageSize}
        setPageSize={setPageSize}
        setPage={setPageRaw}
        onRefresh={() => refetch()}
        onExportCsv={() => downloadCustomersCsv(filteredAndSorted, orderStats)}
        exportDisabled={filteredAndSorted.length === 0}
      />

      <CustomersTableCard
        isLoading={isLoading}
        isError={isError}
        errorTitle={customersErrorUi?.title}
        errorMessage={customersErrorUi?.message}
        onRetry={() => refetch()}
        customersLength={customers.length}
        filteredLength={filteredAndSorted.length}
        pagedCustomers={pagedCustomers}
        orderStats={orderStats}
        selectedCustomer={selectedCustomer}
        setSelectedCustomer={setSelectedCustomer}
        handleSort={handleSort}
        sortIconFor={sortIconFor}
        sortKey={sortKey}
        sortDir={sortDir}
        page={page}
        pageSize={pageSize}
        setPage={setPageRaw}
      />

      <CustomerProfileDialog
        selectedCustomer={selectedCustomer}
        setSelectedCustomer={setSelectedCustomer}
        orderStats={orderStats}
      />
    </AdminPageShell>
  );
}
