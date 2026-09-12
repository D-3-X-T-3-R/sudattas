"use client";

import { Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { StatusBadge } from "@/components/admin/status-badge";
import { formatInrFromPaise } from "@/lib/money";
import type { AdminCouponRow } from "@/lib/admin-coupons";

function formatCouponDate(raw: string | null): string {
  if (!raw) return "—";
  const d = new Date(raw);
  if (Number.isNaN(d.getTime())) return raw;
  return d.toLocaleDateString("en-IN", { day: "2-digit", month: "short", year: "numeric" });
}

function formatDiscount(coupon: AdminCouponRow): string {
  if (coupon.discountType === "percentage") return `${coupon.discountValue}% off`;
  return `${formatInrFromPaise(String(coupon.discountValue))} off`;
}

interface CouponRowProps {
  coupon: AdminCouponRow;
  isToggling: boolean;
  onToggleStatus: () => void;
  onRequestDelete: () => void;
}

export function CouponRow({ coupon, isToggling, onToggleStatus, onRequestDelete }: CouponRowProps) {
  return (
    <tr className="border-b border-[var(--color-line)] last:border-0">
      <td className="py-3 pr-4 font-medium text-[var(--color-ink)]">{coupon.code}</td>
      <td className="py-3 pr-4 text-[var(--color-ink)]">{formatDiscount(coupon)}</td>
      <td className="py-3 pr-4 text-[var(--color-muted)]">
        {coupon.minOrderValuePaise ? formatInrFromPaise(String(coupon.minOrderValuePaise)) : "—"}
      </td>
      <td className="py-3 pr-4 text-[var(--color-muted)]">
        {coupon.usageCount ?? 0}
        {coupon.usageLimit ? ` / ${coupon.usageLimit}` : ""}
      </td>
      <td className="py-3 pr-4">
        <StatusBadge label={coupon.status} />
      </td>
      <td className="py-3 pr-4 text-[var(--color-muted)]">
        {formatCouponDate(coupon.startsAt)} &ndash; {formatCouponDate(coupon.endsAt)}
      </td>
      <td className="py-3">
        <div className="flex flex-wrap gap-1.5">
          <Button type="button" size="sm" variant="outline" disabled={isToggling} onClick={onToggleStatus}>
            {coupon.status === "active" ? "Deactivate" : "Activate"}
          </Button>
          <button
            type="button"
            onClick={onRequestDelete}
            aria-label={`Delete coupon ${coupon.code}`}
            className="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-red-50 hover:text-red-600"
          >
            <Trash2 className="h-4 w-4" />
          </button>
        </div>
      </td>
    </tr>
  );
}
