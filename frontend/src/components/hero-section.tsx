"use client";

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
          <div className="relative aspect-[3200/2133]">
            <video
              className="h-full w-full object-cover"
              autoPlay
              muted
              loop
              playsInline
              preload="metadata"
              poster="/hero/hero-sudattas-full-2.png"
            >
              <source src="/videos/hero_brand.mp4" type="video/mp4" />
            </video>
          </div>
        </div>
      </div>
    </motion.section>
  );
}
