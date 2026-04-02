import { formatInrFromPaise, paiseToRupeesInput } from "@/lib/money";
import type { CustomerListRow, OrderListRow } from "@/lib/admin-queries";

export function formatCreateDate(createDate: string): string {
  try {
    const d = new Date(createDate);
    if (Number.isNaN(d.getTime())) return createDate;
    return d.toLocaleDateString("en-IN", {
      day: "2-digit",
      month: "short",
      year: "numeric",
    });
  } catch {
    return createDate;
  }
}

export function formatCurrency(paise: number): string {
  return formatInrFromPaise(paise);
}

export function aggregateOrderStats(
  orders: OrderListRow[]
): Map<string, { count: number; totalPaise: number }> {
  const map = new Map<string, { count: number; totalPaise: number }>();
  for (const o of orders) {
    const cur = map.get(o.userId) ?? { count: 0, totalPaise: 0 };
    cur.count += 1;
    cur.totalPaise += parseInt(o.totalAmountPaise, 10) || 0;
    map.set(o.userId, cur);
  }
  return map;
}

export function downloadCustomersCsv(
  rows: CustomerListRow[],
  stats: Map<string, { count: number; totalPaise: number }>
): void {
  const headers = [
    "User ID",
    "Email",
    "Name",
    "Auth",
    "Address",
    "Phone",
    "Created",
    "Orders",
    "Total spent (Rs)",
  ];
  const lines = [
    headers.join(","),
    ...rows.map((c) => {
      const s = stats.get(c.userId);
      const escaped = (v: string | null | undefined) =>
        v == null ? "" : `"${String(v).replace(/"/g, '""')}"`;
      return [
        c.userId,
        escaped(c.email),
        escaped(c.fullName ?? c.username),
        c.authProvider,
        escaped(c.address),
        escaped(c.phone),
        formatCreateDate(c.createDate),
        s?.count ?? 0,
        s ? paiseToRupeesInput(s.totalPaise) : "0.00",
      ].join(",");
    }),
  ];
  const blob = new Blob([lines.join("\n")], { type: "text/csv;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `customers-${new Date().toISOString().slice(0, 10)}.csv`;
  a.click();
  URL.revokeObjectURL(url);
}
