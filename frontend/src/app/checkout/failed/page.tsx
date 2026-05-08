import Link from "next/link";
import { XCircle } from "lucide-react";
import { StatusTimeline } from "@/components/status-timeline";
import { TrustStrip } from "@/components/trust-strip";

export default async function CheckoutFailedPage({
  searchParams,
}: {
  searchParams: Promise<{ orderId?: string; reason?: string }>;
}) {
  const params = await searchParams;
  const orderId = params.orderId?.trim() ?? "";
  const reason = params.reason?.trim() ?? "";

  return (
    <main className="min-h-screen bg-[var(--background)] px-4 py-8 text-[var(--foreground)] sm:px-6">
      <section className="mx-auto w-full max-w-[1280px] space-y-6">
        <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_380px]">
          <article className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-5 shadow-[var(--shadow-subtle)] md:p-6">
            <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-full border border-[#B95A40] bg-[#B95A40] text-white">
              <XCircle className="h-6 w-6" />
            </div>
            <p className="mt-4 text-center text-[10px] font-semibold uppercase tracking-[0.2em] text-[#9F4A35]">
              Payment Failed
            </p>
            <h1 className="mt-2 text-center font-display text-[2rem] leading-[1.2] text-[var(--color-ink)] md:text-[2.4rem]">
              We could not complete your payment.
            </h1>
            <p className="mt-3 text-center text-sm text-[var(--color-muted)]">
              Payment verification did not complete. You can retry safely from your bag.
            </p>

            {reason ? (
              <p className="mx-auto mt-4 max-w-md rounded-md border border-[#D8B2A7] bg-[#FFF5EF] px-4 py-3 text-sm text-[#7A5348]">
                Reason: {reason}
              </p>
            ) : null}
            {orderId ? (
              <p className="mx-auto mt-3 max-w-md rounded-md border border-[var(--color-line)] bg-[var(--color-surface-soft)] px-4 py-3 text-center text-xs font-semibold uppercase tracking-[0.14em] text-[var(--color-muted)]">
                Order ID: {orderId}
              </p>
            ) : null}

            <div className="mt-6 grid gap-2 sm:grid-cols-2">
              <Link
                href="/bag"
                className="inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-green)] bg-[var(--color-green)] px-5 text-[11px] font-semibold uppercase tracking-[0.16em] text-white"
              >
                Retry Checkout
              </Link>
              <Link
                href="/bag"
                className="inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-line)] bg-white px-5 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-ink)]"
              >
                Back To Bag
              </Link>
              <Link
                href="/contact-support"
                className="inline-flex h-11 items-center justify-center rounded-md border border-[#D8B2A7] bg-[#FFF5EF] px-5 text-[11px] font-semibold uppercase tracking-[0.16em] text-[#7A5348]"
              >
                Contact Support
              </Link>
            </div>
          </article>

          <aside className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-5 shadow-[var(--shadow-subtle)] md:p-6">
            <p className="text-[11px] font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">What Happens Next</p>
            <div className="mt-4">
              <StatusTimeline
                steps={[
                  { label: "Order created", state: "done" },
                  { label: "Payment failed", state: "current" },
                  { label: "Retry from bag", state: "pending" },
                  { label: "Order confirmation", state: "pending" },
                ]}
              />
            </div>
          </aside>
        </div>

        <TrustStrip />
      </section>
    </main>
  );
}
