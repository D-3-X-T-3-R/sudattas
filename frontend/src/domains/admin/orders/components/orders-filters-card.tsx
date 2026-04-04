"use client";

import Link from "next/link";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { Filter } from "lucide-react";
import { DATE_PRESETS, type DatePreset } from "@/domains/admin/orders/types";

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
}: OrdersFiltersCardProps) {
  return (
    <Card className="rounded-xl border-[var(--color-line)] border-l-4 border-l-blue-500 bg-white shadow-[var(--admin-card-shadow)]">
      <CardTitle className="flex items-center gap-2 text-[var(--color-muted)]">
        <Filter className="h-4 w-4 text-blue-500" />
        Filters
      </CardTitle>
      <CardContent className="mt-3 flex flex-wrap items-center gap-3">
        <div className="flex flex-wrap gap-2">
          {DATE_PRESETS.map(({ key, label }) => (
            <Button
              key={key}
              type="button"
              variant={datePreset === key ? "default" : "outline"}
              size="sm"
              onClick={() => {
                setPage(1);
                setDatePreset(key);
              }}
            >
              {label}
            </Button>
          ))}
        </div>
        {userIdFromUrl && (
          <div className="flex items-center gap-2 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-1.5 text-sm">
            <span className="text-[var(--color-muted)]">Customer:</span>
            <span className="font-mono text-[var(--color-ink)]">{userIdFromUrl}</span>
            <Link href="/imtheboss/orders" className="text-[var(--color-accent-brown)] hover:underline">
              Clear
            </Link>
          </div>
        )}
        <div className="flex items-center gap-2">
          <label htmlFor="orders-status" className="text-sm text-[var(--color-muted)]">
            Status
          </label>
          <select
            id="orders-status"
            className={cn(
              "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
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
                {s.statusName}
              </option>
            ))}
          </select>
        </div>
        <div className="flex items-center gap-2">
          <label htmlFor="orders-page-size" className="text-sm text-[var(--color-muted)]">
            Per page
          </label>
          <select
            id="orders-page-size"
            className={cn(
              "h-9 min-w-[6rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
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
        <Button type="button" variant="outline" size="sm" onClick={onRefresh}>
          Refresh
        </Button>
      </CardContent>
    </Card>
  );
}
