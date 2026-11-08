import { useCallback, useEffect, useState } from "react";
import { ApiEnvelopeError } from "@/lib/api-envelope";
import { fetchActiveCoupons, type ActiveCouponRow } from "@/lib/active-coupons";
import { validateCouponCode } from "@/lib/coupon-validation";

/**
 * Owns coupon-code entry/apply/remove state and the "Available offers" list for the bag page.
 *
 * Applying runs an immediate, dedicated check against the coupon's own properties (exists,
 * active, date window, usage limit, min order value) the moment the button is clicked — a coupon
 * is only ever marked "applied" once that check comes back valid. Does NOT own the shipping
 * estimate (discountAmount is refined there once applied, and cart-scope/per-customer
 * eligibility is checked for real at that point too) — the caller calls
 * `setDiscountAmount`/`setCouponMessage` from its own shipping-estimate effect for that.
 */
export function useCouponFlow(status: string, orderAmountPaise: number) {
  const [couponInput, setCouponInput] = useState("");
  const [appliedCouponCode, setAppliedCouponCode] = useState<string | null>(null);
  const [couponMessage, setCouponMessage] = useState<string | null>(null);
  const [couponApplying, setCouponApplying] = useState(false);
  const [discountAmount, setDiscountAmount] = useState(0);
  const [activeOffers, setActiveOffers] = useState<ActiveCouponRow[]>([]);
  const [offersLoading, setOffersLoading] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const run = async () => {
      if (status !== "authenticated") {
        if (!cancelled) setActiveOffers([]);
        return;
      }
      if (!cancelled) setOffersLoading(true);
      try {
        const offers = await fetchActiveCoupons();
        if (!cancelled) setActiveOffers(offers);
      } catch {
        if (!cancelled) setActiveOffers([]);
      } finally {
        if (!cancelled) setOffersLoading(false);
      }
    };
    void run();
    return () => {
      cancelled = true;
    };
  }, [status]);

  const applyCouponCode = useCallback(
    async (code: string) => {
      const trimmed = code.trim();
      if (!trimmed) return;
      setCouponInput(trimmed);
      setCouponMessage(null);
      setCouponApplying(true);
      try {
        const result = await validateCouponCode(trimmed, String(Math.round(orderAmountPaise)));
        if (result?.isValid) {
          setAppliedCouponCode(trimmed);
          setCouponMessage(null);
        } else {
          setAppliedCouponCode(null);
          setCouponMessage(result?.reason || "Coupon not found");
          setDiscountAmount(0);
        }
      } catch (e) {
        setAppliedCouponCode(null);
        setCouponMessage(
          e instanceof ApiEnvelopeError && e.status === 401
            ? "Sign in to apply a coupon."
            : "Unable to check coupon right now. Please try again."
        );
        setDiscountAmount(0);
      } finally {
        setCouponApplying(false);
      }
    },
    [orderAmountPaise]
  );

  const handleApplyCoupon = useCallback(
    () => applyCouponCode(couponInput),
    [applyCouponCode, couponInput]
  );

  const handleRemoveCoupon = useCallback(() => {
    setAppliedCouponCode(null);
    setCouponMessage(null);
    setCouponInput("");
    setDiscountAmount(0);
  }, []);

  return {
    couponInput,
    setCouponInput,
    appliedCouponCode,
    couponMessage,
    setCouponMessage,
    couponApplying,
    discountAmount,
    setDiscountAmount,
    activeOffers,
    offersLoading,
    applyCouponCode,
    handleApplyCoupon,
    handleRemoveCoupon,
  };
}
