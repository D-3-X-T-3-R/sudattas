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
          Your order was not confirmed. Please retry checkout or choose a different payment method.
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
        </div>
      </section>
    </main>
  );
}
