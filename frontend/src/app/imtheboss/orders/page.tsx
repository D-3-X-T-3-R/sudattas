"use client";

import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { SectionHeading } from "@/components/ui/typography";
import { fetchOrdersList, fetchOrderStatuses, type OrderListRow } from "@/lib/admin-queries";
import { toRouteFailureUi } from "@/lib/route-state";
import { ListOrdered } from "lucide-react";
import { OrdersFiltersCard } from "@/domains/admin/orders/components/orders-filters-card";
import { OrdersTableCard } from "@/domains/admin/orders/components/orders-table-card";
import { getDateRange } from "@/domains/admin/orders/utils";
import type { DatePreset } from "@/domains/admin/orders/types";

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
    <div className="mx-auto w-full max-w-6xl">
      <div className="mb-8">
        <p className="text-sm text-[var(--color-muted)]">Orders</p>
        <SectionHeading size="default" className="mt-1">
          Order management
        </SectionHeading>
        <p className="mt-1 text-sm leading-relaxed text-[var(--color-muted)]">
          Filter by date and status, or view orders for a specific customer.
        </p>
      </div>

      <div className="mb-6 flex flex-wrap gap-3">
        <span className="inline-flex items-center gap-2 rounded-full bg-blue-500/12 px-4 py-2 text-sm font-medium text-blue-700">
          <ListOrdered className="h-4 w-4" />
          {orders.length} order{orders.length !== 1 ? "s" : ""}
        </span>
      </div>

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
    </div>
  );
}
