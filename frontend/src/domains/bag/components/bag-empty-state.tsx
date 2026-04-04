import Link from "next/link";
import { ShoppingBag } from "lucide-react";

export function BagEmptyState() {
  return (
    <div className="mt-24 flex flex-col items-center gap-6 text-center">
      <div className="flex h-24 w-24 items-center justify-center rounded-full border border-[var(--color-line)] bg-white">
        <ShoppingBag size={36} strokeWidth={1.25} className="text-[var(--color-accent-gold)]" />
      </div>
      <div>
        <p className="font-display text-2xl font-medium text-[var(--color-ink)]">Your bag is empty</p>
        <p className="mt-2 text-sm text-[var(--color-muted)]">
          Looks like you haven&apos;t added anything yet.
        </p>
      </div>
      <Link
        href="/"
        className="mt-2 rounded-full bg-[var(--color-accent-gold)] px-8 py-3 text-xs font-semibold uppercase tracking-[0.2em] text-white transition-opacity hover:opacity-90"
      >
        Continue Shopping
      </Link>
    </div>
  );
}
