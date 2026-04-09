import { INR } from "@/lib/constants";

type BagMobileCheckoutBarProps = {
  selectedSubtotal: number;
  shippingAmount: number;
  selectedCount: number;
  onCheckout: () => void;
};

export function BagMobileCheckoutBar({
  selectedSubtotal,
  shippingAmount,
  selectedCount,
  onCheckout,
}: BagMobileCheckoutBarProps) {
  const total = selectedSubtotal + shippingAmount;
  return (
    <div className="fixed bottom-0 left-0 right-0 z-40 border-t border-[var(--color-line)] bg-white/95 px-4 py-3 backdrop-blur-sm md:hidden">
      <div className="flex items-center justify-between gap-4">
        <div>
          <p className="text-[10px] uppercase tracking-[0.18em] text-[var(--color-muted)]">Total</p>
          <p className="font-sans text-base font-bold text-[var(--color-ink)]">{INR.format(total)}</p>
        </div>
        <button
          onClick={onCheckout}
          disabled={selectedCount === 0}
          className="flex-1 rounded-full bg-[var(--color-accent-gold)] py-3 text-xs font-semibold uppercase tracking-[0.2em] text-white transition-opacity hover:opacity-90 disabled:opacity-40"
        >
          Checkout ({selectedCount})
        </button>
      </div>
    </div>
  );
}
