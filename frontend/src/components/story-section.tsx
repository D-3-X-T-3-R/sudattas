"use client";

import { useState, type FormEvent } from "react";
import { Section } from "@/components/ui/section";
import { HeroHeading, Kicker } from "@/components/ui/typography";
import { ScrollReveal } from "@/components/scroll-reveal";
import { subscribeToNewsletter, NewsletterSignupError } from "@/lib/newsletter-api";

export function StorySection() {
  const [email, setEmail] = useState("");
  const [status, setStatus] = useState<"idle" | "submitting" | "done">("idle");
  const [error, setError] = useState("");

  const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (status === "submitting") return;
    setError("");
    setStatus("submitting");
    try {
      await subscribeToNewsletter(email);
      setStatus("done");
      setEmail("");
    } catch (err) {
      setStatus("idle");
      setError(
        err instanceof NewsletterSignupError ? err.message : "Could not subscribe. Please try again."
      );
    }
  };

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
          {status === "done" ? (
            <p className="mx-auto mt-8 max-w-md text-sm font-medium text-[var(--color-gold)]">
              You&rsquo;re in — thanks for subscribing.
            </p>
          ) : (
            <form
              onSubmit={handleSubmit}
              className="mx-auto mt-8 flex max-w-md flex-col gap-3 sm:flex-row"
              noValidate
            >
              <label htmlFor="newsletter-email" className="sr-only">
                Email address
              </label>
              <input
                id="newsletter-email"
                type="email"
                autoComplete="email"
                required
                placeholder="you@example.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                disabled={status === "submitting"}
                className="flex-1 rounded-full border border-white/20 bg-white/10 px-5 py-3.5 text-sm text-[var(--color-on-deep)] placeholder:text-[var(--color-on-deep-muted)] outline-none focus:border-[var(--color-gold)] focus:ring-1 focus:ring-[var(--color-gold)]"
              />
              <button
                type="submit"
                disabled={status === "submitting"}
                className="rounded-full bg-[var(--color-gold)] px-8 py-3.5 text-sm font-semibold text-[var(--color-deep)] transition-colors hover:bg-[var(--color-gold-soft)] disabled:opacity-60"
              >
                {status === "submitting" ? "Submitting…" : "Notify me"}
              </button>
            </form>
          )}
          {error ? (
            <p className="mt-3 text-sm text-red-300" role="alert">
              {error}
            </p>
          ) : null}
        </div>
      </ScrollReveal>
    </Section>
  );
}
