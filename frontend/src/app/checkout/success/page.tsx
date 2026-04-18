import Link from "next/link";

export default async function CheckoutSuccessPage({
  searchParams,
}: {
  searchParams: Promise<{ orderId?: string }>;
}) {
  const params = await searchParams;
  const orderId = params.orderId?.trim() ?? "";

  return (
    <main className="min-h-screen w-full bg-[linear-gradient(135deg,#EFE9DE_0%,#F7F3EB_45%,#EEE6D8_100%)] px-4 py-8 text-[var(--foreground)] sm:px-6">
      <section className="mx-auto w-full max-w-2xl rounded-[28px] border border-white/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.82),rgba(255,255,255,0.72))] p-6 shadow-[0_30px_90px_rgba(15,61,46,0.10)] backdrop-blur-xl sm:p-8">
        <p className="inline-flex items-center rounded-full border border-[#C9A646]/30 bg-[#FFF9EF] px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.22em] text-[#A37D34] sm:px-4 sm:text-[11px]">
          Order Confirmed
        </p>
        <h1 className="mt-5 font-display text-3xl leading-tight text-[#0F3D2E] sm:text-4xl">
          Order placed successfully
        </h1>
        <p className="mt-3 text-sm leading-relaxed text-[#615A50]">
          Thank you for shopping with us. Your order has been created, payment verification is complete, and fulfillment is now moving into our courier booking flow.
        </p>
        {orderId ? (
          <p className="mt-4 rounded-xl bg-[#FFFDF8] px-3 py-2 text-xs font-semibold uppercase tracking-[0.14em] text-[#8B816D]">
            Order ID: {orderId}
          </p>
        ) : null}
        <div className="mt-6 grid gap-3 rounded-[22px] border border-[#0F3D2E]/8 bg-white/65 p-4 text-sm leading-6 text-[#615A50] sm:grid-cols-3">
          <div>
            <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-[#A37D34]">Next step</p>
            <p className="mt-2">We book the quoted courier after payment and schedule pickup within the next 48 hours.</p>
          </div>
          <div>
            <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-[#A37D34]">Cancellation window</p>
            <p className="mt-2">Cancellation stays available until pickup is completed. After that, courier return handling applies.</p>
          </div>
          <div>
            <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-[#A37D34]">Need help?</p>
            <p className="mt-2">Use your order detail page for support, refund status, and the latest shipment updates.</p>
          </div>
        </div>
        <div className="mt-8 flex flex-col gap-3 sm:flex-row">
          <Link
            href="/profile"
            className="inline-flex h-12 items-center justify-center rounded-full bg-[#0F3D2E] px-6 text-xs font-semibold uppercase tracking-[0.18em] text-[#F6F3EA] transition hover:bg-[#0C3126]"
          >
            View Orders
          </Link>
          <Link
            href="/"
            className="inline-flex h-12 items-center justify-center rounded-full border border-[#0F3D2E]/20 bg-white px-6 text-xs font-semibold uppercase tracking-[0.18em] text-[#0F3D2E] transition hover:border-[#0F3D2E]/40"
          >
            Continue Shopping
          </Link>
          <Link
            href="/contact-support"
            className="inline-flex h-12 items-center justify-center rounded-full border border-[#C9A646]/35 bg-[#FFF9EF] px-6 text-xs font-semibold uppercase tracking-[0.18em] text-[#A37D34] transition hover:border-[#C9A646]/60"
          >
            Contact Support
          </Link>
        </div>
      </section>
    </main>
  );
}
