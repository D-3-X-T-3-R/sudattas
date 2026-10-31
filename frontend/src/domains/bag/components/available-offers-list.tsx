"use client";

import { formatInrFromPaise } from "@/lib/money";
import type { ActiveCouponRow } from "@/lib/active-coupons";

function describeDiscount(coupon: ActiveCouponRow): string {
  if (coupon.discountType === "percentage") return `${coupon.discountValue}% off`;
  return `${formatInrFromPaise(String(coupon.discountValue))} off`;
}

function describeConditions(coupon: ActiveCouponRow): string | null {
  if (!coupon.minOrderValuePaise) return null;
  return `on orders above ${formatInrFromPaise(String(coupon.minOrderValuePaise))}`;
}

interface AvailableOffersListProps {
  offers: ActiveCouponRow[];
  isLoading: boolean;
  appliedCouponCode?: string | null;
  onSelect: (code: string) => void;
}

/** "Available offers" — lets a customer browse currently-usable coupons instead of needing to
 * already know a code. Applying still runs the same real eligibility checks as typing a code by
 * hand; this list is just a discovery convenience. */
export function AvailableOffersList({ offers, isLoading, appliedCouponCode, onSelect }: AvailableOffersListProps) {
  if (isLoading) return null;
  if (offers.length === 0) return null;

  return (
    <div className="mt-4 border-t border-[var(--color-line)] pt-4">
      <p className="mb-2 text-xs font-semibold uppercase tracking-[0.14em] text-[var(--color-muted)]">
        Available offers
      </p>
      <ul className="space-y-2">
        {offers.map((offer) => {
          const isApplied = appliedCouponCode === offer.code;
          const conditions = describeConditions(offer);
          return (
            <li
              key={offer.couponId}
              className="flex items-center justify-between gap-3 rounded-md border border-dashed border-[var(--color-line)] px-3 py-2"
            >
              <div className="min-w-0">
                <p className="text-sm font-semibold text-[var(--color-ink)]">
                  {offer.code} <span className="font-normal text-[var(--color-muted)]">— {describeDiscount(offer)}</span>
                </p>
                {conditions ? <p className="mt-0.5 text-xs text-[var(--color-muted)]">{conditions}</p> : null}
              </div>
              <button
                type="button"
                onClick={() => onSelect(offer.code)}
                disabled={isApplied}
                className="shrink-0 rounded-md border border-[var(--color-green)] px-3 py-1.5 text-xs font-semibold uppercase tracking-[0.08em] text-[var(--color-green)] transition hover:bg-[var(--color-green)] hover:text-white disabled:cursor-not-allowed disabled:border-[var(--color-line)] disabled:text-[var(--color-muted)] disabled:hover:bg-transparent"
              >
                {isApplied ? "Applied" : "Apply"}
              </button>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
