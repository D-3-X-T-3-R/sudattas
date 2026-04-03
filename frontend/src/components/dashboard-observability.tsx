"use client";

import { useQuery } from "@tanstack/react-query";
import { Activity, AlertTriangle, Gauge, LogIn, ShieldAlert, ShoppingCart } from "lucide-react";
import { fetchTelemetrySummary } from "@/lib/admin-queries";

function pct(v: number): string {
  return `${v.toFixed(2)}%`;
}

export function DashboardObservability() {
  const { data, isLoading, isError, error } = useQuery({
    queryKey: ["admin", "dashboard-observability"],
    queryFn: fetchTelemetrySummary,
    staleTime: 60 * 1000,
  });

  if (isLoading) {
    return (
      <div className="mt-6 rounded-xl border border-[var(--color-line)] bg-white p-5 shadow-[var(--admin-card-shadow)]">
        <p className="text-sm text-[var(--color-muted)]">Loading observability metrics...</p>
      </div>
    );
  }

  if (isError || !data) {
    return (
      <div className="mt-6 rounded-xl border border-amber-200 bg-amber-50 p-5 text-sm text-amber-800">
        <p className="font-medium">Could not load observability summary.</p>
        <p className="mt-1 text-xs">{error instanceof Error ? error.message : "Unknown error"}</p>
      </div>
    );
  }

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
  ] as const;

  return (
    <section className="mt-6">
      <div className="mb-3">
        <h2 className="text-sm font-semibold text-[var(--color-ink)]">Operational Observability</h2>
        <p className="mt-1 text-xs text-[var(--color-muted)]">
          Derived from structured frontend telemetry (last {data.windowHours}h).
        </p>
      </div>
      <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
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
    </section>
  );
}
