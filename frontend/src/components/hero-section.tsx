"use client";

import { useState, useEffect } from "react";
import Image from "next/image";
import { motion, AnimatePresence, useReducedMotion } from "framer-motion";
import { ChevronLeft, ChevronRight, ChevronDown } from "lucide-react";
import { Button } from "@/components/ui/button";
import { HERO_SLIDES } from "@/lib/seed-data";
import { goTo } from "@/hooks/use-scroll-to";


export function HeroSection() {
  const [slide, setSlide] = useState(0);
  const reduceMotion = useReducedMotion();

  useEffect(() => {
    const t = window.setInterval(() => {
      if (reduceMotion) return;
      setSlide((s) => (s + 1) % HERO_SLIDES.length);
    }, 7000);
    return () => window.clearInterval(t);
  }, [reduceMotion]);

  const nextSlide = () =>
    setSlide((s) => (s + 1) % HERO_SLIDES.length);
  const prevSlide = () =>
    setSlide((s) => (s - 1 + HERO_SLIDES.length) % HERO_SLIDES.length);
  const current = HERO_SLIDES[slide];

  return (
    <section className="relative">
      <div className="relative h-[72vh] min-h-[560px] w-full overflow-hidden">
        <AnimatePresence mode="wait">
          <motion.div
            key={slide}
            initial={{ opacity: 0, scale: 1.02 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 1.02 }}
            transition={{
              duration: reduceMotion ? 0 : 0.7,
              ease: "easeOut",
            }}
            className="absolute inset-0"
          >
            <Image
              src={current.image}
              alt={current.imageAlt}
              fill
              className="object-cover"
              priority
              sizes="100vw"
            />
            <div className="absolute inset-0 bg-gradient-to-t from-black/70 via-black/25 to-transparent" />

            <div className="absolute inset-x-0 bottom-0">
              <div className="mx-auto max-w-7xl px-4 pb-16 pt-24 sm:pb-20">
                <div className="max-w-2xl">
                  <div
                    className="text-[11px] font-semibold tracking-[0.28em] uppercase text-[var(--color-accent-gold)]"
                    style={{ textShadow: "0 1px 2px rgba(0,0,0,0.3)" }}
                  >
                    {current.eyebrow}
                  </div>
                  <h1 className="mt-4 font-display text-4xl font-medium tracking-tight text-white drop-shadow-sm sm:text-6xl md:text-7xl md:leading-[1.1]">
                    {current.title}
                  </h1>
                  <div className="mt-8 flex flex-col gap-3 sm:flex-row sm:items-center">
                    <Button
                      onClick={() => goTo("shop", !!reduceMotion)}
                      className="w-fit rounded-full bg-white px-8 py-3.5 text-sm font-semibold text-[var(--color-ink)] hover:bg-[var(--color-warm-white)]"
                    >
                      {current.cta}
                    </Button>
                    <Button
                      variant="outline"
                      onClick={() => goTo("collections", !!reduceMotion)}
                      className="w-fit rounded-full border-2 border-white/50 bg-transparent px-8 py-3.5 text-sm font-semibold text-white hover:border-white hover:bg-white/10"
                    >
                      Explore collections
                    </Button>
                  </div>
                  <div className="mt-10 flex items-center gap-4 text-xs text-white/80">
                    <span className="inline-flex items-center gap-2">
                      <span className="h-1.5 w-1.5 rounded-full bg-[var(--color-accent-gold)]" />
                      Premium finish, clean drape
                    </span>
                    <span className="hidden h-1 w-px bg-white/40 sm:block" />
                    <span className="hidden sm:inline">Ships in 24–48 hours</span>
                  </div>
                </div>
              </div>
            </div>
          </motion.div>
        </AnimatePresence>

        <Button
          variant="outline"
          size="icon"
          onClick={prevSlide}
          className="absolute left-4 top-1/2 -translate-y-1/2 h-11 w-11 rounded-full bg-white/70 backdrop-blur hover:bg-white"
          aria-label="Previous slide"
        >
          <ChevronLeft className="h-5 w-5 text-[var(--color-ink)]" />
        </Button>
        <Button
          variant="outline"
          size="icon"
          onClick={nextSlide}
          className="absolute right-4 top-1/2 -translate-y-1/2 h-11 w-11 rounded-full bg-white/70 backdrop-blur hover:bg-white"
          aria-label="Next slide"
        >
          <ChevronRight className="h-5 w-5 text-[var(--color-ink)]" />
        </Button>

        <div className="absolute bottom-6 left-1/2 flex -translate-x-1/2 items-center gap-1.5">
          {HERO_SLIDES.map((_, i) => (
            <button
              key={i}
              type="button"
              onClick={() => setSlide(i)}
              className="h-0.5 w-8 rounded-full transition-colors"
              style={{
                background: i === slide ? "white" : "rgba(255,255,255,0.4)",
              }}
              aria-label={`Go to slide ${i + 1}`}
            />
          ))}
        </div>

        <button
          type="button"
          onClick={() => goTo("collections", !!reduceMotion)}
          className="absolute bottom-6 right-4 flex flex-col items-center gap-1 text-white/70 transition-colors hover:text-white md:right-8"
          aria-label="Scroll to collections"
        >
          <span className="text-[10px] font-medium tracking-widest">Discover</span>
          <motion.div
            animate={reduceMotion ? undefined : { y: [0, 6, 0] }}
            transition={{ duration: 2, repeat: Infinity, ease: "easeInOut" }}
          >
            <ChevronDown className="h-5 w-5" />
          </motion.div>
        </button>
      </div>
    </section>
  );
}
