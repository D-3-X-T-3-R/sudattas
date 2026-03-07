"use client";

import Image from "next/image";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Section } from "@/components/ui/section";
import { SectionHeading, Kicker } from "@/components/ui/typography";
import { ScrollReveal } from "@/components/scroll-reveal";
import { goTo } from "@/hooks/use-scroll-to";

const EDITORIAL = {
  image:
    "https://images.unsplash.com/photo-1594938298603-c8148c4dae35?auto=format&fit=crop&w=1400&q=80",
  imageAlt: "Editorial storytelling image",
  kicker: "THE CRAFT",
  headline: "Where tradition meets intention",
  body: "Every piece is conceived with restraint—fewer details, better execution. No clutter, no noise.",
  ctaLabel: "Explore the collection",
};

export function EditorialBlock() {
  return (
    <Section fullWidth className="overflow-hidden bg-[var(--color-ivory)]">
      <ScrollReveal delay={0}>
        <div className="mx-auto max-w-7xl px-4">
          <div className="grid gap-16 lg:grid-cols-12 lg:gap-24">
            <div className="relative aspect-[4/5] min-h-[400px] overflow-hidden lg:col-span-7">
              <motion.div
                initial={{ opacity: 0, y: 12 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true, margin: "-80px" }}
                transition={{ duration: 0.5, ease: "easeOut" }}
                className="absolute inset-0"
              >
                <Image
                  src={EDITORIAL.image}
                  alt={EDITORIAL.imageAlt}
                  fill
                  className="object-cover"
                  sizes="(max-width: 1024px) 100vw, 58vw"
                  priority={false}
                />
              </motion.div>
            </div>
            <div className="flex flex-col justify-center lg:col-span-5 lg:py-20">
              <Kicker className="text-[var(--color-muted)]">
                {EDITORIAL.kicker}
              </Kicker>
              <SectionHeading
                size="lg"
                className="mt-4 text-2xl md:text-3xl lg:text-4xl"
              >
                {EDITORIAL.headline}
              </SectionHeading>
              <p className="mt-8 max-w-md text-sm leading-relaxed text-[var(--color-muted)]">
                {EDITORIAL.body}
              </p>
              <Button
                variant="outline"
                className="mt-12 w-fit rounded-full border-[var(--color-line)] px-8 py-3.5 font-semibold"
                onClick={() => goTo("shop", false)}
              >
                {EDITORIAL.ctaLabel}
              </Button>
            </div>
          </div>
        </div>
      </ScrollReveal>
    </Section>
  );
}
