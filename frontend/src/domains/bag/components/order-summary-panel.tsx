import { INR } from "@/lib/constants";
import { formatInrFromPaise } from "@/lib/money";
import type { CartLine } from "@/lib/schemas";
import type { ActiveCouponRow } from "@/lib/active-coupons";
import { SummaryCard } from "@/components/ui/page-shell";
import { GoldDivider } from "@/domains/bag/components/bag-shared";
import { CouponCodeField } from "@/domains/bag/components/coupon-code-field";
import { AvailableOffersList } from "@/domains/bag/components/available-offers-list";

type OrderSummaryPanelProps = {
  cartLines: CartLine[];
  cartSubtotal: number;
  cartCount: number;
  shippingAmount: number;
  shippingLoading: boolean;
  shippingNote?: string | null;
  discountAmount?: number;
  couponInput: string;
  onCouponInputChange: (value: string) => void;
  appliedCouponCode?: string | null;
  couponMessage?: string | null;
  couponApplying?: boolean;
  onApplyCoupon: () => void;
  onRemoveCoupon: () => void;
  activeOffers?: ActiveCouponRow[];
  offersLoading?: boolean;
  onSelectOffer?: (code: string) => void;
  checkoutLoading?: boolean;
  showMobileCheckoutCta?: boolean;
  onCheckout: () => void;
};

export function OrderSummaryPanel({
  cartLines,
  cartSubtotal,
  cartCount,
  shippingAmount,
  shippingLoading,
  shippingNote,
  discountAmount = 0,
  couponInput,
  onCouponInputChange,
  appliedCouponCode,
  couponMessage,
  couponApplying = false,
  onApplyCoupon,
  onRemoveCoupon,
  activeOffers = [],
  offersLoading = false,
  onSelectOffer,
  checkoutLoading = false,
  showMobileCheckoutCta = false,
  onCheckout,
}: OrderSummaryPanelProps) {
  const totalAmount = Math.max(0, cartSubtotal - discountAmount + shippingAmount);

  return (
    <SummaryCard
      title="Order Summary"
      subtitle={`${cartCount} ${cartCount === 1 ? "item" : "items"} selected`}
      className="lg:sticky lg:top-24"
    >
      <div className="space-y-3">
        {cartLines.map(({ id, product, qty, sizeName }) => (
          <div key={id} className="flex items-start justify-between gap-3 text-sm">
            <p className="min-w-0 text-[var(--color-muted)]">
              <span className="text-[var(--color-ink)]">{product.name}</span>
              {sizeName && sizeName.toLowerCase() !== "free size" ? ` / ${sizeName}` : ""} &times; {qty}
            </p>
            <p className="shrink-0 font-medium text-[var(--color-ink)]">
              {formatInrFromPaise(qty * (product.pricePaise ?? Math.round(product.price * 100)))}
            </p>
          </div>
        ))}
      </div>

      <CouponCodeField
        value={couponInput}
        onChange={onCouponInputChange}
        appliedCouponCode={appliedCouponCode}
        message={couponMessage}
        applying={couponApplying}
        onApply={onApplyCoupon}
        onRemove={onRemoveCoupon}
      />

      {onSelectOffer ? (
        <AvailableOffersList
          offers={activeOffers}
          isLoading={offersLoading}
          appliedCouponCode={appliedCouponCode}
          onSelect={onSelectOffer}
        />
      ) : null}

      <div className="mt-5 space-y-2.5 border-t border-[var(--color-line)] pt-5 text-sm">
        <div className="flex items-center justify-between">
          <span className="text-[var(--color-muted)]">Subtotal</span>
          <span className="font-medium text-[var(--color-ink)]">{INR.format(cartSubtotal)}</span>
        </div>
        {discountAmount > 0 ? (
          <div className="flex items-center justify-between">
            <span className="text-[var(--color-muted)]">Coupon discount</span>
            <span className="font-medium text-[var(--color-green)]">&minus;{INR.format(discountAmount)}</span>
          </div>
        ) : null}
        <div className="flex items-center justify-between">
          <span className="text-[var(--color-muted)]">Shipping</span>
          <span className="font-medium text-[var(--color-ink)]">
            {shippingLoading ? "Calculating..." : INR.format(shippingAmount)}
          </span>
        </div>
        {shippingNote ? <p className="text-xs leading-relaxed text-[var(--color-muted)]">{shippingNote}</p> : null}
      </div>

      <div className="mt-4">
        <GoldDivider />
        <div className="mt-4 flex items-center justify-between">
          <span className="text-sm font-semibold uppercase tracking-[0.12em] text-[var(--color-ink)]">Total</span>
          <span className="font-sans text-xl font-semibold text-[var(--color-green)] md:text-2xl">
            {INR.format(totalAmount)}
          </span>
        </div>
      </div>

      <button
        type="button"
        onClick={onCheckout}
        disabled={checkoutLoading || cartCount === 0}
        aria-busy={checkoutLoading}
        className={`mt-6 h-12 w-full items-center justify-center rounded-md border border-[var(--color-green)] bg-[var(--color-green)] px-4 text-[11px] font-semibold uppercase tracking-[0.16em] text-white transition hover:bg-[var(--color-green-2)] disabled:cursor-not-allowed disabled:opacity-50 lg:inline-flex ${
          showMobileCheckoutCta ? "inline-flex" : "hidden"
        }`}
      >
        {checkoutLoading ? "Processing..." : "Proceed To Checkout"}
      </button>

      <p className="mt-3 text-center text-[10px] font-medium uppercase tracking-[0.14em] text-[var(--color-muted)] lg:mt-3">
        Secure payments and verified order updates
      </p>
    </SummaryCard>
  );
}
