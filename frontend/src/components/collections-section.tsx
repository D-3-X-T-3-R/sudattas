"use client";

import { useState } from "react";
import { MobileCardRow } from "@/components/ui/mobile-card-row";
import Image from "next/image";
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
    media: "aspect-[2/3]",
    panel: "p-3.5 md:p-4",
    title: "text-[1.08rem] md:text-[1.3rem]",
  },
  md: {
    media: "aspect-[2/3]",
    panel: "p-3.5 md:p-4",
    title: "text-[1.1rem] md:text-[1.45rem]",
  },
  feature: {
    media: "aspect-[2/3]",
    panel: "p-3.5 md:p-5",
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
          sizes="(max-width: 1023px) 50vw, 33vw"
          loading="lazy"
        />
      </div>
      <div className={cn("border-t border-[var(--color-line)]", tone.panel)}>
        <p
          className={cn(
            "overflow-hidden font-display leading-[1.15] text-[var(--color-ink)]",
            tone.title
          )}
          style={{ display: "-webkit-box", WebkitLineClamp: 2, WebkitBoxOrient: "vertical" }}
        >
          {c.blurb}
        </p>
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
    moods.length > 0 ? allMoodItems : defaultCollections;

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
                className="hidden text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-green)] hover:text-[var(--color-gold)] md:inline-flex"
              >
                {showAll ? "Show Less" : "View All"}
              </button>
            ) : null
          }
        />
      </ScrollReveal>

      <MobileCardRow itemCount={displayCollections.length} label="Moods" reduceMotion={reduceMotion} className="mx-auto mt-4 flex max-w-[1300px] snap-x snap-mandatory gap-3 overflow-x-auto overscroll-x-contain [scrollbar-width:none] [&::-webkit-scrollbar]:hidden pb-3 md:grid md:snap-none md:grid-cols-2 md:gap-5 md:overflow-visible md:pb-0 lg:grid-cols-3">
        <AnimatePresence initial={false}>
          {displayCollections.map((item, idx) => (
            <motion.div
              key={item.key}
              initial={{ opacity: 0, y: 14 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: 10 }}
              transition={{ duration: 0.28, ease: [0.22, 1, 0.36, 1] }}
              className={`h-full min-w-0 shrink-0 basis-[calc((100%-0.75rem)/2)] snap-start md:basis-auto ${idx >= 6 && (!showAll || moods.length === 0) ? "md:hidden" : ""}`}
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
      </MobileCardRow>
    </Section>
  );
}
