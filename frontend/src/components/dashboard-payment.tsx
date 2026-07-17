"use client";

import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { ResponsiveLine } from "@nivo/line";
import { Card, CardContent } from "@/components/ui/card";
import { DonutCard, type Slice } from "@/components/dashboard-donuts";
import { fetchAllOrdersList, fetchOrderStatuses } from "@/lib/admin-queries";
import { formatInrFromPaise } from "@/lib/money";
import { Banknote, CreditCard } from "lucide-react";

// Chart chrome tokens — shared with dashboard-charts.tsx's mark specs (hairline grid, muted axis).
const GRID_LINE = "#e1e0d9";
const AXIS_MUTED = "#898781";
const chartTheme = {
  axis: { ticks: { text: { fill: AXIS_MUTED, fontSize: 13 } } },
  grid: { line: { stroke: GRID_LINE, strokeWidth: 1 } },
};

// Fixed brand colors for the two payment modes — stable across the donut, the trend line, and the matrix.
const COD_HEX = "#C9A646";
const PREPAID_HEX = "#1E5B43";
const OTHER_HEX = "#c3c2b7";

function monthKey(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
}
function monthLabel(key: string): string {
  const [y, m] = key.split("-");
  return new Date(Number(y), Number(m) - 1, 1).toLocaleDateString("en-IN", { month: "short", year: "2-digit" });
}
function normalizeMode(pm: string | null): "cod" | "prepaid" | "other" {
  const v = (pm ?? "").trim().toLowerCase();
  return v === "cod" || v === "prepaid" ? v : "other";
}

export interface ModeStats {
  count: number;
  revenuePaise: number;
  aovPaise: number;
  cancellationRate: number;
}

function usePaymentData() {
  const ordersQuery = useQuery({
    queryKey: ["admin", "orders", "all-for-stats"],
    queryFn: () => fetchAllOrdersList(),
    staleTime: 2 * 60 * 1000,
  });
  const { data: statuses = [] } = useQuery({
    queryKey: ["admin", "order-statuses"],
    queryFn: fetchOrderStatuses,
  });

  const aggregates = useMemo(() => {
    const allOrders = ordersQuery.data ?? [];
    const idToStatusName = new Map(statuses.map((s) => [s.statusId, s.statusName.trim().toLowerCase()]));

    let codCount = 0;
    let prepaidCount = 0;
    let otherCount = 0;
    let codRevenue = 0;
    let prepaidRevenue = 0;
    let codCancelled = 0;
    let prepaidCancelled = 0;

    const now = new Date();
    const bucketKeys: string[] = [];
    for (let i = 11; i >= 0; i--) bucketKeys.push(monthKey(new Date(now.getFullYear(), now.getMonth() - i, 1)));
    const codByMonth = new Map(bucketKeys.map((k) => [k, 0]));
    const prepaidByMonth = new Map(bucketKeys.map((k) => [k, 0]));

    for (const o of allOrders) {
      const mode = normalizeMode(o.paymentMethod);
      const paise = Number.parseInt(o.totalAmountPaise, 10) || 0;
      const isCancelled = idToStatusName.get(o.statusId) === "cancelled";

      if (mode === "cod") {
        codCount++;
        codRevenue += paise;
        if (isCancelled) codCancelled++;
      } else if (mode === "prepaid") {
        prepaidCount++;
        prepaidRevenue += paise;
        if (isCancelled) prepaidCancelled++;
      } else {
        otherCount++;
      }

      const d = new Date(o.orderDate);
      if (!Number.isNaN(d.getTime())) {
        const k = monthKey(d);
        if (mode === "cod" && codByMonth.has(k)) codByMonth.set(k, (codByMonth.get(k) ?? 0) + 1);
        else if (mode === "prepaid" && prepaidByMonth.has(k)) prepaidByMonth.set(k, (prepaidByMonth.get(k) ?? 0) + 1);
      }
    }

    const donutSlices: Slice[] = [
      { id: "cod", label: "COD", value: codCount, color: COD_HEX },
      { id: "prepaid", label: "Prepaid", value: prepaidCount, color: PREPAID_HEX },
    ];
    if (otherCount > 0) donutSlices.push({ id: "other", label: "Other", value: otherCount, color: OTHER_HEX });

    const trend = bucketKeys.map((k) => ({
      month: monthLabel(k),
      cod: codByMonth.get(k) ?? 0,
      prepaid: prepaidByMonth.get(k) ?? 0,
    }));

    const cod: ModeStats = {
      count: codCount,
      revenuePaise: codRevenue,
      aovPaise: codCount > 0 ? Math.round(codRevenue / codCount) : 0,
      cancellationRate: codCount > 0 ? (codCancelled / codCount) * 100 : 0,
    };
    const prepaid: ModeStats = {
      count: prepaidCount,
      revenuePaise: prepaidRevenue,
      aovPaise: prepaidCount > 0 ? Math.round(prepaidRevenue / prepaidCount) : 0,
      cancellationRate: prepaidCount > 0 ? (prepaidCancelled / prepaidCount) * 100 : 0,
    };

    return { donutSlices, trend, cod, prepaid };
  }, [ordersQuery.data, statuses]);

  return { ...aggregates, isLoading: ordersQuery.isLoading, isError: ordersQuery.isError };
}

export function PaymentTrendChart({ trend }: { trend: { month: string; cod: number; prepaid: number }[] }) {
  const maxVal = useMemo(() => Math.max(1, ...trend.flatMap((t) => [t.cod, t.prepaid])), [trend]);
  const series = useMemo(
    () => [
      { id: "COD", color: COD_HEX, data: trend.map((t) => ({ x: t.month, y: t.cod })) },
      { id: "Prepaid", color: PREPAID_HEX, data: trend.map((t) => ({ x: t.month, y: t.prepaid })) },
    ],
    [trend]
  );

  return (
    <div style={{ height: 300 }}>
      <ResponsiveLine
        data={series}
        margin={{ top: 30, right: 24, bottom: 44, left: 44 }}
        xScale={{ type: "point" }}
        yScale={{ type: "linear", min: 0, max: maxVal, clamp: true }}
        theme={chartTheme}
        colors={{ datum: "color" }}
        lineWidth={2}
        pointSize={8}
        pointColor={{ theme: "background" }}
        pointBorderWidth={2}
        pointBorderColor={{ from: "serieColor" }}
        enableArea={false}
        enableGridX={false}
        gridYValues={4}
        axisBottom={{ tickSize: 0, tickPadding: 10, tickRotation: -30 }}
        axisLeft={{ tickSize: 0, tickPadding: 8, tickValues: 4 }}
        useMesh
        enableSlices="x"
        sliceTooltip={({ slice }) => (
          <div className="rounded-lg border border-[var(--color-line)] bg-white px-3.5 py-2.5 shadow-[0_8px_24px_rgba(45,42,38,0.12)]">
            <p className="text-sm text-[var(--color-muted)]">{String(slice.points[0]?.data.x ?? "")}</p>
            {slice.points.map((p) => (
              <p key={p.seriesId} className="mt-1 flex items-center gap-1.5 text-base font-semibold text-[var(--color-ink)]">
                <span className="inline-block h-2.5 w-2.5 rounded-full" style={{ backgroundColor: String(p.seriesColor) }} />
                {p.seriesId}: {String(p.data.y)}
              </p>
            ))}
          </div>
        )}
        legends={[
          {
            anchor: "top-left",
            direction: "row",
            translateY: -26,
            itemWidth: 80,
            itemHeight: 20,
            symbolSize: 10,
            symbolShape: "circle",
            itemTextColor: "var(--color-ink)",
          },
        ]}
      />
    </div>
  );
}

export function PaymentModeCard({
  label,
  icon,
  color,
  stats,
}: {
  label: string;
  icon: React.ReactNode;
  color: string;
  stats: ModeStats;
}) {
  return (
    <Card className="rounded-2xl border-[var(--color-line)] bg-white p-5 shadow-[var(--admin-card-shadow)]">
      <div className="flex items-center gap-2.5">
        <span
          className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-lg"
          style={{ backgroundColor: `${color}1A`, color }}
        >
          {icon}
        </span>
        <p className="text-[15px] font-semibold text-[var(--color-ink)]">{label}</p>
      </div>
      <dl className="mt-4 grid grid-cols-2 gap-4">
        <div>
          <dt className="text-sm text-[var(--color-muted)]">Orders</dt>
          <dd className="mt-0.5 text-xl font-semibold text-[var(--color-ink)]">{stats.count}</dd>
        </div>
        <div>
          <dt className="text-sm text-[var(--color-muted)]">Revenue</dt>
          <dd className="mt-0.5 text-xl font-semibold text-[var(--color-ink)]">{formatInrFromPaise(stats.revenuePaise)}</dd>
        </div>
        <div>
          <dt className="text-sm text-[var(--color-muted)]">Avg order value</dt>
          <dd className="mt-0.5 text-xl font-semibold text-[var(--color-ink)]">{formatInrFromPaise(stats.aovPaise)}</dd>
        </div>
        <div>
          <dt className="text-sm text-[var(--color-muted)]">Cancelled</dt>
          <dd className="mt-0.5 text-xl font-semibold text-[var(--color-ink)]">{stats.cancellationRate.toFixed(0)}%</dd>
        </div>
      </dl>
    </Card>
  );
}

/** How customers pay: mode split, trend over time, and a revenue/orders/AOV/cancellation matrix by mode. */
export function DashboardPayment() {
  const { donutSlices, trend, cod, prepaid, isLoading, isError } = usePaymentData();

  return (
    <div className="mt-10 space-y-4">
      <div>
        <p className="text-base font-semibold text-[var(--color-ink)]">Payment behavior</p>
        <p className="mt-1 text-sm text-[var(--color-muted)]">How customers choose to pay, and how that&apos;s trending.</p>
      </div>

      {isError ? (
        <div className="rounded-md border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
          Could not load payment data.
        </div>
      ) : isLoading ? (
        <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading payment data...</p>
      ) : (
        <>
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,1fr)_2fr]">
            <DonutCard title="Payment mode split" subtitle="Cash on delivery vs. paid online" slices={donutSlices} />
            <Card className="overflow-hidden rounded-2xl border-[var(--color-line)] bg-white shadow-[var(--admin-card-shadow)]">
              <div className="p-5 pb-0">
                <p className="text-[15px] font-semibold text-[var(--color-ink)]">Payment mode trend</p>
                <p className="mt-0.5 text-sm text-[var(--color-muted)]">Orders by mode, last 12 months</p>
              </div>
              <CardContent className="mt-2 pt-2">
                <PaymentTrendChart trend={trend} />
              </CardContent>
            </Card>
          </div>

          <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
            <PaymentModeCard label="Cash on delivery" icon={<Banknote className="h-5 w-5" />} color={COD_HEX} stats={cod} />
            <PaymentModeCard label="Prepaid" icon={<CreditCard className="h-5 w-5" />} color={PREPAID_HEX} stats={prepaid} />
          </div>
        </>
      )}
    </div>
  );
}
