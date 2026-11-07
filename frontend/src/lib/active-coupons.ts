import { fetchApiEnvelope } from "@/lib/api-envelope";

export interface ActiveCouponRow {
  couponId: string;
  code: string;
  /** "percentage" | "fixed_amount" */
  discountType: string;
  discountValue: number;
  minOrderValuePaise?: number | null;
  /** RFC3339; absent if the coupon never expires. */
  endsAt?: string | null;
}

/** Currently-usable coupons for the "Available offers" list on the bag page — no code required
 * up front. Backend already filters to active/in-window/not-exhausted; this doesn't re-check
 * per-cart eligibility (scope/per-customer limit), which is still enforced for real on apply. */
export async function fetchActiveCoupons(): Promise<ActiveCouponRow[]> {
  return fetchApiEnvelope<ActiveCouponRow[]>("/api/checkout/active-coupons", { cache: "no-store" });
}
