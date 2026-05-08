import Link from "next/link";
import { PageShell, EmptyState } from "@/components/ui/page-shell";

export default function NotFound() {
  return (
    <PageShell containerClassName="py-10">
      <EmptyState
        title="Page not found"
        description="The link may be outdated or the page may have moved. Continue browsing collections or contact support for help."
        action={
          <div className="flex flex-col gap-2 sm:flex-row sm:justify-center">
            <Link
              href="/"
              className="inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-green)] bg-[var(--color-green)] px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-white"
            >
              Continue Shopping
            </Link>
            <Link
              href="/contact-support"
              className="inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-line)] bg-white px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-ink)]"
            >
              Contact Support
            </Link>
          </div>
        }
      />
    </PageShell>
  );
}
