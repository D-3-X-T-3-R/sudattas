"use client";

import { useEffect, useRef, useState, type ReactNode } from "react";
import { cn } from "@/lib/utils";

export function MobileCardRow({
  children,
  className,
  itemCount,
  label,
  reduceMotion = false,
}: {
  children: ReactNode;
  className?: string;
  itemCount: number;
  label: string;
  reduceMotion?: boolean;
}) {
  const rowRef = useRef<HTMLDivElement>(null);
  const [active, setActive] = useState(0);
  // Two cards are visible; each dot advances the row by one card.
  const positions = Math.max(1, itemCount - 1);

  useEffect(() => {
    const row = rowRef.current;
    if (!row) return;
    const update = () => {
      const maxScroll = row.scrollWidth - row.clientWidth;
      setActive(maxScroll > 0
        ? Math.max(0, Math.min(positions - 1, Math.round(row.scrollLeft / maxScroll * (positions - 1))))
        : 0);
    };
    update();
    row.addEventListener("scroll", update, { passive: true });
    const observer = new ResizeObserver(update);
    observer.observe(row);
    return () => {
      row.removeEventListener("scroll", update);
      observer.disconnect();
    };
  }, [positions]);

  return (
    <>
      <div ref={rowRef} className={className}>{children}</div>
      {positions > 1 && (
        <nav aria-label={`${label} carousel navigation`} className="flex flex-wrap justify-center md:hidden">
          {Array.from({ length: positions }, (_, index) => (
            <button
              key={index}
              type="button"
              aria-label={`Show ${label.toLowerCase()} ${index + 1} and ${index + 2}`}
              aria-current={active === index ? "true" : undefined}
              className="flex h-8 w-6 items-center justify-center rounded-full"
              onClick={() => {
                const row = rowRef.current;
                if (!row) return;
                const prefersReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
                row.scrollTo({
                  left: index / (positions - 1) * (row.scrollWidth - row.clientWidth),
                  behavior: reduceMotion || prefersReducedMotion ? "instant" : "smooth",
                });
              }}
            >
              <span className={cn("h-1.5 w-1.5 rounded-full", active === index ? "bg-black" : "bg-[#9CA0A5]")} />
            </button>
          ))}
        </nav>
      )}
    </>
  );
}
