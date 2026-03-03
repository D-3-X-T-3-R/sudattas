"use client";

import Image from "next/image";
import { ChevronRight } from "lucide-react";
import { COLLECTIONS } from "@/lib/constants";
import { COLLECTION_IMAGES } from "@/lib/seed-data";
import { goTo } from "@/hooks/use-scroll-to";
import { ScrollReveal } from "@/components/scroll-reveal";

export interface CollectionsSectionProps {
  setCollection: (c: string) => void;
  reduceMotion?: boolean;
}

function CollectionCard({
  c,
  idx,
  setCollection,
  reduceMotion,
  large = false,
}: {
  c: (typeof COLLECTIONS)[number];
  idx: number;
  setCollection: (x: string) => void;
  reduceMotion: boolean;
  large?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={() => {
        setCollection(c.key);
        goTo("shop", reduceMotion);
      }}
      className={`group relative overflow-hidden rounded-sm bg-white text-left shadow-[0_1px_3px_rgba(26,24,20,0.06)] transition-shadow duration-300 hover:shadow-[0_8px_32px_rgba(26,24,20,0.1)] ${large ? "md:row-span-2" : ""}`}
    >
      <div className={`w-full relative ${large ? "aspect-[3/4] md:aspect-auto md:h-full md:min-h-[480px]" : "aspect-[16/10]"}`}>
        <Image
          src={COLLECTION_IMAGES[idx % COLLECTION_IMAGES.length]}
          alt={c.key}
          fill
          className="object-cover transition duration-700 ease-out group-hover:scale-[1.04]"
          sizes={large ? "(max-width: 768px) 100vw, 50vw" : "(max-width: 768px) 100vw, 50vw"}
        />
      </div>
      <div className="absolute inset-0 bg-gradient-to-t from-black/60 via-black/15 to-transparent" />
      <div className="absolute inset-x-0 bottom-0 p-6 text-left sm:p-8">
        <div className="text-[11px] font-semibold tracking-[0.26em] text-[var(--color-accent-gold)]">
          {c.key.toUpperCase()}
        </div>
        <div className={`mt-2 font-display font-medium tracking-tight text-white ${large ? "text-2xl sm:text-3xl md:text-4xl" : "text-2xl sm:text-3xl"}`}>
          {c.blurb}
        </div>
        <div className="mt-4 inline-flex items-center gap-2 rounded-full border-2 border-[var(--color-accent-gold)] bg-transparent px-5 py-2.5 text-xs font-semibold text-white transition-colors group-hover:bg-[var(--color-accent-gold)] sm:mt-5">
          Explore
          <ChevronRight className="h-4 w-4" />
        </div>
      </div>
    </button>
  );
}

export function CollectionsSection({
  setCollection,
  reduceMotion = false,
}: CollectionsSectionProps) {
  return (
    <section id="collections" className="mx-auto max-w-7xl px-4 py-20">
      <ScrollReveal>
        <div className="flex items-end justify-between gap-4">
          <div>
            <div className="flex items-center gap-2">
              <span className="h-px w-6 bg-[var(--color-accent-gold)]" />
              <span className="text-[11px] font-semibold tracking-[0.26em] text-[var(--color-muted)]">
                COLLECTIONS
              </span>
            </div>
            <h2 className="mt-3 font-display text-3xl font-medium tracking-tight text-[var(--color-ink)] sm:text-4xl">
              Shop by mood
            </h2>
          </div>
          <button
            type="button"
            onClick={() => goTo("shop", reduceMotion)}
            className="hidden sm:inline-flex items-center gap-2 text-xs font-semibold tracking-[0.2em] text-[var(--color-ink)] hover:text-[var(--color-accent-brown)] transition-colors"
          >
            VIEW ALL
            <ChevronRight className="h-4 w-4" />
          </button>
        </div>
      </ScrollReveal>

      <div className="mt-12 grid gap-6 md:grid-cols-2 md:grid-rows-2">
        <ScrollReveal delay={0.1} className="md:row-span-2">
          <CollectionCard
            c={COLLECTIONS[0]}
            idx={0}
            setCollection={setCollection}
            reduceMotion={reduceMotion}
            large
          />
        </ScrollReveal>
        <ScrollReveal delay={0.15}>
          <CollectionCard
            c={COLLECTIONS[1]}
            idx={1}
            setCollection={setCollection}
            reduceMotion={reduceMotion}
          />
        </ScrollReveal>
        <ScrollReveal delay={0.2}>
          <CollectionCard
            c={COLLECTIONS[2]}
            idx={2}
            setCollection={setCollection}
            reduceMotion={reduceMotion}
          />
        </ScrollReveal>
      </div>
    </section>
  );
}
