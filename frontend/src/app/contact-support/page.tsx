import Link from "next/link";

export default function ContactSupportPage() {
  return (
    <main className="min-h-screen bg-[linear-gradient(180deg,#F8F3EC_0%,#F4EBDD_50%,#F7F3EB_100%)] px-4 py-10 text-[var(--foreground)] sm:px-6">
      <section className="mx-auto max-w-4xl rounded-[32px] border border-white/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.9),rgba(255,255,255,0.76))] p-6 shadow-[0_30px_90px_rgba(15,61,46,0.08)] backdrop-blur-xl sm:p-10">
        <p className="inline-flex rounded-full border border-[#C9A646]/30 bg-[#FFF9EF] px-4 py-2 text-[10px] font-semibold uppercase tracking-[0.24em] text-[#A37D34]">
          Contact Support
        </p>
        <h1 className="mt-5 font-display text-3xl leading-tight text-[#0F3D2E] sm:text-5xl">
          Order help with a real support path.
        </h1>
        <p className="mt-4 max-w-3xl text-sm leading-7 text-[#615A50] sm:text-base">
          For delivery updates, cancellation questions, refund status, or styling assistance, start with the order detail page in your profile. That gives our team the exact order context immediately.
        </p>
        <div className="mt-10 grid gap-4 sm:grid-cols-2">
          <div className="rounded-[24px] border border-[#0F3D2E]/8 bg-white/65 p-5">
            <h2 className="text-sm font-semibold uppercase tracking-[0.18em] text-[#0F3D2E]">Fastest order support</h2>
            <p className="mt-3 text-sm leading-7 text-[#615A50]">
              Open Profile &gt; Orders for cancellation eligibility, refund status, courier updates, and support request options tied to your order.
            </p>
          </div>
          <div className="rounded-[24px] border border-[#0F3D2E]/8 bg-white/65 p-5">
            <h2 className="text-sm font-semibold uppercase tracking-[0.18em] text-[#0F3D2E]">Styling and bulk enquiries</h2>
            <p className="mt-3 text-sm leading-7 text-[#615A50]">
              Email <a className="font-semibold text-[#0F3D2E]" href="mailto:support@sudattas.com">support@sudattas.com</a> with your event date, order reference if available, and the assistance you need.
            </p>
          </div>
        </div>
        <div className="mt-10 flex flex-col gap-3 sm:flex-row">
          <Link
            href="/profile"
            className="inline-flex h-12 items-center justify-center rounded-full bg-[#0F3D2E] px-6 text-xs font-semibold uppercase tracking-[0.18em] text-[#F6F3EA] transition hover:bg-[#0C3126]"
          >
            Go To Orders
          </Link>
          <Link
            href="/returns-exchanges"
            className="inline-flex h-12 items-center justify-center rounded-full border border-[#0F3D2E]/20 bg-white px-6 text-xs font-semibold uppercase tracking-[0.18em] text-[#0F3D2E] transition hover:border-[#0F3D2E]/40"
          >
            Returns & Exchanges
          </Link>
        </div>
      </section>
    </main>
  );
}
