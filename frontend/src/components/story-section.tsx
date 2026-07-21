import { Section } from "@/components/ui/section";
import { HeroHeading, Kicker } from "@/components/ui/typography";
import { ScrollReveal } from "@/components/scroll-reveal";

export function StorySection() {
  return (
    <Section id="story" fullWidth className="bg-deep-feature py-16 md:py-24">
      <ScrollReveal>
        <div className="mx-auto max-w-2xl px-[var(--gutter-mobile)] text-center md:px-[var(--gutter-tablet)]">
          <Kicker tone="inverse">Stay in the loop</Kicker>
          <HeroHeading inverse size="sm" className="mt-4">
            Get the next drop first.
          </HeroHeading>
          <p className="mt-6 text-sm leading-relaxed text-[var(--color-on-deep-muted)] sm:text-base">
            Weekly releases. No spam. Unsubscribe anytime.
          </p>
          <div className="mx-auto mt-8 flex max-w-md flex-col gap-3 sm:flex-row">
            <label htmlFor="newsletter-email" className="sr-only">
              Email address
            </label>
            <input
              id="newsletter-email"
              type="email"
              autoComplete="email"
              placeholder="you@example.com"
              className="flex-1 rounded-full border border-white/20 bg-white/10 px-5 py-3.5 text-sm text-[var(--color-on-deep)] placeholder:text-[var(--color-on-deep-muted)] outline-none focus:border-[var(--color-gold)] focus:ring-1 focus:ring-[var(--color-gold)]"
            />
            <button
              type="submit"
              className="rounded-full bg-[var(--color-gold)] px-8 py-3.5 text-sm font-semibold text-[var(--color-deep)] transition-colors hover:bg-[var(--color-gold-soft)]"
            >
              Notify me
            </button>
          </div>
        </div>
      </ScrollReveal>
    </Section>
  );
}
