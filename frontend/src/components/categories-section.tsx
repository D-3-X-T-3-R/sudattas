"use client";

import { useState } from "react";
import Image from "next/image";
import { ChevronRight } from "lucide-react";
import { AnimatePresence, motion } from "framer-motion";
import { ScrollReveal } from "@/components/scroll-reveal";
import { goTo } from "@/hooks/use-scroll-to";
import { SectionHeader } from "@/components/ui/page-shell";
import { COLLECTION_IMAGES } from "@/lib/seed-data";

type DisplayCollection = {
  key: string;
  blurb: string;
  thumbnailUrl?: string;
  isPlaceholder?: boolean;
};

const PLACEHOLDER_CATEGORIES: DisplayCollection[] = [
  { key: "Sarees", blurb: "Sarees", isPlaceholder: true },
  { key: "Kurtas & Sets", blurb: "Kurtas & Sets", isPlaceholder: true },
  { key: "Lehengas", blurb: "Lehengas", isPlaceholder: true },
];

const MIN_CATEGORY_TILES = 3;

export interface CategoriesSectionProps {
  categories: { categoryId: string; name: string; thumbnailUrl?: string }[];
  onPickCategory: (name: string) => void;
  reduceMotion?: boolean;
}

function CategoryCard({
  item,
  idx,
  onPickCategory,
  reduceMotion,
}: {
  item: DisplayCollection;
  idx: number;
  onPickCategory: (name: string) => void;
  reduceMotion: boolean;
}) {
  const imgSrc = item.thumbnailUrl || COLLECTION_IMAGES[idx % COLLECTION_IMAGES.length];

  return (
    <button
      type="button"
      onClick={() => {
        if (!item.isPlaceholder) onPickCategory(item.key);
        goTo("explore", reduceMotion);
      }}
      className="group h-full w-full overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] text-left shadow-[var(--shadow-subtle)]"
    >
      <div className="relative aspect-[4/5] w-full overflow-hidden">
        <Image
          src={imgSrc}
          alt={item.key}
          fill
          className="object-cover transition duration-500 ease-out group-hover:scale-[1.03]"
          sizes="(max-width: 768px) 50vw, 25vw"
          loading="lazy"
        />
      </div>
      <div className="flex items-center justify-between gap-3 border-t border-[var(--color-line)] p-3.5 md:p-4">
        <div className="min-w-0">
          <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-[var(--color-muted)]">
            Category
          </p>
          <p className="mt-1 truncate font-display text-[1.1rem] leading-[1.15] text-[var(--color-ink)] md:text-[1.3rem]">
            {item.blurb}
          </p>
        </div>
        <span className="inline-flex shrink-0 items-center gap-1.5 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-green)]">
          Explore
          <ChevronRight className="h-3.5 w-3.5" />
        </span>
      </div>
    </button>
  );
}

export function CategoriesSection({
  categories,
  onPickCategory,
  reduceMotion = false,
}: CategoriesSectionProps) {
  const [showAll, setShowAll] = useState(false);

  const realItems: DisplayCollection[] = categories.map((c) => ({
    key: c.name,
    blurb: c.name,
    thumbnailUrl: c.thumbnailUrl,
  }));

  const realKeys = new Set(realItems.map((item) => item.key));
  const availablePlaceholders = PLACEHOLDER_CATEGORIES.filter((p) => !realKeys.has(p.key));

  const items: DisplayCollection[] =
    realItems.length >= MIN_CATEGORY_TILES
      ? realItems
      : [
          ...realItems,
          ...availablePlaceholders.slice(0, MIN_CATEGORY_TILES - realItems.length),
        ];

  const visible = showAll ? items : items.slice(0, 6);

  return (
    <section id="category-collections" className="bg-[var(--color-section-alt)] py-6 md:py-10">
      <div className="mx-auto w-full max-w-[var(--container-max)] px-[var(--gutter-mobile)] md:px-[var(--gutter-tablet)]">
        <ScrollReveal>
          <SectionHeader
            title="Shop By Category"
            centered
            className="pb-4 md:pb-5"
            action={
              items.length > 6 ? (
                <button
                  type="button"
                  onClick={() => setShowAll((v) => !v)}
                  className="text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-green)] hover:text-[var(--color-gold)]"
                >
                  {showAll ? "Show Less" : "View All"}
                </button>
              ) : null
            }
          />
        </ScrollReveal>

        <div className="mx-auto mt-4 grid max-w-[1450px] grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
          <AnimatePresence initial={false}>
            {visible.map((item, idx) => (
              <motion.div
                key={item.key}
                initial={{ opacity: 0, y: 14 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: 10 }}
                transition={{ duration: 0.28, ease: [0.22, 1, 0.36, 1] }}
              >
                <CategoryCard
                  item={item}
                  idx={idx}
                  onPickCategory={onPickCategory}
                  reduceMotion={reduceMotion}
                />
              </motion.div>
            ))}
          </AnimatePresence>
        </div>
      </div>
    </section>
  );
}
