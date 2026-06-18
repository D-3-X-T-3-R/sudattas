"use client";

import { ProductCard } from "@/components/product-card";
import type { Product } from "@/lib/schemas";
import { Button } from "@/components/ui/button";
import { Section } from "@/components/ui/section";
import { ScrollReveal } from "@/components/scroll-reveal";
import { SectionHeader } from "@/components/ui/page-shell";

export interface ShopSectionProps {
  products: Product[];
  wishlist: Record<string, boolean>;
  onToggleWish: (p: Product) => void;
  onQuickView: (p: Product) => void;
  onQuickAdd?: (p: Product) => void;
  onViewAll?: () => void;
}

export function ShopSection({
  products,
  wishlist,
  onToggleWish,
  onQuickView,
  onQuickAdd,
  onViewAll,
}: ShopSectionProps) {
  const preview = products.slice(0, 6);
  if (preview.length === 0) return null;

  return (
    <Section id="shop" className="relative z-0 bg-[var(--background)]">
      <ScrollReveal>
        <SectionHeader
          title="Thoughtfully Designed, Beautifully Crafted"
          centered
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

      <div className="mt-8 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 md:gap-5">
        {preview.map((p, i) => (
          <ScrollReveal key={p.id} delay={i * 0.05}>
            <ProductCard
              product={p}
              wished={!!wishlist[p.id]}
              onToggleWish={onToggleWish}
              onQuickView={onQuickView}
              onQuickAdd={onQuickAdd}
            />
          </ScrollReveal>
        ))}
      </div>

      <div className="mt-8 flex justify-center">
        <Button variant="outline" className="rounded-full px-8" onClick={onViewAll}>
          Shop the Collection
        </Button>
      </div>
    </Section>
  );
}
