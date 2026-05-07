"use client";

import Link from "next/link";

export default function GlobalError({
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <main className="min-h-screen bg-[linear-gradient(135deg,#EFE9DE_0%,#F7F3EB_45%,#EEE6D8_100%)] px-4 py-10 text-[var(--foreground)] sm:px-6">
      <section className="mx-auto w-full max-w-2xl rounded-[28px] border border-white/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.85),rgba(255,255,255,0.72))] p-6 shadow-[0_30px_90px_rgba(15,61,46,0.10)] backdrop-blur-xl sm:p-8">
        <p className="inline-flex items-center rounded-full border border-[#B95A40]/30 bg-[#FFF3EE] px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.22em] text-[#9F4A35] sm:px-4 sm:text-[11px]">
          Something Went Wrong
        </p>
        <h1 className="mt-5 font-display text-3xl leading-tight text-[#0F3D2E] sm:text-4xl">
          We couldn&apos;t load this page.
        </h1>
        <p className="mt-3 text-sm leading-relaxed text-[#615A50]">
          Please try again. If this keeps happening, contact support and we&apos;ll help you complete your order journey.
        </p>
        <div className="mt-8 flex flex-col gap-3 sm:flex-row">
          <button
            type="button"
            onClick={() => reset()}
            className="inline-flex h-12 items-center justify-center rounded-full bg-[#0F3D2E] px-6 text-xs font-semibold uppercase tracking-[0.18em] text-[#F6F3EA] transition hover:bg-[#0C3126]"
          >
            Try Again
          </button>
          <Link
            href="/"
            className="inline-flex h-12 items-center justify-center rounded-full border border-[#0F3D2E]/20 bg-white px-6 text-xs font-semibold uppercase tracking-[0.18em] text-[#0F3D2E] transition hover:border-[#0F3D2E]/40"
          >
            Continue Shopping
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
