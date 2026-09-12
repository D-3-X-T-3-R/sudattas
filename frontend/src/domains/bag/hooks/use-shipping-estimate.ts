import { useCallback, useEffect, useState } from "react";
import { fetchApiEnvelope } from "@/lib/api-envelope";

export type ShippingEstimate = {
  shippingAmountPaise: string;
  courierName?: string | null;
  estimatedDeliveryDays?: number | null;
  itemSubtotalPaise: string;
  orderTotalPaise: string;
  quoteAvailable: boolean;
  note?: string | null;
};

interface UseShippingEstimateParams {
  status: string;
  selectedAddressId: string | null;
  cartLinesLength: number;
  selectedLineIds: Set<string>;
  appliedCouponCode: string | null;
  selectedSubtotal: number;
  cartSignature: string;
  setCouponMessage: (message: string | null) => void;
  setDiscountAmount: (amount: number) => void;
}

/**
 * Live shipping + coupon-discount estimate for the bag page. `refreshEstimate` is exposed
 * (rather than only firing internally) so checkout can force one final live check right before
 * payment — a coupon can go stale while the tab sits open, and the debounced effect here only
 * re-fires on local input changes, never on external state changing (e.g. an admin deactivating
 * a coupon in another tab).
 */
export function useShippingEstimate({
  status,
  selectedAddressId,
  cartLinesLength,
  selectedLineIds,
  appliedCouponCode,
  selectedSubtotal,
  cartSignature,
  setCouponMessage,
  setDiscountAmount,
}: UseShippingEstimateParams) {
  const [shippingAmount, setShippingAmount] = useState(0);
  const [shippingLoading, setShippingLoading] = useState(false);
  const [shippingNote, setShippingNote] = useState<string | null>(null);

  const refreshEstimate = useCallback(
    async (signal?: AbortSignal): Promise<ShippingEstimate | null> => {
      if (status !== "authenticated" || !selectedAddressId || cartLinesLength === 0) {
        setShippingAmount(0);
        setDiscountAmount(0);
        setShippingNote(status === "authenticated" ? "Select address to calculate shipping." : "Sign in to calculate shipping.");
        return null;
      }
      const selectedCartLineIds = [...selectedLineIds];
      if (selectedCartLineIds.length === 0) {
        setShippingAmount(0);
        setDiscountAmount(0);
        setShippingNote("Select at least one bag item to calculate shipping.");
        return null;
      }
      setShippingLoading(true);
      try {
        const row = await fetchApiEnvelope<ShippingEstimate>("/api/checkout/shipping-estimate", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            shippingAddressId: selectedAddressId,
            selectedCartLineIds,
            couponCode: appliedCouponCode || undefined,
          }),
          signal,
        });
        const paise = Number.parseInt(row.shippingAmountPaise, 10);
        setShippingAmount(Number.isFinite(paise) ? paise / 100 : 0);
        // When a coupon is applied, `note` is a rejection reason meant for the coupon field, not
        // the shipping line — keep the shipping note clean in that case.
        setShippingNote(appliedCouponCode ? null : row.note ?? null);
        if (appliedCouponCode) {
          setCouponMessage(row.note ?? null);
          const itemSubtotalPaise = Number.parseInt(row.itemSubtotalPaise, 10);
          const discountRupees = Number.isFinite(itemSubtotalPaise)
            ? Math.max(0, selectedSubtotal - itemSubtotalPaise / 100)
            : 0;
          setDiscountAmount(row.note ? 0 : discountRupees);
        } else {
          setCouponMessage(null);
          setDiscountAmount(0);
        }
        return row;
      } catch (e) {
        if (e instanceof DOMException && e.name === "AbortError") return null;
        setShippingAmount(0);
        setDiscountAmount(0);
        setShippingNote("Unable to fetch live shipping right now.");
        return null;
      } finally {
        if (!signal?.aborted) {
          setShippingLoading(false);
        }
      }
    },
    [
      status,
      selectedAddressId,
      cartLinesLength,
      selectedLineIds,
      appliedCouponCode,
      selectedSubtotal,
      setCouponMessage,
      setDiscountAmount,
    ]
  );

  useEffect(() => {
    const controller = new AbortController();
    const timer = window.setTimeout(() => {
      void refreshEstimate(controller.signal);
    }, 180);
    return () => {
      window.clearTimeout(timer);
      controller.abort();
    };
    // cartSignature intentionally included so an in-place quantity edit re-triggers even though
    // it doesn't change selectedLineIds identity.
  }, [refreshEstimate, cartSignature]);

  return { shippingAmount, shippingLoading, shippingNote, refreshEstimate };
}
