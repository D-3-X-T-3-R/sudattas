"use client";

import Image from "next/image";
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
          </div>
        </div>
      </div>
    </motion.section>
  );
}
