"use client";

import Image from "next/image";
import Link from "next/link";
import { motion, useReducedMotion } from "framer-motion";

export function HeroSection() {
  const reduce = useReducedMotion();

  return (
    <motion.section
      className="pt-6 md:pt-8"
      initial={{ opacity: 0, y: reduce ? 0 : 12 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: reduce ? 0 : 0.7, ease: [0.22, 1, 0.36, 1] }}
    >
      <div className="mx-auto w-full max-w-[var(--container-max)] px-[var(--gutter-mobile)] md:px-[var(--gutter-tablet)]">
        <div className="overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] shadow-[var(--shadow-soft)]">
          <div className="relative">
            <div className="block md:hidden">
              <Image
                src="/hero/hero-sudattas-6.png"
                alt="Sudatta's handcrafted sarees hero - mobile"
                width={3200}
                height={2133}
                className="h-auto w-full"
                priority
              />
            </div>
            <div className="hidden md:block">
              <Image
                src="/hero/hero-sudattas-full-2.png"
                alt="Sudatta's handcrafted sarees hero - desktop"
                width={3200}
                height={2133}
                className="h-auto w-full"
                priority
              />
            </div>
            <div className="pointer-events-none absolute inset-0 bg-gradient-to-r from-black/44 via-black/8 to-transparent" />

            <div className="absolute inset-x-3 bottom-3 rounded-md border border-white/25 bg-[rgba(24,20,16,0.42)] px-4 py-4 text-white backdrop-blur-[2px] sm:inset-x-5 sm:bottom-5 sm:max-w-[340px] sm:px-5 sm:py-5 md:bottom-8 md:left-8 md:right-auto md:max-w-[420px] md:px-6 md:py-6">
              <p className="text-[10px] font-semibold uppercase tracking-[0.24em] text-[var(--color-gold-soft)]">
                Crafted for modern Indian women
              </p>
              <h1 className="mt-2 font-display text-[2rem] leading-[1.1] tracking-[-0.02em] text-white md:text-[3rem]">
                Timeless Elegance, Crafted in India
              </h1>
              <p className="mt-2 text-sm leading-relaxed text-white/85 md:text-base">
                Discover curated sarees and occasion-ready pieces rooted in heritage craftsmanship.
              </p>
              <Link
                href="/#shop"
                className="mt-4 inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-green)] bg-[var(--color-green)] px-5 text-[11px] font-semibold uppercase tracking-[0.16em] text-white transition hover:border-[var(--color-green-2)] hover:bg-[var(--color-green-2)]"
              >
                Explore New Arrivals
              </Link>
            </div>
          </div>
        </div>
      </div>
    </motion.section>
  );
}
