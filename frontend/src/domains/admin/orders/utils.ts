import type { DatePreset } from "@/domains/admin/orders/types";
import type { OrderListRow } from "@/lib/admin-queries";

export function getDateRange(
  preset: DatePreset
): { orderDateStart?: string; orderDateEnd?: string } {
  const now = new Date();
  const end = new Date(now);
  end.setHours(23, 59, 59, 999);
  const endSec = Math.floor(end.getTime() / 1000);

  switch (preset) {
    case "7": {
      const start = new Date(now);
      start.setDate(start.getDate() - 7);
      start.setHours(0, 0, 0, 0);
      return { orderDateStart: String(Math.floor(start.getTime() / 1000)), orderDateEnd: String(endSec) };
    }
    case "30": {
      const start = new Date(now);
      start.setDate(start.getDate() - 30);
      start.setHours(0, 0, 0, 0);
      return { orderDateStart: String(Math.floor(start.getTime() / 1000)), orderDateEnd: String(endSec) };
    }
    case "month": {
      const start = new Date(now.getFullYear(), now.getMonth(), 1, 0, 0, 0, 0);
      return { orderDateStart: String(Math.floor(start.getTime() / 1000)), orderDateEnd: String(endSec) };
    }
    default:
      return {};
  }
}

export function formatOrderDate(orderDate: string): string {
  try {
    const d = new Date(orderDate);
    if (Number.isNaN(d.getTime())) return orderDate;
    return d.toLocaleDateString("en-IN", {
      day: "2-digit",
      month: "short",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return orderDate;
  }
}

export function getStatusLabel(
  statusId: string,
  statuses: { statusId: string; statusName: string }[]
): string {
  const s = statuses.find((x) => x.statusId === statusId);
  return s ? formatOrderStatusName(s.statusName) : statusId;
}

export function formatOrderStatusName(statusName: string): string {
  return statusName.trim().toLowerCase() === "processing" ? "processing order" : statusName;
}

/** Same CSV-blob-download pattern as downloadCustomersCsv (domains/admin/customers/utils.ts). */
export function downloadOrdersCsv(
  rows: OrderListRow[],
  statuses: { statusId: string; statusName: string }[]
): void {
  const headers = ["Order ID", "Date", "Customer ID", "Amount", "Status"];
  const escaped = (v: string | null | undefined) =>
    v == null ? "" : `"${String(v).replace(/"/g, '""')}"`;
  const lines = [
    headers.join(","),
    ...rows.map((o) =>
      [
        o.orderId,
        escaped(formatOrderDate(o.orderDate)),
        o.userId,
        escaped(o.totalAmountFormatted),
        escaped(getStatusLabel(o.statusId, statuses)),
      ].join(",")
    ),
  ];
  const blob = new Blob([lines.join("\n")], { type: "text/csv;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `orders-${new Date().toISOString().slice(0, 10)}.csv`;
  a.click();
  URL.revokeObjectURL(url);
}
