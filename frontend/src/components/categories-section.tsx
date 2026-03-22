"use client";

import { useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { Section } from "@/components/ui/section";
import { SectionHeading, Kicker } from "@/components/ui/typography";
import { ScrollReveal } from "@/components/scroll-reveal";
import { CollectionCard, type DisplayCollection } from "@/components/collections-section";
import { goTo } from "@/hooks/use-scroll-to";

export interface CategoriesSectionProps {
  categories: { categoryId: string; name: string; thumbnailUrl?: string }[];
  onPickCategory: (name: string) => void;
  reduceMotion?: boolean;
}

export function CategoriesSection({
  categories,
  onPickCategory,
  reduceMotion = false,
}: CategoriesSectionProps) {
  const [showAll, setShowAll] = useState(false);

  if (categories.length === 0) return null;

  const items: DisplayCollection[] = categories.map((c) => ({
    key: c.name,
    blurb: c.name,
    thumbnailUrl: c.thumbnailUrl,
  }));

  const visible = showAll ? items : items.slice(0, 4);

  return (
    <Section id="category-collections">
      <ScrollReveal>
        <div className="flex flex-col gap-4 border-b border-[var(--color-line)] pb-8 sm:flex-row sm:items-end sm:justify-between">
          <SectionHeading size="lg" className="uppercase tracking-[0.18em] text-[var(--color-accent-gold)]">
            Collections
          </SectionHeading>
          {items.length > 4 && (
            <button
              type="button"
              onClick={() => setShowAll((v) => !v)}
              className="hidden text-xs font-semibold uppercase tracking-[0.18em] text-[var(--color-ink)] transition-colors hover:text-[var(--color-accent-brown)] sm:inline-flex"
            >
              {showAll ? "Show less" : "View all"}
            </button>
          )}
        </div>
      </ScrollReveal>

      <div className="mt-12 grid grid-cols-2 gap-4 md:grid-cols-4">
        <AnimatePresence initial={false}>
          {visible.map((item, idx) => (
            <motion.div
              key={item.key}
              initial={{ opacity: 0, y: 20, scale: 0.97 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: 16, scale: 0.97 }}
              transition={{ duration: 0.35, ease: [0.22, 1, 0.36, 1], delay: idx < 4 ? 0 : (idx - 4) * 0.06 }}
            >
              <CollectionCard
                c={item}
                idx={idx}
                setCollection={(name) => {
                  onPickCategory(name);
                  goTo("explore", reduceMotion);
                }}
                reduceMotion={reduceMotion}
              />
            </motion.div>
          ))}
        </AnimatePresence>
      </div>
    </Section>
  );
}
