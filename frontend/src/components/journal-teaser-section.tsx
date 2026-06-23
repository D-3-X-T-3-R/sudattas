"use client";

import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Section } from "@/components/ui/section";
import { Kicker } from "@/components/ui/typography";
import { ScrollReveal } from "@/components/scroll-reveal";

// Placeholder collage — swap files in /public/journal for the real photography (see README there).
const COLLAGE_TILES = [
  {
    src: "/journal/journal-1.png",
    alt: "Sudatta's journal",
    className: "left-[0%] top-[4%] h-[48%] w-[30%] -rotate-6",
  },
  {
    src: "/journal/journal-2.png",
    alt: "Sudatta's journal",
    className: "left-[19%] top-[42%] h-[46%] w-[26%] rotate-3",
  },
  {
    src: "/journal/journal-3.png",
    alt: "Sudatta's journal",
    className: "left-[39%] top-[10%] h-[42%] w-[24%] -rotate-2",
  },
  {
    src: "/journal/journal-4.png",
    alt: "Sudatta's journal",
    className: "right-[1%] top-[2%] h-[44%] w-[28%] rotate-6",
  },
  {
    src: "/journal/journal-5.png",
    alt: "Sudatta's journal",
    className: "right-[16%] bottom-[4%] h-[42%] w-[26%] -rotate-3",
  },
];

export function JournalTeaserSection() {
  return (
    <Section id="journal" fullWidth className="overflow-hidden">
      <div className="relative aspect-[4/5] w-full sm:aspect-[16/9] lg:aspect-[21/9]">
        {/* Overlapping photo mosaic, sits behind the frosted glass */}
        <div className="absolute inset-0 bg-[var(--color-deep)]">
          {COLLAGE_TILES.map((tile, i) => (
            <div
              key={i}
              className={`absolute overflow-hidden rounded-md border border-white/10 shadow-[0_16px_36px_rgba(0,0,0,0.4)] ${tile.className}`}
            >
              <Image
                src={tile.src}
                alt={tile.alt}
                fill
                className="object-cover"
                sizes="(max-width: 640px) 40vw, 25vw"
              />
            </div>
          ))}
        </div>

        {/* Frosted glass card floats over the crisp mosaic — only this panel is blurred/washed, not the whole section */}
        <div className="absolute inset-0 flex items-center justify-center px-6">
          <ScrollReveal>
            <div className="flex max-w-md flex-col items-center gap-4 rounded-2xl border border-white/25 bg-white/10 px-8 py-10 text-center shadow-[0_8px_40px_rgba(0,0,0,0.3)] backdrop-blur-xl sm:px-12 sm:py-12">
              <Kicker tone="inverse">The Journal</Kicker>
              <h2 className="font-display text-2xl text-white sm:text-3xl">
                The story behind every weave
              </h2>
              <p className="text-sm text-white/80 sm:text-base">
                From loom to wardrobe — follow the people, places, and process behind Sudatta&apos;s.
              </p>
              <Button
                asChild
                size="lg"
                className="mt-2 rounded-full border-[var(--color-gold)] bg-[var(--color-gold)] text-[var(--color-deep)] hover:border-[var(--color-gold-soft)] hover:bg-[var(--color-gold-soft)]"
              >
                <Link href="/journal">Read the Journal</Link>
              </Button>
            </div>
          </ScrollReveal>
        </div>
      </div>
    </Section>
  );
}
