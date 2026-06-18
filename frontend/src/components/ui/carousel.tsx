"use client";

import { Children, useRef } from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";

export interface ScrollCarouselProps {
  children: React.ReactNode;
  className?: string;
  itemClassName?: string;
}

export function ScrollCarousel({ children, className, itemClassName }: ScrollCarouselProps) {
  const trackRef = useRef<HTMLDivElement>(null);

  const scrollByPage = (direction: 1 | -1) => {
    const node = trackRef.current;
    if (!node) return;
    node.scrollBy({ left: direction * node.clientWidth * 0.85, behavior: "smooth" });
  };

  return (
    <div className={cn(className)}>
      <div
        ref={trackRef}
        className="flex snap-x snap-mandatory gap-4 overflow-x-auto scroll-smooth pb-2 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
      >
        {Children.map(children, (child) => (
          <div className={cn("w-[68%] shrink-0 snap-start sm:w-[42%] lg:w-[24%]", itemClassName)}>
            {child}
          </div>
        ))}
      </div>

      <div className="mt-4 hidden justify-end gap-2 md:flex">
        <button
          type="button"
          onClick={() => scrollByPage(-1)}
          aria-label="Scroll to previous items"
          className="flex h-10 w-10 items-center justify-center rounded-full border border-[var(--color-gold)] text-[var(--color-green)] transition-colors hover:bg-[var(--color-surface-soft)]"
        >
          <ChevronLeft className="h-4 w-4" />
        </button>
        <button
          type="button"
          onClick={() => scrollByPage(1)}
          aria-label="Scroll to next items"
          className="flex h-10 w-10 items-center justify-center rounded-full border border-[var(--color-gold)] text-[var(--color-green)] transition-colors hover:bg-[var(--color-surface-soft)]"
        >
          <ChevronRight className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
}
