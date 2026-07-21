"use client";

import { useState } from "react";
import Image from "next/image";
import { ChevronRight } from "lucide-react";
import { AnimatePresence, motion } from "framer-motion";
import { COLLECTIONS } from "@/lib/constants";
import { COLLECTION_IMAGES } from "@/lib/seed-data";
import { goTo } from "@/hooks/use-scroll-to";
import { Section } from "@/components/ui/section";
import { ScrollReveal } from "@/components/scroll-reveal";
import { SectionHeader } from "@/components/ui/page-shell";
import { cn } from "@/lib/utils";

export interface CollectionsSectionProps {
  setCollection: (c: string) => void;
  moods?: { moodId: string; moodName: string; thumbnailUrl?: string }[];
  onPickMood?: (m: { moodId: string; moodName: string }) => void;
  reduceMotion?: boolean;
}

export type DisplayCollection = {
  key: string;
  blurb: string;
  isMood?: boolean;
  moodId?: string;
  thumbnailUrl?: string;
};

type MoodCardVariant = "sm" | "md" | "feature";

const moodCard = {
  sm: {
    media: "aspect-[4/5]",
    panel: "min-h-[118px] p-3.5 md:min-h-[142px] md:p-4 lg:min-h-[154px] flex flex-1 flex-col",
    title: "text-[1.08rem] md:text-[1.3rem]",
  },
  md: {
    media: "aspect-[4/5]",
    panel: "min-h-[118px] p-3.5 md:min-h-[142px] md:p-4 lg:min-h-[154px] flex flex-1 flex-col",
    title: "text-[1.1rem] md:text-[1.45rem]",
  },
  feature: {
    media: "aspect-[4/5] md:aspect-[5/6] lg:aspect-[4/5]",
    panel: "min-h-[118px] p-3.5 md:min-h-[142px] lg:min-h-[154px] md:p-5 flex flex-1 flex-col",
    title: "text-[1.1rem] md:text-[1.55rem]",
  },
} as const;

export function CollectionCard({
  c,
  idx,
  setCollection,
  onPickMood,
  reduceMotion,
  variant = "md",
}: {
  c: DisplayCollection;
  idx: number;
  setCollection: (x: string) => void;
  onPickMood?: (m: { moodId: string; moodName: string }) => void;
  reduceMotion: boolean;
  variant?: MoodCardVariant;
}) {
  const imgSrc = c.thumbnailUrl || COLLECTION_IMAGES[idx % COLLECTION_IMAGES.length];
  const tone = moodCard[variant];

  return (
    <button
      type="button"
      onClick={() => {
        if (c.isMood && c.moodId && onPickMood) {
          onPickMood({ moodId: c.moodId, moodName: c.key });
          goTo("shop", reduceMotion);
          return;
        }
        setCollection(c.isMood ? "All" : c.key);
        goTo("shop", reduceMotion);
      }}
      className={cn(
        "group h-full w-full overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] text-left shadow-[var(--shadow-subtle)]",
        variant === "feature" && "border-[var(--color-line-strong)] shadow-[var(--shadow-soft)]"
      )}
    >
      <div className={cn("relative w-full overflow-hidden", tone.media)}>
        <Image
          src={imgSrc}
          alt={c.key}
          fill
          className="object-cover transition duration-500 ease-out group-hover:scale-[1.03]"
          sizes="(max-width: 768px) 50vw, 25vw"
          loading="lazy"
        />
      </div>
      <div className={cn("border-t border-[var(--color-line)]", tone.panel)}>
        <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-[var(--color-muted)]">
          {c.isMood ? "Mood" : "Collection"}
        </p>
        <p
          className={cn(
            "mt-1 h-[2.8rem] overflow-hidden font-display leading-[1.15] text-[var(--color-ink)] md:h-[3.1rem] lg:h-[3.45rem]",
            tone.title
          )}
          style={{ display: "-webkit-box", WebkitLineClamp: 2, WebkitBoxOrient: "vertical" }}
        >
          {c.blurb}
        </p>
        <span className="mt-auto pt-3 inline-flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-green)]">
          Explore
          <ChevronRight className="h-3.5 w-3.5" />
        </span>
      </div>
    </button>
  );
}

export function CollectionsSection({
  setCollection,
  moods = [],
  onPickMood,
  reduceMotion = false,
}: CollectionsSectionProps) {
  const [showAll, setShowAll] = useState(false);

  const allMoodItems: DisplayCollection[] = moods.map((m) => ({
    key: m.moodName,
    blurb: m.moodName,
    isMood: true,
    moodId: m.moodId,
    thumbnailUrl: m.thumbnailUrl,
  }));

  const defaultCollections: DisplayCollection[] = COLLECTIONS.map((item) => ({
    key: item.key,
    blurb: item.blurb,
  }));

  const displayCollections: DisplayCollection[] =
    moods.length > 0
      ? showAll
        ? allMoodItems
        : allMoodItems.slice(0, 6)
      : defaultCollections.slice(0, 6);

  if (displayCollections.length === 0) return null;

  return (
    <Section id="collections" compact className="relative z-0 bg-[var(--background)] pt-4 md:pt-6">
      <ScrollReveal>
        <SectionHeader
          title="Signature Moods & Collections"
          centered
          className="pb-4 md:pb-5"
          action={
            moods.length > 6 ? (
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

      <div className="mx-auto mt-4 grid max-w-[1300px] grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 sm:gap-5">
        <AnimatePresence initial={false}>
          {displayCollections.map((item, idx) => (
            <motion.div
              key={item.key}
              initial={{ opacity: 0, y: 14 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: 10 }}
              transition={{ duration: 0.28, ease: [0.22, 1, 0.36, 1] }}
              className="h-full"
            >
              <CollectionCard
                c={item}
                idx={idx}
                setCollection={setCollection}
                onPickMood={onPickMood}
                reduceMotion={reduceMotion}
                variant="md"
              />
            </motion.div>
          ))}
        </AnimatePresence>
      </div>
    </Section>
  );
}
