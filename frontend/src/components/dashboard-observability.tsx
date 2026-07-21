"use client";

import { useQuery } from "@tanstack/react-query";
import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  Gauge,
  LogIn,
  PackageSearch,
  RefreshCcw,
  ShieldAlert,
  ShoppingCart,
  Truck,
  Wallet,
} from "lucide-react";
import { fetchTelemetrySummary, type TelemetrySummary } from "@/lib/admin-queries";
import { CollapsibleSection } from "@/components/admin/collapsible-section";
import { cn } from "@/lib/utils";

function pct(v: number): string {
  return `${v.toFixed(2)}%`;
}

/** Plain-language read of the same telemetry, for a non-technical shop owner glancing at the dashboard. */
function summarizeStoreHealth(data: TelemetrySummary): { ok: boolean; issues: string[] } {
  const issues: string[] = [];
  const failedLogins = data.loginFailureRate.numerator;
  if (failedLogins > 0) issues.push(`${failedLogins} failed sign-in attempt${failedLogins === 1 ? "" : "s"}`);
  if (data.checkoutFailureRate.numerator > 0) issues.push("some checkouts did not go through");
  if ((data.backendSignals?.refundFailureCount?.value ?? 0) > 0) issues.push("a refund needs attention");
  if ((data.backendSignals?.shiprocketBookingFailureCount?.value ?? 0) > 0) issues.push("a shipment could not be booked automatically");
  if ((data.backendSignals?.stuckPendingOrders?.value ?? 0) > 0) issues.push("some orders have been pending a while");
  if (data.releaseConfidence.score < 80) issues.push("overall store health is lower than usual");
  return { ok: issues.length === 0, issues };
}

export function DashboardObservability() {
  const { data, isLoading, isError, error } = useQuery({
    queryKey: ["admin", "dashboard-observability"],
    queryFn: fetchTelemetrySummary,
    staleTime: 60 * 1000,
  });

  if (isLoading) {
    return (
      <div className="mt-6 rounded-2xl border border-[var(--color-line)] bg-white p-5 shadow-[var(--admin-card-shadow)]">
        <p className="text-sm text-[var(--color-muted)]">Checking store health...</p>
      </div>
    );
  }

  if (isError || !data) {
    return (
      <div className="mt-6 rounded-2xl border border-amber-200 bg-amber-50 p-5 text-sm text-amber-800">
        <p className="font-medium">Could not load store health summary.</p>
        <p className="mt-1 text-xs">{error instanceof Error ? error.message : "Unknown error"}</p>
      </div>
    );
  }

  const health = summarizeStoreHealth(data);

  const cards = [
    {
      key: "login",
      label: "Login failure rate",
      value: pct(data.loginFailureRate.percent),
      hint: `${data.loginFailureRate.numerator}/${data.loginFailureRate.denominator} failed`,
      Icon: LogIn,
    },
    {
      key: "dropoff",
      label: "Cart conversion drop-off",
      value: pct(data.cartConversionDropoff.percent),
      hint: `${data.cartConversionDropoff.numerator}/${data.cartConversionDropoff.denominator} drop-off`,
      Icon: ShoppingCart,
    },
    {
      key: "checkout",
      label: "Checkout failure rate",
      value: pct(data.checkoutFailureRate.percent),
      hint: `${data.checkoutFailureRate.numerator}/${data.checkoutFailureRate.denominator} failed`,
      Icon: AlertTriangle,
    },
    {
      key: "payment",
      label: "Payment mismatch rate",
      value: pct(data.paymentMismatchRate.percent),
      hint: `${data.paymentMismatchRate.numerator}/${data.paymentMismatchRate.denominator} mismatch`,
      Icon: Activity,
    },
    {
      key: "admin",
      label: "Admin action failure rate",
      value: pct(data.adminActionFailureRate.percent),
      hint: `${data.adminActionFailureRate.numerator}/${data.adminActionFailureRate.denominator} failed`,
      Icon: ShieldAlert,
    },
    {
      key: "confidence",
      label: "Release confidence",
      value: `${data.releaseConfidence.score}/100`,
      hint: `Window: ${data.windowHours}h`,
      Icon: Gauge,
    },
    {
      key: "webhook-latency",
      label: "Webhook latency (avg)",
      value:
        data.webhookProcessingLatency.available && typeof data.webhookProcessingLatency.averageMs === "number"
          ? `${data.webhookProcessingLatency.averageMs.toFixed(2)} ms`
          : "N/A",
      hint:
        data.webhookProcessingLatency.available
          ? "From Rust backend processing metrics"
          : data.webhookProcessingLatency.message ?? "Unavailable",
      Icon: Activity,
    },
    {
      key: "refund-failures",
      label: "Refund failures",
      value: String(data.backendSignals?.refundFailureCount?.value ?? 0),
      hint: "Refund create or reconciliation failures",
      Icon: Wallet,
    },
    {
      key: "booking-failures",
      label: "Shiprocket booking failures",
      value: String(data.backendSignals?.shiprocketBookingFailureCount?.value ?? 0),
      hint: "Auto-fulfillment booking failures",
      Icon: Truck,
    },
    {
      key: "stuck-pending",
      label: "Stuck pending orders",
      value: String(data.backendSignals?.stuckPendingOrders?.value ?? 0),
      hint: "Pending orders older than 15 minutes",
      Icon: PackageSearch,
    },
    {
      key: "cancel-backlog",
      label: "Cancel logistics backlog",
      value: String(data.backendSignals?.cancelPendingLogisticsBacklog?.value ?? 0),
      hint: "Orders waiting for logistics cancellation retry",
      Icon: RefreshCcw,
    },
    {
      key: "outbox-backlog",
      label: "Outbox backlog",
      value: String(data.backendSignals?.outboxBacklog?.value ?? 0),
      hint: "Pending notification/email events",
      Icon: Activity,
    },
  ] as const;

  return (
    <section className="mt-6 space-y-4">
      <div>
        <h2 className="text-base font-semibold text-[var(--color-ink)]">Store health</h2>
        <p className="mt-1 text-sm text-[var(--color-muted)]">How things are running behind the scenes.</p>
      </div>

      <div
        className={cn(
          "flex items-start gap-3 rounded-2xl border p-5",
          health.ok ? "border-emerald-200 bg-emerald-50" : "border-amber-200 bg-amber-50"
        )}
      >
        {health.ok ? (
          <CheckCircle2 className="mt-0.5 h-6 w-6 shrink-0 text-emerald-600" />
        ) : (
          <AlertTriangle className="mt-0.5 h-6 w-6 shrink-0 text-amber-700" />
        )}
        <div>
          <p className={cn("text-base font-semibold", health.ok ? "text-emerald-600" : "text-amber-900")}>
            {health.ok ? "Everything looks fine today." : "A few things could use a look."}
          </p>
          {!health.ok ? (
            <ul className="mt-2 list-disc space-y-1 pl-5 text-sm text-amber-900/90">
              {health.issues.map((issue) => (
                <li key={issue}>{issue}</li>
              ))}
            </ul>
          ) : null}
        </div>
      </div>

      <CollapsibleSection
        title="Technical details"
        description={`For your developer — raw system metrics from the last ${data.windowHours}h.`}
      >
        <div className="grid grid-cols-1 gap-3 pt-1 md:grid-cols-2 xl:grid-cols-3">
          {cards.map(({ key, label, value, hint, Icon }) => (
            <article
              key={key}
              className="rounded-xl border border-[var(--color-line)] bg-white p-4 shadow-[var(--admin-card-shadow)]"
            >
              <div className="flex items-center justify-between">
                <p className="text-xs font-semibold uppercase tracking-[0.14em] text-[var(--color-muted)]">
                  {label}
                </p>
                <Icon className="h-4 w-4 text-[var(--color-accent-gold)]" />
              </div>
              <p className="mt-3 text-2xl font-semibold tracking-tight text-[var(--color-ink)]">{value}</p>
              <p className="mt-1 text-xs text-[var(--color-muted)]">{hint}</p>
            </article>
          ))}
        </div>
      </CollapsibleSection>
    </section>
  );
}
