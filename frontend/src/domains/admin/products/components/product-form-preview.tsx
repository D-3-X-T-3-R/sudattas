"use client";
/* eslint-disable @next/next/no-img-element */

import { useState } from "react";
import { ChevronDown, Eye } from "lucide-react";
import { cn } from "@/lib/utils";

function formatPreviewPrice(priceRupees: string): string {
  const n = Number.parseFloat(priceRupees);
  if (!Number.isFinite(n) || n <= 0) return "—";
  return `₹${n.toLocaleString("en-IN", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

interface ProductFormPreviewProps {
  name: string;
  priceRupees: string;
  categoryName: string;
  statusLabel: string;
  imageUrl?: string;
}

function PreviewCard({ name, priceRupees, categoryName, statusLabel, imageUrl }: ProductFormPreviewProps) {
  return (
    <div className="overflow-hidden rounded-xl border border-[var(--color-line)] bg-white">
      <div className="aspect-[2/3] w-full bg-[var(--color-surface-soft)]">
        {imageUrl ? (
          <img src={imageUrl} alt="" className="h-full w-full object-cover" />
        ) : (
          <div className="flex h-full items-center justify-center text-sm text-[var(--color-muted)]">
            No photo yet
          </div>
        )}
      </div>
      <div className="space-y-1 border-t border-[var(--color-line)] p-3.5">
        <p className="line-clamp-1 text-[15px] font-semibold text-[var(--color-ink)]">
          {name.trim() || "Untitled product"}
        </p>
        <p className="text-sm text-[var(--color-muted)]">{categoryName || "No category yet"}</p>
        <div className="flex items-center justify-between pt-1">
          <p className="text-[15px] font-medium text-[var(--color-ink)]">{formatPreviewPrice(priceRupees)}</p>
          <span className="rounded-full bg-[var(--color-surface-soft)] px-2.5 py-0.5 text-xs font-medium text-[var(--color-muted)]">
            {statusLabel === "—" ? "Draft" : statusLabel}
          </span>
        </div>
      </div>
    </div>
  );
}

/** Read-only live preview of how the product will look in the store — updates as the form is filled in. */
export function ProductFormPreview(props: ProductFormPreviewProps) {
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <>
      {/* Desktop: always visible, sticky */}
      <div className="hidden lg:block">
        <p className="mb-2 flex items-center gap-1.5 text-sm font-semibold text-[var(--color-muted)]">
          <Eye className="h-3.5 w-3.5" />
          Live preview
        </p>
        <PreviewCard {...props} />
      </div>

      {/* Mobile: collapsed behind a disclosure so it doesn't push the form down */}
      <div className="lg:hidden">
        <button
          type="button"
          onClick={() => setMobileOpen((v) => !v)}
          className="flex w-full items-center justify-between gap-2 rounded-xl border border-[var(--color-line)] bg-white px-4 py-3 text-sm font-semibold text-[var(--color-ink)]"
          aria-expanded={mobileOpen}
        >
          <span className="flex items-center gap-1.5">
            <Eye className="h-4 w-4" />
            Preview
          </span>
          <ChevronDown className={cn("h-4 w-4 transition-transform", mobileOpen && "rotate-180")} />
        </button>
        {mobileOpen ? (
          <div className="mt-3 max-w-xs">
            <PreviewCard {...props} />
          </div>
        ) : null}
      </div>
    </>
  );
}
