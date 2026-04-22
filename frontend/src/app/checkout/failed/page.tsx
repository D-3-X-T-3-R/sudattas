import Link from "next/link";

export default async function CheckoutFailedPage({
  searchParams,
}: {
  searchParams: Promise<{ orderId?: string; reason?: string }>;
}) {
  const params = await searchParams;
  const orderId = params.orderId?.trim() ?? "";
  const reason = params.reason?.trim() ?? "";

  return (
    <main className="min-h-screen w-full bg-[linear-gradient(135deg,#EFE9DE_0%,#F7F3EB_45%,#EEE6D8_100%)] px-4 py-8 text-[var(--foreground)] sm:px-6">
      <section className="mx-auto w-full max-w-2xl rounded-[28px] border border-white/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.82),rgba(255,255,255,0.72))] p-6 shadow-[0_30px_90px_rgba(15,61,46,0.10)] backdrop-blur-xl sm:p-8">
        <p className="inline-flex items-center rounded-full border border-[#B95A40]/30 bg-[#FFF3EE] px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.22em] text-[#9F4A35] sm:px-4 sm:text-[11px]">
          Payment Failed
        </p>
        <h1 className="mt-5 font-display text-3xl leading-tight text-[#0F3D2E] sm:text-4xl">
          We could not complete your payment
        </h1>
        <p className="mt-3 text-sm leading-relaxed text-[#615A50]">
          Your order was not confirmed. Your selected bag items remain available for retry, and no shipment is created until payment verification succeeds.
        </p>
        {reason ? (
          <p className="mt-4 rounded-xl bg-[#FFFDF8] px-3 py-2 text-xs text-[#8B816D]">
            Reason: {reason}
          </p>
        ) : null}
        {orderId ? (
          <p className="mt-3 rounded-xl bg-[#FFFDF8] px-3 py-2 text-xs font-semibold uppercase tracking-[0.14em] text-[#8B816D]">
            Order ID: {orderId}
          </p>
        ) : null}
        <div className="mt-6 grid gap-3 rounded-[22px] border border-[#B95A40]/10 bg-white/70 p-4 text-sm leading-6 text-[#615A50] sm:grid-cols-3">
          <div>
            <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-[#9F4A35]">What happened</p>
            <p className="mt-2">Payment verification did not complete, so the order did not proceed into shipment booking.</p>
          </div>
          <div>
            <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-[#9F4A35]">Retry safely</p>
            <p className="mt-2">Retry from your bag with the same selected items. Duplicate payment verification is blocked by backend idempotency and signature checks.</p>
          </div>
          <div>
            <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-[#9F4A35]">Need support</p>
            <p className="mt-2">If you saw a bank debit but this page persisted, contact support with the order reference shown here.</p>
          </div>
        </div>
        <div className="mt-8 flex flex-col gap-3 sm:flex-row">
          <Link
            href="/bag"
            className="inline-flex h-12 items-center justify-center rounded-full bg-[#0F3D2E] px-6 text-xs font-semibold uppercase tracking-[0.18em] text-[#F6F3EA] transition hover:bg-[#0C3126]"
          >
            Retry Checkout
          </Link>
          <Link
            href="/bag"
            className="inline-flex h-12 items-center justify-center rounded-full border border-[#0F3D2E]/20 bg-white px-6 text-xs font-semibold uppercase tracking-[0.18em] text-[#0F3D2E] transition hover:border-[#0F3D2E]/40"
          >
            Back To Bag
          </Link>
          <Link
            href="/contact-support"
            className="inline-flex h-12 items-center justify-center rounded-full border border-[#B95A40]/25 bg-[#FFF3EE] px-6 text-xs font-semibold uppercase tracking-[0.18em] text-[#9F4A35] transition hover:border-[#B95A40]/45"
          >
            Contact Support
          </Link>
        </div>
      </section>
    </main>
  );
}
