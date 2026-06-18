"use client";

import Image from "next/image";
import { motion, useReducedMotion } from "framer-motion";
import { ArrowRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { HeroHeading, Kicker } from "@/components/ui/typography";
import { goTo } from "@/hooks/use-scroll-to";

export function HeroSection() {
  const reduce = useReducedMotion();

  return (
    <motion.section
      className="pt-4 md:pt-6"
      initial={{ opacity: 0, y: reduce ? 0 : 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: reduce ? 0 : 0.8, ease: [0.22, 1, 0.36, 1] }}
    >
      <div className="mx-auto w-full max-w-[var(--container-max)] px-[var(--gutter-mobile)] md:px-[var(--gutter-tablet)]">
        <div className="grid overflow-hidden rounded-lg border border-[var(--color-line)] shadow-[var(--shadow-soft)] lg:grid-cols-[30fr_70fr]">
          <div className="bg-deep-feature relative order-2 flex flex-col justify-center gap-5 px-6 py-10 sm:px-10 sm:py-14 lg:order-1 lg:px-10 lg:py-16">
            <div className="ornament-rule ornament-rule--inverse w-28">
              <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-[var(--color-gold)]" />
            </div>
            <Kicker tone="inverse">Sudatta&apos;s Designer Boutique</Kicker>
            <HeroHeading inverse className="max-w-md">
              Heirloom weaves for the modern wardrobe
            </HeroHeading>
            <p className="max-w-sm text-sm leading-relaxed text-[var(--color-on-deep-muted)] sm:text-base">
              Hand-finished sarees, kurtas, and ethnic wear crafted in small batches — designed to be worn often, and kept for years.
            </p>
            <div className="flex flex-wrap gap-3 pt-2">
              <Button
                size="lg"
                onClick={() => goTo("shop")}
                className="rounded-full border-[var(--color-gold)] bg-[var(--color-gold)] text-[var(--color-deep)] hover:border-[var(--color-gold-soft)] hover:bg-[var(--color-gold-soft)]"
              >
                Shop New Arrivals
              </Button>
              <Button
                variant="outline"
                size="lg"
                onClick={() => goTo("collections")}
                className="rounded-full border-white/30 bg-transparent text-[var(--color-on-deep)] hover:border-white hover:bg-white/10 hover:text-white"
              >
                Explore Collections
                <ArrowRight className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <div className="relative order-1 aspect-[5/4] sm:aspect-[16/10] lg:order-2 lg:aspect-auto">
            <Image
              src="/hero/hero-sudattas-6.png"
              alt="Sudatta's models wearing handcrafted sarees in signature weaves and borders"
              fill
              priority
              className="object-cover"
              sizes="(max-width: 1023px) 100vw, 70vw"
            />
          </div>
        </div>
      </div>
    </motion.section>
  );
}
