import type { DatePreset } from "@/domains/admin/orders/types";

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
  return s ? s.statusName : statusId;
}
