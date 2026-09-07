"use client";

import Link from "next/link";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { Download, Filter } from "lucide-react";
import { DATE_PRESETS, type DatePreset } from "@/domains/admin/orders/types";
import { formatOrderStatusName } from "@/domains/admin/orders/utils";
import { AdminFilterCard } from "@/components/admin/admin-cards";

type OrdersFiltersCardProps = {
  datePreset: DatePreset;
  setDatePreset: (value: DatePreset) => void;
  statusId: string;
  setStatusId: (value: string) => void;
  pageSize: number;
  setPageSize: (value: number) => void;
  setPage: (value: number) => void;
  statuses: { statusId: string; statusName: string }[];
  userIdFromUrl?: string;
  onRefresh: () => void;
  onExportCsv: () => void;
  exportDisabled: boolean;
};

export function OrdersFiltersCard({
  datePreset,
  setDatePreset,
  statusId,
  setStatusId,
  pageSize,
  setPageSize,
  setPage,
  statuses,
  userIdFromUrl,
  onRefresh,
  onExportCsv,
  exportDisabled,
}: OrdersFiltersCardProps) {
  return (
    <AdminFilterCard title="Filters" icon={<Filter className="h-4 w-4 text-[var(--color-green)]" />}>
      <div className="flex flex-wrap items-end gap-5">
        <div>
          <p className="mb-1.5 text-sm font-medium text-[var(--color-muted)]">Date range</p>
          <div className="flex flex-wrap gap-2">
            {DATE_PRESETS.map(({ key, label }) => (
              <Button
                key={key}
                type="button"
                variant={datePreset === key ? "default" : "outline"}
                onClick={() => {
                  setPage(1);
                  setDatePreset(key);
                }}
              >
                {label}
              </Button>
            ))}
          </div>
        </div>

        {userIdFromUrl ? (
          <div className="flex items-center gap-2 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-soft)] px-4 py-2.5 text-sm">
            <span className="text-[var(--color-muted)]">Customer:</span>
            <span className="text-[var(--color-ink)]">{userIdFromUrl}</span>
            <Link href="/imtheboss/orders" className="text-[var(--color-green)] hover:underline">
              Clear
            </Link>
          </div>
        ) : null}

        <div>
          <label htmlFor="orders-status" className="mb-1.5 block text-sm font-medium text-[var(--color-muted)]">
            Status
          </label>
          <select
            id="orders-status"
            className={cn(
              "h-11 min-w-[11rem] rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px]",
              "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
            )}
            value={statusId}
            onChange={(e) => {
              setPage(1);
              setStatusId(e.target.value);
            }}
          >
            <option value="">All statuses</option>
            {statuses.map((s) => (
              <option key={s.statusId} value={s.statusId}>
                {formatOrderStatusName(s.statusName)}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label htmlFor="orders-page-size" className="mb-1.5 block text-sm font-medium text-[var(--color-muted)]">
            Per page
          </label>
          <select
            id="orders-page-size"
            className={cn(
              "h-11 min-w-[6.5rem] rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px]",
              "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
            )}
            value={String(pageSize)}
            onChange={(e) => {
              setPage(1);
              setPageSize(Number(e.target.value));
            }}
          >
            <option value="20">20</option>
            <option value="50">50</option>
            <option value="100">100</option>
          </select>
        </div>

        <Button type="button" variant="outline" onClick={onRefresh}>
          Refresh
        </Button>

        <Button type="button" variant="outline" onClick={onExportCsv} disabled={exportDisabled}>
          <Download className="mr-1.5 h-4 w-4" />
          Export CSV
        </Button>
      </div>
    </AdminFilterCard>
  );
}
