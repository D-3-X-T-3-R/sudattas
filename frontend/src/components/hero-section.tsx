"use client";

import Image from "next/image";

export function HeroSection() {
  return (
    <section className="relative bg-[var(--background)]">
      <div className="mx-auto w-full max-w-[2000px] px-4 py-0">
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

        {/* Desktop / tablet: composite hero image */}
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
    </section>
  );
}
