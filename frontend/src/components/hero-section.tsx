"use client";

import Image from "next/image";
import { motion } from "framer-motion";
import { useReducedMotion } from "framer-motion";

export function HeroSection() {
  const reduce = useReducedMotion();
  return (
    <motion.section
      className="relative bg-[var(--background)]"
      initial={{ opacity: 0, y: reduce ? 0 : 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: reduce ? 0 : 0.8, ease: [0.22, 1, 0.36, 1] }}
    >
      <div className="mx-auto w-full max-w-[2000px] px-4 py-0">
        <div className="block md:hidden">
          <Image
            src="/hero/hero-sudattas-6.png"
            alt="Sudatta's handcrafted sarees hero – mobile"
            width={3200}
            height={2133}
            className="w-full h-auto"
            priority
          />
        </div>
        <div className="hidden md:block">
          <Image
            src="/hero/hero-sudattas-full-2.png"
            alt="Sudatta's handcrafted sarees hero – desktop"
            width={3200}
            height={2133}
            className="w-full h-auto"
            priority
          />
        </div>
      </div>
    </motion.section>
  );
}
