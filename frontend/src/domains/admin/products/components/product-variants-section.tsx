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

/** Rows with the same (size, color) combination represent the exact same real item split
 * across two DB rows with independently-tracked stock — the backend rejects this on save, but
 * flagging it inline here is faster to notice than a save-time error. Blank-size rows are
 * excluded; the separate "must have a size" check already covers those. */
export function findDuplicateVariantIndexes(variants: AdminProductVariantRow[]): Set<number> {
  const firstIndexForKey = new Map<string, number>();
  const duplicates = new Set<number>();
  variants.forEach((v, idx) => {
    const sizeId = v.sizeId?.trim();
    if (!sizeId) return;
    const key = `${sizeId}::${v.colorId?.trim() ?? ""}`;
    const firstIndex = firstIndexForKey.get(key);
    if (firstIndex === undefined) {
      firstIndexForKey.set(key, idx);
    } else {
      duplicates.add(firstIndex);
      duplicates.add(idx);
    }
  });
  return duplicates;
}

export function ProductVariantsSection({
  variants,
  setVariants,
  sizes,
  colors,
}: ProductVariantsSectionProps) {
  const duplicateIndexes = findDuplicateVariantIndexes(variants);
  return (
    <div className="mt-8 border-t border-[var(--color-line)] pt-5">
      <h3 className="text-[15px] font-semibold text-[var(--color-ink)]">
        Sizes &amp; colors *
      </h3>
      <p className="mt-1.5 text-sm text-[var(--color-muted)]">
        Add size/color combinations. Each one can have an extra price and starting stock.
      </p>
      <div className="mt-3 space-y-2.5">
        {variants.map((v, idx) => {
          const isDuplicate = duplicateIndexes.has(idx);
          return (
          <div
            key={idx}
            className={cn(
              "flex flex-wrap items-end gap-2.5 rounded-xl border bg-white/40 p-3.5",
              isDuplicate ? "border-red-300 bg-red-50/60" : "border-[var(--color-line)]"
            )}
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
            {isDuplicate && (
              <p className="w-full text-sm text-red-600" role="alert">
                Duplicate size/color — this combination is already used by another row above. Remove one or change its size/color.
              </p>
            )}
          </div>
          );
        })}
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
