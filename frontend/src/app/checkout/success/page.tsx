import Link from "next/link";
import { Check, Clock3, ShieldAlert } from "lucide-react";
import { StatusTimeline, type TimelineStep } from "@/components/status-timeline";
import { TrustStrip } from "@/components/trust-strip";

type SuccessVariant = "paid" | "pending" | "needs_review" | "cod";

function resolveSuccessVariant(raw?: string): SuccessVariant {
  const value = (raw ?? "").trim().toLowerCase();
  if (value === "cod") return "cod";
  if (value === "needs_review") return "needs_review";
  if (value === "paid" || value === "captured" || value === "confirmed") return "paid";
  return "pending";
}

function timelineForVariant(variant: SuccessVariant): TimelineStep[] {
  if (variant === "paid" || variant === "cod") {
    return [
      { label: "Order confirmed", state: "done" },
      { label: "Processing", state: "current" },
      { label: "Shipped", state: "pending" },
      { label: "Out for delivery", state: "pending" },
    ];
  }
  if (variant === "needs_review") {
    return [
      { label: "Order created", state: "done" },
      { label: "Payment review", detail: "Manual verification in progress", state: "current" },
      { label: "Order confirmation", state: "pending" },
      { label: "Shipment processing", state: "pending" },
    ];
  }
  return [
    { label: "Order created", state: "done" },
    { label: "Payment pending", detail: "Awaiting gateway confirmation", state: "current" },
    { label: "Order confirmation", state: "pending" },
    { label: "Shipment processing", state: "pending" },
  ];
}

export default async function CheckoutSuccessPage({
  searchParams,
}: {
  searchParams: Promise<{ orderId?: string; payment?: string; invoice?: string }>;
}) {
  const cancelWindowHours = Number.parseInt((process.env.CANCEL_WINDOW_HOURS ?? "12").trim(), 10);
  const pickupDelayHours = Number.parseInt((process.env.PICKUP_DELAY_HOURS ?? "48").trim(), 10);
  const normalizedCancelWindowHours = Number.isFinite(cancelWindowHours) && cancelWindowHours > 0 ? cancelWindowHours : 12;
  const normalizedPickupDelayHours = Number.isFinite(pickupDelayHours) && pickupDelayHours > 0 ? pickupDelayHours : 48;

  const params = await searchParams;
  const orderId = params.orderId?.trim() ?? "";
  const variant = resolveSuccessVariant(params.payment);
  const invoiceAvailable = (params.invoice ?? "").trim().toLowerCase() === "available";

  const badgeByVariant: Record<SuccessVariant, string> = {
    paid: "Order Confirmed",
    pending: "Payment Pending",
    needs_review: "Payment Under Review",
    cod: "Order Received",
  };

  const headingByVariant: Record<SuccessVariant, string> = {
    paid: "Thank you. Your order has been placed.",
    pending: "We're confirming your payment",
    needs_review: "Your payment is under manual review",
    cod: "Your Cash on Delivery order is confirmed.",
  };

  const summaryByVariant: Record<SuccessVariant, string> = {
    paid: "Your payment is verified and your order is now in processing.",
    pending: "We're confirming your payment. Please don't place another order yet.",
    needs_review:
      "We received your payment update, but it needs manual verification. We'll contact you if action is needed.",
    cod: "Your order is confirmed with Cash on Delivery and will move to shipment processing.",
  };

  const nextStepByVariant: Record<SuccessVariant, string> = {
    paid: `Shipment is booked after the cancellation window and scheduled with a ${normalizedPickupDelayHours}-hour pickup buffer.`,
    pending: "We will update this order as soon as payment confirmation is complete.",
    needs_review: "Manual verification is in progress. We will notify you if any action is needed.",
    cod: `Shipment is booked after the cancellation window and scheduled with a ${normalizedPickupDelayHours}-hour pickup buffer.`,
  };

  const iconByVariant = {
    paid: Check,
    cod: Check,
    pending: Clock3,
    needs_review: ShieldAlert,
  } as const;

  const Icon = iconByVariant[variant];
  const timeline = timelineForVariant(variant);

  return (
    <main className="min-h-screen bg-[var(--background)] px-4 py-8 text-[var(--foreground)] sm:px-6">
      <section className="mx-auto w-full max-w-[1280px] space-y-6">
        <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_380px]">
          <article className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-5 shadow-[var(--shadow-subtle)] md:p-6">
            <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-full border border-[var(--color-green)] bg-[var(--color-green)] text-white">
              <Icon className="h-6 w-6" />
            </div>
            <p className="mt-4 text-center text-[10px] font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">
              {badgeByVariant[variant]}
            </p>
            <h1 className="mt-2 text-center font-display text-[2rem] leading-[1.2] text-[var(--color-ink)] md:text-[2.4rem]">
              {headingByVariant[variant]}
            </h1>
            <p className="mt-3 text-center text-sm leading-relaxed text-[var(--color-muted)]">
              {summaryByVariant[variant]}
            </p>

            {orderId ? (
              <div className="mx-auto mt-5 max-w-sm rounded-md border border-[var(--color-line)] bg-[var(--color-surface-soft)] px-4 py-3 text-center">
                <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-[var(--color-muted)]">Order ID</p>
                <p className="mt-1 font-mono text-sm text-[var(--color-ink)]">{orderId}</p>
              </div>
            ) : null}

            <div className="mt-6 rounded-md border border-[var(--color-line)] bg-[var(--color-surface-soft)] p-4 text-sm text-[var(--color-muted)]">
              {nextStepByVariant[variant]}
            </div>

            <div className="mt-6 grid gap-2 sm:grid-cols-2">
              <Link
                href="/profile"
                className="inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-green)] bg-[var(--color-green)] px-5 text-[11px] font-semibold uppercase tracking-[0.16em] text-white"
              >
                View Orders
              </Link>
              <Link
                href="/"
                className="inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-line)] bg-white px-5 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-ink)]"
              >
                Continue Shopping
              </Link>
              <Link
                href="/contact-support"
                className="inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-gold)] bg-[var(--color-surface-soft)] px-5 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-green)]"
              >
                Contact Support
              </Link>
              {orderId && invoiceAvailable ? (
                <Link
                  href={`/api/account/orders/${encodeURIComponent(orderId)}/invoice`}
                  className="inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-line)] bg-white px-5 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-ink)]"
                >
                  Download Invoice
                </Link>
              ) : (
                <p className="inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-line)] bg-[var(--color-surface-soft)] px-5 text-[10px] font-semibold uppercase tracking-[0.14em] text-[var(--color-muted)]">
                  Invoice will be available after confirmation.
                </p>
              )}
            </div>
          </article>

          <aside className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-5 shadow-[var(--shadow-subtle)] md:p-6">
            <p className="text-[11px] font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">Order Status</p>
            <div className="mt-4">
              <StatusTimeline steps={timeline} />
            </div>
            <div className="mt-5 border-t border-[var(--color-line)] pt-4 text-sm text-[var(--color-muted)]">
              <p>
                Cancellation is available for {normalizedCancelWindowHours} hours from order creation. After this window,
                you can refuse delivery and request support.
              </p>
            </div>
          </aside>
        </div>

        <TrustStrip />
      </section>
    </main>
  );
}
