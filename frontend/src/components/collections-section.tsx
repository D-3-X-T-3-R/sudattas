"use client";

import Image from "next/image";
import { ChevronRight } from "lucide-react";
import { COLLECTIONS } from "@/lib/constants";
import { COLLECTION_IMAGES } from "@/lib/seed-data";
import { goTo } from "@/hooks/use-scroll-to";
import { Section } from "@/components/ui/section";
import { SectionHeading, Kicker } from "@/components/ui/typography";
import { ScrollReveal } from "@/components/scroll-reveal";

export interface CollectionsSectionProps {
  setCollection: (c: string) => void;
  moods?: { moodId: string; moodName: string; thumbnailUrl?: string }[];
  /** Applies searchProduct mood filter and scrolls to shop */
  onPickMood?: (m: { moodId: string; moodName: string }) => void;
  reduceMotion?: boolean;
}

type DisplayCollection = {
  key: string;
  blurb: string;
  isMood?: boolean;
  moodId?: string;
  thumbnailUrl?: string;
};

function CollectionCard({
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
        <span
          className={`mt-2 block uppercase tracking-[0.18em] font-semibold text-[var(--color-accent-gold)] [-webkit-text-stroke:0.3px_white] ${large ? "text-2xl sm:text-3xl md:text-4xl" : "text-2xl sm:text-3xl"}`}
        >
          {c.blurb}
        </span>
        <div className="mt-4 inline-flex items-center gap-2 rounded-full border-2 border-[#1a4a2e] bg-transparent px-5 py-2.5 text-xs font-semibold text-white transition-colors group-hover:bg-[#1a4a2e] group-hover:text-white sm:mt-5">
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
  const displayCollections: DisplayCollection[] =
    moods.length > 0
      ? moods.slice(0, 4).map((m) => ({
          key: m.moodName,
          blurb: m.moodName,
          isMood: true,
          moodId: m.moodId,
          thumbnailUrl: m.thumbnailUrl,
        }))
      : [...COLLECTIONS];

  const first = displayCollections[0];
  const second = displayCollections[1];
  const third = displayCollections[2];
  const fourth = displayCollections[3];

  return (
    <Section id="collections">
      <ScrollReveal>
        <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <Kicker className="text-[var(--color-muted)] invisible">Collections</Kicker>
            <SectionHeading size="lg" className="mt-3 uppercase tracking-[0.18em] text-[var(--color-accent-gold)]">
              {"This week's moods"}
            </SectionHeading>
          </div>
          <button
            type="button"
            onClick={() => goTo("shop", reduceMotion)}
            className="hidden text-xs font-semibold uppercase tracking-[0.18em] text-[var(--color-ink)] transition-colors hover:text-[var(--color-accent-brown)] sm:inline-flex"
          >
            View all
          </button>
        </div>
      </ScrollReveal>

      {moods.length > 0 ? (
        <div className="mt-12 grid grid-cols-2 gap-4 md:grid-cols-4">
          {[first, second, third, fourth].filter(Boolean).map((item, idx) => (
            <div key={`${item!.key}-${idx}`}>
              <CollectionCard
                c={item!}
                idx={idx}
                setCollection={setCollection}
                onPickMood={onPickMood}
                reduceMotion={reduceMotion}
              />
            </div>
          ))}
        </div>
      ) : (
        <div className="mt-12 grid gap-6 md:grid-cols-2 md:grid-rows-2">
          {first && (
            <div className="md:row-span-2">
              <CollectionCard
                c={first}
                idx={0}
                setCollection={setCollection}
                onPickMood={onPickMood}
                reduceMotion={reduceMotion}
                large
              />
            </div>
          )}
          {second && (
            <div>
              <CollectionCard
                c={second}
                idx={1}
                setCollection={setCollection}
                onPickMood={onPickMood}
                reduceMotion={reduceMotion}
              />
            </div>
          )}
          {third && (
            <div>
              <CollectionCard
                c={third}
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
