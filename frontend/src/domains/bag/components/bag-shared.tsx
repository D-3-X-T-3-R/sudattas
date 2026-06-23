import type { SVGProps } from "react";
import type { Product } from "@/lib/schemas";

type CatalogSize = { sizeId: string; sizeName: string };
type IconProps = SVGProps<SVGSVGElement>;

/** Only sizes that are in stock for this product. */
export function buildSizeOptions(product: Product, catalog: CatalogSize[]): CatalogSize[] {
  const stock = product.variantStock ?? [];
  const byId = new Map(stock.map((variant) => [variant.sizeId, variant]));

  if (catalog.length > 0) {
    const seenIds = new Set<string>();
    return catalog
      .filter((size) => size.sizeName.toLowerCase() !== "free size")
      .map((size) => {
        if (seenIds.has(size.sizeId)) return null;
        const variant = byId.get(size.sizeId);
        if (!variant || variant.quantity <= 0) return null;
        seenIds.add(size.sizeId);
        return { sizeId: size.sizeId, sizeName: size.sizeName };
      })
      .filter((value): value is CatalogSize => value !== null);
  }

  const seen = new Set<string>();
  return stock
    .filter((variant) => {
      if (variant.sizeName.toLowerCase() === "free size") return false;
      if (variant.quantity <= 0) return false;
      if (seen.has(variant.sizeId)) return false;
      seen.add(variant.sizeId);
      return true;
    })
    .map((variant) => ({ sizeId: variant.sizeId, sizeName: variant.sizeName }));
}

export function CheckIcon(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" {...props}>
      <path d="m5 12 5 5L20 7" />
    </svg>
  );
}

export function BagIcon(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" {...props}>
      <path d="M6 8h12l-1 11H7L6 8Z" />
      <path d="M9 8a3 3 0 0 1 6 0" />
    </svg>
  );
}

export function HeartIcon(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" {...props}>
      <path d="M12 20s-6.7-4.35-9-8.2C1.3 8.6 3.1 5 7 5c2.2 0 3.6 1.2 5 3 1.4-1.8 2.8-3 5-3 3.9 0 5.7 3.6 4 6.8-2.3 3.85-9 8.2-9 8.2Z" />
    </svg>
  );
}

export function TrashIcon(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" {...props}>
      <path d="M3 6h18" />
      <path d="M8 6V4h8v2" />
      <path d="m19 6-1 14H6L5 6" />
      <path d="M10 11v6" />
      <path d="M14 11v6" />
    </svg>
  );
}

export function ArrowRightIcon(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" {...props}>
      <path d="M5 12h14" />
      <path d="m13 5 7 7-7 7" />
    </svg>
  );
}

export function GoldDivider() {
  return (
    <div className="flex items-center gap-3">
      <div className="h-px flex-1 bg-gradient-to-r from-transparent via-[#C9A646]/65 to-transparent" />
      <div className="h-1.5 w-1.5 rounded-full bg-[#C9A646]" />
      <div className="h-px flex-1 bg-gradient-to-r from-transparent via-[#C9A646]/65 to-transparent" />
    </div>
  );
}
