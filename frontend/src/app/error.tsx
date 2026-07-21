"use client";

import Link from "next/link";
import { ErrorState, PageShell } from "@/components/ui/page-shell";

export default function GlobalError({
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <PageShell containerClassName="py-10">
      <ErrorState
        title="We could not load this page"
        message="Please try again. If the issue continues, contact support and we will help you complete your order journey."
        action={
          <div className="flex flex-col gap-2 sm:flex-row">
            <button
              type="button"
              onClick={() => reset()}
              className="inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-green)] bg-[var(--color-green)] px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-white"
            >
              Try Again
            </button>
            <Link
              href="/"
              className="inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-line)] bg-white px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-ink)]"
            >
              Continue Shopping
            </Link>
            <Link
              href="/contact-support"
              className="inline-flex h-11 items-center justify-center rounded-md border border-[#D8B2A7] bg-[#FFF5EF] px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-[#7A5348]"
            >
              Contact Support
            </Link>
          </div>
        }
      />
    </PageShell>
  );
}
