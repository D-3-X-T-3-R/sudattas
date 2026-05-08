import Link from "next/link";
import { ShoppingBag } from "lucide-react";

export function BagEmptyState() {
  return (
    <div className="mx-auto mt-16 max-w-xl rounded-lg border border-dashed border-[var(--color-line-strong)] bg-[var(--color-surface-soft)] p-10 text-center">
      <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-md border border-[var(--color-line)] bg-white">
        <ShoppingBag size={26} strokeWidth={1.7} className="text-[var(--color-green)]" />
      </div>
      <p className="mt-5 font-display text-3xl text-[var(--color-ink)]">Your bag is empty</p>
      <p className="mt-2 text-sm text-[var(--color-muted)]">
        Add your favorite pieces and they will appear here.
      </p>
      <Link
        href="/"
        className="mt-6 inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-green)] bg-[var(--color-green)] px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-white"
      >
        Continue Shopping
      </Link>
    </div>
  );
}
