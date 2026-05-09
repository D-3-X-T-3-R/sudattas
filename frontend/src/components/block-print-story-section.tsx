"use client";

import { BadgeCheck, PackageCheck, Scissors, Sparkles } from "lucide-react";
import { Section } from "@/components/ui/section";
import { SectionHeading, Kicker } from "@/components/ui/typography";

const CRAFT_POINTS = [
  { label: "Block-print inspired detailing", icon: Sparkles },
  { label: "Stitching and finishing checks", icon: Scissors },
  { label: "Textile-first selection", icon: BadgeCheck },
  { label: "Carefully packed before dispatch", icon: PackageCheck },
] as const;

export function BlockPrintStorySection() {
  return (
    <Section id="block-print-story" className="pb-2 md:pb-4">
      <div className="rounded-[1.75rem] border border-[var(--color-line)] bg-[var(--color-surface)] p-5 shadow-[var(--shadow-soft)] md:p-8 lg:p-10">
        <div className="grid items-center gap-10 lg:grid-cols-[minmax(0,1fr)_minmax(320px,420px)] lg:gap-14">
          <div className="max-w-2xl">
            <Kicker className="text-[var(--color-muted)]">
              The Block Print Story
            </Kicker>
            <SectionHeading size="lg" className="mt-4 max-w-xl">
              Print, stitch, rhythm, and patience.
            </SectionHeading>
            <p className="mt-6 max-w-xl text-sm leading-7 text-[var(--color-muted)] md:text-base md:leading-8">
              Every print begins with touch, pressure, rhythm, and care. Our
              block-print inspired pieces are chosen for their textile
              character, quiet irregularities, and boutique finish.
            </p>

            <ul className="mt-8 grid gap-3 sm:grid-cols-2">
              {CRAFT_POINTS.map(({ label, icon: Icon }) => (
                <li
                  key={label}
                  className="flex items-center gap-3 rounded-md border border-[var(--color-line)]/75 bg-[var(--color-surface-soft)] px-3.5 py-3"
                >
                  <span className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-sm border border-[var(--color-line)] bg-white text-[var(--color-green)]">
                    <Icon className="h-4 w-4" />
                  </span>
                  <span className="text-[11px] font-semibold uppercase tracking-[0.14em] text-[var(--color-ink)]">
                    {label}
                  </span>
                </li>
              ))}
            </ul>
          </div>

          <div className="mx-auto aspect-[9/16] w-full max-w-[420px] overflow-hidden rounded-[2rem] border border-[var(--color-line)] bg-[var(--color-green)]/10 shadow-[var(--shadow-soft)]">
            <video
              className="h-full w-full object-cover"
              autoPlay
              muted
              loop
              playsInline
              preload="metadata"
            >
              <source src="/videos/block_print_story.mp4" type="video/mp4" />
            </video>
          </div>
        </div>
      </div>
    </Section>
  );
}
