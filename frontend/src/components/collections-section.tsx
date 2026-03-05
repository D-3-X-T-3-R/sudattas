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
      <div
        className={`relative w-full ${
          large ? "aspect-[3/4] md:aspect-[3/4] md:min-h-[480px]" : "aspect-[16/10]"
        }`}
      >
        <Image
          src={COLLECTION_IMAGES[idx % COLLECTION_IMAGES.length]}
          alt={c.key}
          fill
          className="object-cover transition duration-500 ease-out group-hover:scale-[1.03]"
          sizes={large ? "(max-width: 768px) 100vw, 50vw" : "(max-width: 768px) 100vw, 50vw"}
          loading={large ? "eager" : "lazy"}
        />
      </div>
      <div className="absolute inset-0 bg-gradient-to-t from-black/60 via-black/15 to-transparent" />
      <div className="absolute inset-x-0 bottom-0 p-6 text-left sm:p-8">
        <Kicker tone="accent">{c.key.toUpperCase()}</Kicker>
        <span
          className={`mt-2 block font-display font-medium tracking-tight text-white ${large ? "text-2xl sm:text-3xl md:text-4xl" : "text-2xl sm:text-3xl"}`}
        >
          {c.blurb}
        </span>
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
    <Section id="collections">
      <ScrollReveal>
        <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <Kicker className="text-[var(--color-muted)]">Collections</Kicker>
            <SectionHeading size="lg" className="mt-3">
              Shop by mood
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
    </Section>
  );
}
