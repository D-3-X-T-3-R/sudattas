"use client";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import type { ColorRow, SizeRow } from "@/lib/admin-queries";
import type { AdminProductVariantRow } from "@/lib/schemas";
import type { Dispatch, SetStateAction } from "react";

type ProductVariantsSectionProps = {
  variants: AdminProductVariantRow[];
  setVariants: Dispatch<SetStateAction<AdminProductVariantRow[]>>;
  sizes: SizeRow[];
  colors: ColorRow[];
};

export function ProductVariantsSection({
  variants,
  setVariants,
  sizes,
  colors,
}: ProductVariantsSectionProps) {
  return (
    <div className="mt-8 border-t border-[var(--color-line)] pt-5">
      <h3 className="text-[15px] font-semibold text-[var(--color-ink)]">
        Sizes &amp; colors *
      </h3>
      <p className="mt-1.5 text-sm text-[var(--color-muted)]">
        Add size/color combinations. Each one can have an extra price and starting stock.
      </p>
      <div className="mt-3 space-y-2.5">
        {variants.map((v, idx) => (
          <div
            key={idx}
            className="flex flex-wrap items-end gap-2.5 rounded-xl border border-[var(--color-line)] bg-white/40 p-3.5"
          >
            <select
              className={cn(
                "h-11 min-w-[7rem] rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px]",
                "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
              )}
              value={v.sizeId}
              onChange={(e) =>
                setVariants((prev) => {
                  const next = [...prev];
                  next[idx] = { ...next[idx], sizeId: e.target.value };
                  return next;
                })
              }
              aria-label={`Variant ${idx + 1} size`}
            >
              <option value="">Select size</option>
              {sizes.map((s) => (
                <option key={s.sizeId} value={s.sizeId}>
                  {s.sizeName}
                </option>
              ))}
            </select>
            <select
              className={cn(
                "h-11 min-w-[7rem] rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px]",
                "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
              )}
              value={v.colorId}
              onChange={(e) =>
                setVariants((prev) => {
                  const next = [...prev];
                  next[idx] = { ...next[idx], colorId: e.target.value };
                  return next;
                })
              }
              aria-label={`Variant ${idx + 1} color`}
            >
              <option value="">Select color</option>
              {colors.map((c) => (
                <option key={c.colorId} value={c.colorId}>
                  {c.colorName}
                </option>
              ))}
            </select>
            <Input
              type="text"
              placeholder="Extra price (₹)"
              value={v.additionalPricePaise}
              onChange={(e) =>
                setVariants((prev) => {
                  const next = [...prev];
                  next[idx] = { ...next[idx], additionalPricePaise: e.target.value };
                  return next;
                })
              }
              className="h-11 w-32 rounded-lg text-[15px]"
              aria-label={`Variant ${idx + 1} extra price in rupees`}
            />
            <Input
              type="text"
              placeholder="Qty"
              value={v.quantityAvailable}
              onChange={(e) =>
                setVariants((prev) => {
                  const next = [...prev];
                  next[idx] = { ...next[idx], quantityAvailable: e.target.value };
                  return next;
                })
              }
              className="h-11 w-24 rounded-lg text-[15px]"
              aria-label={`Variant ${idx + 1} quantity`}
            />
            <Input
              type="text"
              placeholder="Reorder"
              value={v.reorderLevel}
              onChange={(e) =>
                setVariants((prev) => {
                  const next = [...prev];
                  next[idx] = { ...next[idx], reorderLevel: e.target.value };
                  return next;
                })
              }
              className="h-11 w-24 rounded-lg text-[15px]"
              aria-label={`Variant ${idx + 1} reorder level`}
            />
            <Button
              type="button"
              variant="outline"
              className="text-red-600"
              onClick={() => setVariants((prev) => prev.filter((_, i) => i !== idx))}
            >
              Remove
            </Button>
          </div>
        ))}
        <Button
          type="button"
          variant="outline"
          className="rounded-lg border-[var(--color-line)]"
          onClick={() =>
            setVariants((prev) => [
              ...prev,
              {
                sizeId: "",
                colorId: "",
                additionalPricePaise: "",
                quantityAvailable: "0",
                reorderLevel: "",
              },
            ])
          }
        >
          + Add variant
        </Button>
      </div>
    </div>
  );
}
