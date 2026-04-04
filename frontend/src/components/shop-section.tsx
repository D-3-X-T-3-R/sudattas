"use client";

import { ProductCard } from "@/components/product-card";
import type { Product } from "@/lib/schemas";
import { Section } from "@/components/ui/section";
import { SectionHeading, Kicker } from "@/components/ui/typography";
import { ScrollReveal } from "@/components/scroll-reveal";

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
  return (
    <Section id="shop">
      <ScrollReveal>
        <div className="flex items-end justify-between border-b border-[var(--color-line)] pb-8">
          <div>
            <Kicker className="invisible text-[var(--color-muted)]">Shop</Kicker>
            <SectionHeading size="lg" className="mt-3 uppercase tracking-[0.18em] text-[var(--color-accent-gold)]">
              New arrivals
            </SectionHeading>
          </div>
          <button
            type="button"
            onClick={onViewAll}
            className="text-xs font-semibold uppercase tracking-[0.18em] text-[var(--color-ink)] transition-colors hover:text-[var(--color-accent-brown)]"
          >
            View all
          </button>
        </div>
      </ScrollReveal>

      {preview.length > 0 && (
        <div className="mt-14 grid grid-cols-1 gap-8 sm:grid-cols-2 lg:grid-cols-4">
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
      )}
    </Section>
  );
}
