"use client";

import { useState } from "react";
import Image from "next/image";
import { ChevronRight } from "lucide-react";
import { AnimatePresence, motion } from "framer-motion";
import { Section } from "@/components/ui/section";
import { ScrollReveal } from "@/components/scroll-reveal";
import { goTo } from "@/hooks/use-scroll-to";
import { SectionHeader } from "@/components/ui/page-shell";
import { COLLECTION_IMAGES } from "@/lib/seed-data";

type DisplayCollection = {
  key: string;
  blurb: string;
  thumbnailUrl?: string;
};

const categoryCard = {
  default: {
    media: "aspect-[4/5]",
    panel: "min-h-[116px] p-4",
    title: "text-[1.25rem] md:text-[1.4rem]",
  },
} as const;

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
  const tone = categoryCard.default;

  return (
    <button
      type="button"
      onClick={() => {
        onPickCategory(item.key);
        goTo("explore", reduceMotion);
      }}
      className="group h-full w-full overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] text-left shadow-[var(--shadow-subtle)]"
    >
      <div className={`relative w-full overflow-hidden ${tone.media}`}>
        <Image
          src={imgSrc}
          alt={item.key}
          fill
          className="object-cover transition duration-500 ease-out group-hover:scale-[1.03]"
          sizes="(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 25vw"
          loading="lazy"
        />
      </div>
      <div className={`border-t border-[var(--color-line)] ${tone.panel}`}>
        <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-[var(--color-muted)]">
          Category
        </p>
        <p className={`mt-1 font-display leading-tight text-[var(--color-ink)] ${tone.title}`}>
          {item.blurb}
        </p>
        <span className="mt-3 inline-flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-green)]">
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
        <SectionHeader
          label="Collections"
          title="Shop By Category"
          description="Explore signature silhouettes across sarees, kurtis, and festive edits."
          action={
            items.length > 4 ? (
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

      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 sm:gap-5 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
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
    </Section>
  );
}
