"use client";

import { useState } from "react";
import Image from "next/image";
import { ChevronRight } from "lucide-react";
import { AnimatePresence, motion } from "framer-motion";
import { COLLECTIONS } from "@/lib/constants";
import { COLLECTION_IMAGES } from "@/lib/seed-data";
import { goTo } from "@/hooks/use-scroll-to";
import { Section } from "@/components/ui/section";
import { SectionHeading } from "@/components/ui/typography";
import { ScrollReveal } from "@/components/scroll-reveal";

export interface CollectionsSectionProps {
  setCollection: (c: string) => void;
  moods?: { moodId: string; moodName: string; thumbnailUrl?: string }[];
  /** Applies searchProduct mood filter and scrolls to shop */
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

export function CollectionCard({
  c,
  idx,
  setCollection,
  onPickMood,
  reduceMotion,
  large = false,
}: {
  c: DisplayCollection;
  idx: number;
  setCollection: (x: string) => void;
  onPickMood?: (m: { moodId: string; moodName: string }) => void;
  reduceMotion: boolean;
  large?: boolean;
}) {
  const imgSrc = c.thumbnailUrl || COLLECTION_IMAGES[idx % COLLECTION_IMAGES.length];
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
      className={`group relative w-full overflow-hidden rounded-sm bg-white text-left shadow-[0_1px_3px_rgba(26,24,20,0.06)] transition-shadow duration-300 hover:shadow-[0_8px_32px_rgba(26,24,20,0.1)] ${large ? "md:row-span-2" : ""}`}
    >
      <div
        className={`relative w-full ${
          large ? "aspect-[3/4] md:aspect-[3/4] md:min-h-[480px]" : "aspect-[3/4]"
        }`}
      >
        <Image
          src={imgSrc}
          alt={c.key}
          fill
          className="object-cover transition duration-500 ease-out group-hover:scale-[1.03]"
          sizes={large ? "(max-width: 768px) 100vw, 50vw" : "(max-width: 768px) 100vw, 50vw"}
          loading={large ? "eager" : "lazy"}
        />
      </div>
      <div className="absolute inset-0 bg-gradient-to-t from-black/60 via-black/15 to-transparent" />
      <div className="absolute inset-x-0 bottom-0 p-6 text-left sm:p-8">
        <p className="text-xs font-semibold uppercase tracking-[0.18em] text-white/60">
          {c.isMood ? "Mood" : "Collection"}
        </p>
        <span
          className={`mt-1 block uppercase tracking-[0.18em] font-semibold text-[var(--color-accent-gold)] [text-shadow:0_1px_4px_rgba(0,0,0,0.55)] ${large ? "text-2xl sm:text-3xl md:text-4xl" : "text-2xl sm:text-3xl"}`}
        >
          {c.blurb}
        </span>
        <div className="mt-4 inline-flex items-center gap-2 rounded-full border-2 border-[var(--color-accent-gold)] bg-transparent px-5 py-2.5 text-xs font-semibold text-white transition-colors group-hover:bg-[var(--color-accent-gold)] group-hover:text-white sm:mt-5">
          Explore
          <ChevronRight className="h-4 w-4" />
        </div>
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

  const displayCollections: DisplayCollection[] =
    moods.length > 0
      ? (showAll ? allMoodItems : allMoodItems.slice(0, 4))
      : [...COLLECTIONS];

  return (
    <Section id="collections">
      <ScrollReveal>
        <div className="flex flex-col gap-4 border-b border-[var(--color-line)] pb-8 sm:flex-row sm:items-end sm:justify-between">
          <SectionHeading size="lg" className="uppercase tracking-[0.18em] text-[var(--color-accent-gold)]">
            {"This week's moods"}
          </SectionHeading>
          <button
            type="button"
            onClick={() => moods.length > 0 ? setShowAll((v) => !v) : goTo("shop", reduceMotion)}
            className="hidden text-xs font-semibold uppercase tracking-[0.18em] text-[var(--color-ink)] transition-colors hover:text-[var(--color-accent-brown)] sm:inline-flex"
          >
            {moods.length > 0 && showAll ? "Show less" : "View all"}
          </button>
        </div>
      </ScrollReveal>

      {moods.length > 0 ? (
        <div className="mt-12 grid grid-cols-2 gap-4 md:grid-cols-4">
          <AnimatePresence initial={false}>
            {displayCollections.map((item, idx) => (
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
                  setCollection={setCollection}
                  onPickMood={onPickMood}
                  reduceMotion={reduceMotion}
                />
              </motion.div>
            ))}
          </AnimatePresence>
        </div>
      ) : (
        <div className="mt-12 grid gap-6 md:grid-cols-2 md:grid-rows-2">
          {displayCollections[0] && (
            <div className="md:row-span-2">
              <CollectionCard
                c={displayCollections[0]}
                idx={0}
                setCollection={setCollection}
                onPickMood={onPickMood}
                reduceMotion={reduceMotion}
                large
              />
            </div>
          )}
          {displayCollections[1] && (
            <div>
              <CollectionCard
                c={displayCollections[1]}
                idx={1}
                setCollection={setCollection}
                onPickMood={onPickMood}
                reduceMotion={reduceMotion}
              />
            </div>
          )}
          {displayCollections[2] && (
            <div>
              <CollectionCard
                c={displayCollections[2]}
                idx={2}
                setCollection={setCollection}
                onPickMood={onPickMood}
                reduceMotion={reduceMotion}
              />
            </div>
          )}
        </div>
      )}
    </Section>
  );
}
