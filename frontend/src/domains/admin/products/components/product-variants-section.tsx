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
    <div className="mt-8 border-t border-[var(--color-line)] pt-4">
      <h3 className="text-xs font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">
        Variants *
      </h3>
      <p className="mt-2 text-xs text-[var(--color-muted)]">
        Add size/color combinations. Each variant can have an extra price (paise) and initial stock.
      </p>
      <div className="mt-3 space-y-2">
        {variants.map((v, idx) => (
          <div
            key={idx}
            className="flex flex-wrap items-end gap-2 rounded-lg border border-[var(--color-line)] bg-white/40 p-3"
          >
            <select
              className={cn(
                "h-9 min-w-[6rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
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
                "h-9 min-w-[6rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
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
              placeholder="Extra price (paise)"
              value={v.additionalPricePaise}
              onChange={(e) =>
                setVariants((prev) => {
                  const next = [...prev];
                  next[idx] = { ...next[idx], additionalPricePaise: e.target.value };
                  return next;
                })
              }
              className="h-9 w-28 rounded-md"
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
              className="h-9 w-20 rounded-md"
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
              className="h-9 w-20 rounded-md"
            />
            <Button
              type="button"
              variant="outline"
              size="sm"
              className="h-9 text-red-600"
              onClick={() => setVariants((prev) => prev.filter((_, i) => i !== idx))}
            >
              Remove
            </Button>
          </div>
        ))}
        <Button
          type="button"
          variant="outline"
          size="sm"
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
