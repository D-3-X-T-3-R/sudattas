"use client";

import Image from "next/image";
import { motion, AnimatePresence, useReducedMotion } from "framer-motion";
import { ChevronDown } from "lucide-react";
import { Button } from "@/components/ui/button";
import { HERO_IMAGES } from "@/lib/seed-data";
import { goTo } from "@/hooks/use-scroll-to";
import { useState, useEffect } from "react";

const ROTATE_INTERVAL_MS = 5000;

export function HeroSection() {
  const reduceMotion = useReducedMotion();
  const [index, setIndex] = useState(0);

  useEffect(() => {
    if (reduceMotion) return;
    const id = setInterval(() => {
      setIndex((i) => (i + 1) % HERO_IMAGES.length);
    }, ROTATE_INTERVAL_MS);
    return () => clearInterval(id);
  }, [reduceMotion]);

  return (
    <section className="relative min-h-screen overflow-hidden bg-[#0d0d0d]">
      <div className="relative mx-auto flex min-h-screen w-full max-w-[2560px] items-center justify-center px-4 sm:px-6 md:px-8">
        {/* Centered row: text | gap | image */}
        <div className="flex w-full max-w-6xl flex-col items-center justify-center gap-10 md:flex-row md:gap-16 lg:gap-20">
          {/* Hero copy */}
          <motion.div
            className="flex shrink-0 flex-col items-center text-center md:items-start md:text-left"
            initial={reduceMotion ? false : { opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.2, ease: "easeOut" }}
          >
            <h1
              className="font-[var(--font-hero)] text-3xl font-medium leading-tight text-white sm:text-4xl md:text-5xl"
              style={{ textShadow: "0 2px 8px rgba(0,0,0,0.3)" }}
            >
              Handcrafted Sarees
              <br />
              for the Modern Woman
            </h1>
            <Button
              onClick={() => goTo("shop", !!reduceMotion)}
              className="mt-8 w-fit rounded-full bg-white px-8 py-3.5 text-sm font-semibold text-[var(--color-ink)] hover:bg-[var(--color-warm-white)]"
            >
              Explore Collection
            </Button>
          </motion.div>

          {/* Gap is handled by gap-10/16/20 on the flex container */}

          {/* Single image */}
          <div className="relative h-[50vh] min-h-[320px] w-full max-w-lg flex-1 md:h-[60vh] md:min-h-[400px]">
            <AnimatePresence mode="wait" initial={false}>
              <motion.div
                key={index}
                className="absolute inset-0 flex items-end justify-center"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.5, ease: "easeInOut" }}
              >
                <div
                  className="relative h-full w-full"
                  style={{
                    filter: "drop-shadow(0 50px 80px rgba(0,0,0,0.6)) drop-shadow(0 20px 40px rgba(0,0,0,0.4))",
                  }}
                >
                  <Image
                    src={HERO_IMAGES[index].src}
                    alt={HERO_IMAGES[index].alt}
                    fill
                    className="object-contain object-bottom"
                    priority={index === 0}
                    sizes="(max-width: 768px) 100vw, 512px"
                  />
                </div>
              </motion.div>
            </AnimatePresence>
          </div>
        </div>

        {/* Dots */}
        <div className="absolute bottom-6 left-1/2 z-20 flex -translate-x-1/2 gap-2">
          {HERO_IMAGES.map((_, i) => (
            <button
              key={i}
              type="button"
              onClick={() => setIndex(i)}
              className="h-2 w-2 rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-white/50"
              style={{
                backgroundColor: i === index ? "rgba(255,255,255,0.9)" : "rgba(255,255,255,0.35)",
              }}
              aria-label={`Go to slide ${i + 1}`}
            />
          ))}
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
