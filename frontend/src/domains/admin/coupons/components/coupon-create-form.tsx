"use client";

import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { createCouponAdmin } from "@/lib/admin-coupons";

function toRfc3339FromLocalInput(value: string): string {
  // <input type="datetime-local"> gives "YYYY-MM-DDTHH:mm" in local time, no offset.
  const d = new Date(value);
  return d.toISOString();
}

export function CouponCreateForm() {
  const queryClient = useQueryClient();
  const [code, setCode] = useState("");
  const [discountType, setDiscountType] = useState<"percentage" | "fixed_amount">("percentage");
  const [discountValue, setDiscountValue] = useState("");
  const [minOrderValue, setMinOrderValue] = useState("");
  const [usageLimit, setUsageLimit] = useState("");
  const [maxUsesPerCustomer, setMaxUsesPerCustomer] = useState("");
  const [startsAt, setStartsAt] = useState(() => new Date().toISOString().slice(0, 16));
  const [endsAt, setEndsAt] = useState("");
  const [error, setError] = useState("");

  const createMutation = useMutation({
    mutationFn: () =>
      createCouponAdmin({
        code: code.trim().toUpperCase(),
        discountType,
        discountValue:
          discountType === "fixed_amount"
            ? Math.round(Number.parseFloat(discountValue) * 100)
            : Number.parseInt(discountValue, 10),
        minOrderValuePaise: minOrderValue.trim() ? Math.round(Number.parseFloat(minOrderValue) * 100) : null,
        usageLimit: usageLimit.trim() ? Number.parseInt(usageLimit, 10) : null,
        maxUsesPerCustomer: maxUsesPerCustomer.trim() ? Number.parseInt(maxUsesPerCustomer, 10) : null,
        startsAt: toRfc3339FromLocalInput(startsAt),
        endsAt: endsAt.trim() ? toRfc3339FromLocalInput(endsAt) : null,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin", "coupons"] });
      setCode("");
      setDiscountValue("");
      setMinOrderValue("");
      setUsageLimit("");
      setMaxUsesPerCustomer("");
      setEndsAt("");
      setError("");
    },
    onError: (err: Error) => setError(err.message || "Failed to create coupon."),
  });

  const canSubmit = code.trim().length > 0 && discountValue.trim().length > 0 && startsAt.trim().length > 0;

  return (
    <div className="mt-4 border-t border-[var(--color-line)] pt-4">
      <p className="mb-2 text-sm font-medium text-[var(--color-muted)]">Create a coupon</p>
      <div className="grid gap-2.5 sm:grid-cols-2 lg:grid-cols-4">
        <Input
          value={code}
          onChange={(e) => setCode(e.target.value.toUpperCase())}
          placeholder="Code (e.g. WELCOME20)"
          className="h-10 rounded-lg text-[15px] uppercase"
        />
        <select
          value={discountType}
          onChange={(e) => setDiscountType(e.target.value as "percentage" | "fixed_amount")}
          className="h-10 rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px] focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
        >
          <option value="percentage">Percentage off</option>
          <option value="fixed_amount">Fixed amount off</option>
        </select>
        <Input
          value={discountValue}
          onChange={(e) => setDiscountValue(e.target.value)}
          placeholder={discountType === "percentage" ? "Discount % (e.g. 10)" : "Discount ₹ (e.g. 200)"}
          inputMode="decimal"
          className="h-10 rounded-lg text-[15px]"
        />
        <Input
          value={minOrderValue}
          onChange={(e) => setMinOrderValue(e.target.value)}
          placeholder="Min order ₹ (optional)"
          inputMode="decimal"
          className="h-10 rounded-lg text-[15px]"
        />
        <Input
          value={usageLimit}
          onChange={(e) => setUsageLimit(e.target.value)}
          placeholder="Total usage limit (optional)"
          inputMode="numeric"
          className="h-10 rounded-lg text-[15px]"
        />
        <Input
          value={maxUsesPerCustomer}
          onChange={(e) => setMaxUsesPerCustomer(e.target.value)}
          placeholder="Max uses per customer (optional)"
          inputMode="numeric"
          className="h-10 rounded-lg text-[15px]"
        />
        <label className="flex flex-col gap-1 text-xs text-[var(--color-muted)]">
          Starts at
          <input
            type="datetime-local"
            value={startsAt}
            onChange={(e) => setStartsAt(e.target.value)}
            className="h-10 rounded-lg border border-[var(--color-line)] px-3 text-[15px] focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
          />
        </label>
        <label className="flex flex-col gap-1 text-xs text-[var(--color-muted)]">
          Ends at (optional)
          <input
            type="datetime-local"
            value={endsAt}
            onChange={(e) => setEndsAt(e.target.value)}
            className="h-10 rounded-lg border border-[var(--color-line)] px-3 text-[15px] focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
          />
        </label>
      </div>
      <Button
        type="button"
        variant="outline"
        size="sm"
        className="mt-3"
        disabled={!canSubmit || createMutation.isPending}
        onClick={() => createMutation.mutate()}
      >
        {createMutation.isPending ? "Creating…" : "Create coupon"}
      </Button>
      {error && (
        <p className="mt-2 text-sm text-red-600" role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
