import { fetchApiEnvelope } from "@/lib/api-envelope";

export interface CouponValidationResult {
  couponId: string;
  code: string;
  discountType: string;
  discountValue: number;
  discountAmountPaise: string;
  finalAmountPaise: string;
  isValid: boolean;
  /** Human-readable reason, e.g. "Coupon not found", "Coupon is not active", "Coupon has expired". */
  reason: string;
}

/** Immediate coupon check for the bag page's "Apply" button. `orderAmountPaise` is the current
 * item subtotal in paise (not the final total — min-order checks compare against this). */
export async function validateCouponCode(
  code: string,
  orderAmountPaise: string
): Promise<CouponValidationResult | null> {
  return fetchApiEnvelope<CouponValidationResult | null>("/api/checkout/validate-coupon", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ code, orderAmountPaise }),
  });
}
