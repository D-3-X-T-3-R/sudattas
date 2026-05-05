import Link from "next/link";

export default function NotFound() {
  return (
    <main className="min-h-screen bg-[linear-gradient(135deg,#EFE9DE_0%,#F7F3EB_45%,#EEE6D8_100%)] px-4 py-10 text-[var(--foreground)] sm:px-6">
      <section className="mx-auto w-full max-w-2xl rounded-[28px] border border-white/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.85),rgba(255,255,255,0.72))] p-6 shadow-[0_30px_90px_rgba(15,61,46,0.10)] backdrop-blur-xl sm:p-8">
        <p className="inline-flex items-center rounded-full border border-[#C9A646]/30 bg-[#FFF9EF] px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.22em] text-[#A37D34] sm:px-4 sm:text-[11px]">
          Page Not Found
        </p>
        <h1 className="mt-5 font-display text-3xl leading-tight text-[#0F3D2E] sm:text-4xl">
          This page is unavailable right now.
        </h1>
        <p className="mt-3 text-sm leading-relaxed text-[#615A50]">
          The link may be outdated or the page may have moved. You can continue browsing collections or reach support for quick help.
        </p>
        <div className="mt-8 flex flex-col gap-3 sm:flex-row">
          <Link
            href="/"
            className="inline-flex h-12 items-center justify-center rounded-full bg-[#0F3D2E] px-6 text-xs font-semibold uppercase tracking-[0.18em] text-[#F6F3EA] transition hover:bg-[#0C3126]"
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
