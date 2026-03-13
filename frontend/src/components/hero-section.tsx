"use client";

import Image from "next/image";
import { motion, useReducedMotion } from "framer-motion";
import { ChevronDown } from "lucide-react";
import { Button } from "@/components/ui/button";
import { HERO_IMAGES } from "@/lib/seed-data";
import { goTo } from "@/hooks/use-scroll-to";

export function HeroSection() {
  const reduceMotion = useReducedMotion();
  const hero = HERO_IMAGES[0];

  return (
    <section className="relative bg-[var(--background)]">
      <div className="mx-auto w-full max-w-7xl px-4 py-0">
        {/* Mobile: models-only hero image */}
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

        {/* Desktop / tablet: composite hero image with logo + models */}
        <div className="hidden md:block">
          <Image
            src={hero.src}
            alt={hero.alt}
            width={3200}
            height={2133}
            className="w-full h-auto"
            priority
          />
        </div>
      </div>

      <button
        type="button"
        onClick={() => goTo("collections", !!reduceMotion)}
        className="absolute bottom-6 right-4 z-30 flex flex-col items-center gap-1 text-white/70 transition-colors hover:text-white md:right-8"
        aria-label="Scroll to collections"
      >
        <span className="text-[10px] font-medium tracking-widest">Discover</span>
        <motion.div
          animate={reduceMotion ? undefined : { y: [0, 4, 0] }}
          transition={{ duration: 2.5, repeat: Infinity, ease: "easeInOut" }}
        >
          <ChevronDown className="h-5 w-5" />
        </motion.div>
      </button>
    </section>
  );
}
