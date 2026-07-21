"use client";

import Image from "next/image";
import { PageShell } from "@/components/ui/page-shell";
import { Kicker, HeroHeading, SectionHeading } from "@/components/ui/typography";
import { ScrollReveal } from "@/components/scroll-reveal";
import { cn } from "@/lib/utils";

// Placeholder chapters — swap images/copy for the real journal content once provided.
const CHAPTERS = [
  {
    image: "/hero/hero-sudattas-6.png",
    alt: "The beginning of Sudatta's",
    eyebrow: "Chapter One",
    title: "Where it began",
    body: "Sudatta's started as a small idea — to bring heirloom weaves to the modern wardrobe without losing the hand of the craftsperson behind them.",
  },
  {
    image: "/hero/hero-sudattas-full.png",
    alt: "The craft behind Sudatta's",
    eyebrow: "Chapter Two",
    title: "The craft",
    body: "Every piece passes through hands that have spent decades perfecting their weave, print, or embroidery — a quiet, deliberate process that can't be rushed.",
  },
  {
    image: "/hero/hero-sudattas-full-2.png",
    alt: "Sudatta's today",
    eyebrow: "Chapter Three",
    title: "Today",
    body: "From a small atelier to a growing community of women who wear Sudatta's for the moments that matter — this is just the beginning of the story.",
  },
];

export function JournalPageContent() {
  return (
    <PageShell wide containerClassName="py-10 md:py-16">
      <div className="mx-auto max-w-2xl text-center">
        <Kicker tone="accent">The Journal</Kicker>
        <HeroHeading size="sm" className="mt-4">
          The journey of Sudatta&apos;s
        </HeroHeading>
        <p className="mt-4 text-sm leading-relaxed text-[var(--color-muted)] sm:text-base">
          A look behind the loom — the people, the process, and the story still being written.
        </p>
      </div>

      <div className="mt-16 flex flex-col gap-16 md:mt-24 md:gap-28">
        {CHAPTERS.map((chapter, i) => {
          const reverse = i % 2 === 1;
          return (
            <div key={chapter.title} className="grid items-center gap-8 md:grid-cols-2 md:gap-16">
              <ScrollReveal direction={reverse ? "right" : "left"} className={cn(reverse && "md:order-2")}>
                <div className="relative aspect-[4/5] w-full overflow-hidden rounded-lg border border-[var(--color-line)]">
                  <Image
                    src={chapter.image}
                    alt={chapter.alt}
                    fill
                    className="object-cover"
                    sizes="(max-width: 768px) 100vw, 50vw"
                  />
                </div>
              </ScrollReveal>
              <ScrollReveal
                direction={reverse ? "left" : "right"}
                delay={0.1}
                className={cn(reverse && "md:order-1")}
              >
                <p className="text-[11px] font-semibold uppercase tracking-[0.2em] text-[var(--color-gold)]">
                  {chapter.eyebrow}
                </p>
                <SectionHeading className="mt-3">{chapter.title}</SectionHeading>
                <p className="mt-4 max-w-md text-sm leading-relaxed text-[var(--color-muted)] sm:text-base">
                  {chapter.body}
                </p>
              </ScrollReveal>
            </div>
          );
        })}
      </div>
    </PageShell>
  );
}
