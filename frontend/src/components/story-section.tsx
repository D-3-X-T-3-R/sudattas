"use client";

import { ScrollReveal } from "@/components/scroll-reveal";

export function StorySection() {
  return (
    <section id="story" className="relative mx-auto max-w-7xl px-4 pb-24">
      <ScrollReveal>
        <div className="grid gap-16 md:grid-cols-2">
          <div className="relative border-t border-[var(--color-line)] pt-14">
            <span
              className="absolute right-0 top-4 font-display text-[8rem] font-medium leading-none text-[var(--color-line)] select-none md:top-6 md:text-[10rem]"
              aria-hidden
            >
              01
            </span>
          <div className="flex items-center gap-2">
            <span className="h-px w-6 bg-[var(--color-accent-gold)]" />
            <span className="text-[11px] font-semibold tracking-[0.26em] text-[var(--color-muted)]">
              OUR POINT OF VIEW
            </span>
          </div>
          <h3 className="mt-4 font-display text-3xl font-medium tracking-tight text-[var(--color-ink)] sm:text-4xl sm:leading-[1.2]">
            Premium is calm.
          </h3>
          <p className="mt-6 max-w-md text-sm leading-relaxed text-[var(--color-muted)]">
            If your website feels busy, the product feels cheaper. We keep the
            layout clean, the typography deliberate, and the interactions
            subtle—so your sarees feel expensive before a user even scrolls.
          </p>
          <div className="mt-10 grid gap-4">
            {[
              "Clarity over clutter",
              "Editorial imagery",
              "Details that feel intentional",
            ].map((x) => (
              <div key={x} className="flex items-center gap-4">
                <span className="h-2 w-2 shrink-0 rounded-full bg-[var(--color-accent-brown)]" />
                <span className="text-sm font-medium text-[var(--color-ink)]">{x}</span>
              </div>
            ))}
          </div>
          </div>

          <div className="border-t border-[var(--color-line)] pt-14">
          <div className="flex items-center gap-2">
            <span className="h-px w-6 bg-[var(--color-accent-gold)]" />
            <span className="text-[11px] font-semibold tracking-[0.26em] text-[var(--color-muted)]">
              STAY IN THE LOOP
            </span>
          </div>
          <h3 className="mt-4 font-display text-3xl font-medium tracking-tight text-[var(--color-ink)] sm:text-4xl">
            Get the next drop first.
          </h3>
          <p className="mt-6 text-sm leading-relaxed text-[var(--color-muted)]">
            Weekly releases. No spam. Unsubscribe anytime.
          </p>
          <div className="mt-8 flex flex-col gap-3 sm:flex-row">
            <input
              placeholder="you@example.com"
              className="flex-1 rounded-full border border-[var(--color-line)] bg-white/70 px-5 py-3.5 text-sm outline-none focus:border-[var(--color-accent-gold)] focus:bg-white focus:ring-1 focus:ring-[var(--color-accent-gold)]"
            />
            <button
              type="button"
              className="rounded-full bg-[var(--color-ink)] px-8 py-3.5 text-sm font-semibold text-white transition-opacity hover:opacity-90"
              onClick={() => alert("Mock: wire to backend later")}
            >
              Notify me
            </button>
          </div>
        </div>
        </div>
      </ScrollReveal>
    </section>
  );
}
