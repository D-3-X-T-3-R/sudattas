import Link from "next/link";
import { ShoppingBag } from "lucide-react";

export function BagEmptyState() {
  return (
    <div className="mx-auto mt-12 max-w-xl rounded-[var(--radius-lg)] border border-dashed border-[var(--color-line-strong)] bg-[var(--color-surface-soft)] p-10 text-center md:mt-16 md:p-14">
      <ShoppingBag size={32} strokeWidth={1.5} className="mx-auto text-[var(--color-gold)]" />
      <p className="mt-5 font-display text-[2rem] font-medium leading-[1.12] tracking-[-0.01em] text-[var(--color-ink)] sm:text-[2.4rem]">
        Your bag is empty
      </p>
      <p className="mt-3 text-sm leading-relaxed text-[var(--color-muted)]">
        Add your favorite pieces and they will appear here.
      </p>
      <Link
        href="/"
        className="mt-6 inline-flex h-11 items-center justify-center rounded-[var(--radius-md)] border border-[var(--color-green)] bg-[var(--color-green)] px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-white transition-colors hover:bg-[var(--color-green-2)]"
      >
        Continue Shopping
      </Link>
    </div>
  );
}
