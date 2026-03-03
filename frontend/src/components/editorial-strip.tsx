"use client";

import { ScrollReveal } from "@/components/scroll-reveal";

const QUOTE =
  "Craft in every thread. Quiet luxury, not loud labels.";

export function EditorialStrip() {
  return (
    <ScrollReveal>
      <section className="relative w-full overflow-hidden bg-[var(--color-ink)] py-20 sm:py-28">
        <div
          className="absolute inset-0 opacity-20"
          style={{
            backgroundImage: `radial-gradient(ellipse 80% 50% at 50% 50%, var(--color-accent-gold) 0%, transparent 70%)`,
          }}
        />
        <div className="relative mx-auto max-w-4xl px-6 text-center">
          <p className="font-display text-2xl font-medium tracking-tight text-white/95 sm:text-4xl md:text-5xl md:leading-[1.2]">
            {QUOTE}
          </p>
          <span className="mt-6 inline-block h-px w-16 bg-[var(--color-accent-gold)]" />
        </div>
      </section>
    </ScrollReveal>
  );
}
