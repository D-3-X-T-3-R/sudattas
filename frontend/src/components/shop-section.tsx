"use client";

import { ProductCard } from "@/components/product-card";
import type { Product } from "@/lib/schemas";
import { Section } from "@/components/ui/section";
import { ScrollReveal } from "@/components/scroll-reveal";
import { SectionHeader } from "@/components/ui/page-shell";

export interface ShopSectionProps {
  products: Product[];
  wishlist: Record<string, boolean>;
  onToggleWish: (p: Product) => void;
  onQuickView: (p: Product) => void;
  onViewAll?: () => void;
}

export function ShopSection({
  products,
  wishlist,
  onToggleWish,
  onQuickView,
  onViewAll,
}: ShopSectionProps) {
  const preview = products.slice(0, 4);
  if (preview.length === 0) return null;

  return (
    <Section id="shop">
      <ScrollReveal>
        <SectionHeader
          label="New Arrivals"
          title="Thoughtfully Designed, Beautifully Crafted"
          description="Freshly added pieces curated for this season's celebrations and everyday dressing."
          action={
            <button
              type="button"
              onClick={onViewAll}
              className="text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-green)] hover:text-[var(--color-gold)]"
            >
              View All
            </button>
          }
        />
      </ScrollReveal>

      <div className="mt-8 grid grid-cols-2 gap-4 md:grid-cols-4 md:gap-5">
        {preview.map((p, i) => (
          <ScrollReveal key={p.id} delay={i * 0.05}>
            <ProductCard
              product={p}
              wished={!!wishlist[p.id]}
              onToggleWish={onToggleWish}
              onQuickView={onQuickView}
            />
          </ScrollReveal>
        ))}
      </div>
    </Section>
  );
}
