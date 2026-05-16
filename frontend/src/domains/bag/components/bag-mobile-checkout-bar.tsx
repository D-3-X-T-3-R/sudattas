import { INR } from "@/lib/constants";

type BagMobileCheckoutBarProps = {
  selectedSubtotal: number;
  shippingAmount: number;
  selectedCount: number;
  checkoutLoading?: boolean;
  visible?: boolean;
  onCheckout: () => void;
};

export function BagMobileCheckoutBar({
  selectedSubtotal,
  shippingAmount,
  selectedCount,
  checkoutLoading = false,
  visible = true,
  onCheckout,
}: BagMobileCheckoutBarProps) {
  const total = selectedSubtotal + shippingAmount;
  return (
    <div
      aria-hidden={!visible}
      className={`fixed bottom-[calc(4.75rem+env(safe-area-inset-bottom))] left-0 right-0 z-30 border-t border-[var(--color-line)] bg-[var(--color-surface)]/98 px-4 py-3 shadow-[0_-8px_22px_rgba(45,42,38,0.07)] backdrop-blur-sm transition duration-200 ease-out lg:hidden ${
        visible ? "translate-y-0 opacity-100" : "pointer-events-none translate-y-3 opacity-0"
      }`}
    >
      <div className="flex items-center justify-between gap-4">
        <div>
          <p className="text-[10px] uppercase tracking-[0.18em] text-[var(--color-muted)]">Total</p>
          <p className="font-sans text-base font-bold text-[var(--color-green)]">{INR.format(total)}</p>
        </div>
        <button
          type="button"
          onClick={onCheckout}
          disabled={selectedCount === 0 || checkoutLoading || !visible}
          aria-busy={checkoutLoading}
          tabIndex={visible ? 0 : -1}
          className="flex-1 rounded-md border border-[var(--color-green)] bg-[var(--color-green)] py-3 text-xs font-semibold uppercase tracking-[0.16em] text-white transition-opacity hover:bg-[var(--color-green-2)] disabled:opacity-40"
        >
          {checkoutLoading ? "Processing..." : `Checkout (${selectedCount})`}
        </button>
      </div>
    </div>
  );
}
