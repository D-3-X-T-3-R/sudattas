"use client";

import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { fetchOrdersList, fetchOrderStatuses, type OrderListRow } from "@/lib/admin-queries";
import { toRouteFailureUi } from "@/lib/route-state";
import { ListOrdered } from "lucide-react";
import { OrdersFiltersCard } from "@/domains/admin/orders/components/orders-filters-card";
import { OrdersTableCard } from "@/domains/admin/orders/components/orders-table-card";
import { getDateRange } from "@/domains/admin/orders/utils";
import type { DatePreset } from "@/domains/admin/orders/types";
import { AdminPageShell } from "@/components/admin/admin-page-shell";

const MAX_ORDER_PAGE_SIZE = 100;

export default function AdminOrdersPage() {
  const userIdFromUrl =
    typeof window !== "undefined"
      ? new URLSearchParams(window.location.search).get("userId") ?? undefined
      : undefined;

  const [datePreset, setDatePreset] = useState<DatePreset>("30");
  const [statusId, setStatusId] = useState("");
  const [page, setPageRaw] = useState(1);
  const [pageSize, setPageSize] = useState(50);

  const filters = useMemo(() => {
    const dateRange = getDateRange(datePreset);
    const safePageSize = Math.min(Math.max(pageSize, 10), MAX_ORDER_PAGE_SIZE);
    const offset = String((page - 1) * safePageSize);
    return {
      ...dateRange,
      statusId: statusId.trim() || undefined,
      userId: userIdFromUrl,
      limit: String(safePageSize),
      offset,
    };
  }, [datePreset, statusId, userIdFromUrl, page, pageSize]);

  const { data: statuses = [] } = useQuery({
    queryKey: ["admin", "order-statuses"],
    queryFn: fetchOrderStatuses,
  });

  const {
    data: orders = [],
    isLoading,
    isError,
    error,
    refetch,
  } = useQuery<OrderListRow[], Error>({
    queryKey: ["admin", "orders", filters],
    queryFn: () => fetchOrdersList(filters),
  });

  const ordersErrorUi = isError ? toRouteFailureUi("admin", error) : null;

  return (
    <AdminPageShell
      label="Orders"
      title="Order management"
      description="Filter by date, status, and customer to manage fulfillment efficiently."
      action={
        <span className="inline-flex items-center gap-2 rounded-md border border-[var(--color-line)] bg-[var(--color-surface-soft)] px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.14em] text-[var(--color-ink)]">
          <ListOrdered className="h-3.5 w-3.5" />
          {orders.length} orders
        </span>
      }
    >
      <OrdersFiltersCard
        datePreset={datePreset}
        setDatePreset={setDatePreset}
        statusId={statusId}
        setStatusId={setStatusId}
        pageSize={pageSize}
        setPageSize={setPageSize}
        setPage={setPageRaw}
        statuses={statuses}
        userIdFromUrl={userIdFromUrl}
        onRefresh={() => refetch()}
      />

      <OrdersTableCard
        isLoading={isLoading}
        isError={isError}
        errorTitle={ordersErrorUi?.title}
        errorMessage={ordersErrorUi?.message}
        orders={orders}
        statuses={statuses}
        page={page}
        pageSize={pageSize}
        setPage={setPageRaw}
      />
    </AdminPageShell>
  );
}
